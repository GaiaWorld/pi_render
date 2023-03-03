use std::{hash::Hash, marker::PhantomData, fmt::Debug};

use pi_assets::{asset::{Handle, Asset, GarbageEmpty}, mgr::AssetMgr};
use pi_hash::XHashMap;
use pi_share::Share;

pub const ASSET_SIZE_FOR_UNKOWN: usize = 256;

pub fn bytes_write_to_memory(
    bytes: &[u8],
    offset: usize,
    memory: &mut [u8],
) {
    let mut index = 0;
    for v in bytes.iter() {
        memory[offset + index] = *v;
        index += 1;
    }
}

pub struct AssetDataCenter<K: Clone + Hash + PartialEq + Eq, A: Asset<Key = K>, P: Clone> {
    datamap: XHashMap<K, (A, Option<P>)>,
    asset_mgr: Share<AssetMgr<A>>,
}
impl<K: Clone + Hash + PartialEq + Eq, A: Asset<Key = K>, P: Clone> AssetDataCenter<K, A, P> {
    pub fn new(ref_garbage: bool, capacity: usize, timeout: usize) -> Self {
        let asset_mgr = AssetMgr::<A>::new(GarbageEmpty(), ref_garbage, capacity, timeout);

        Self { datamap: XHashMap::default(), asset_mgr }
    }
    pub fn get(
        &self,
        key: &K,
    ) -> Option<Handle<A>> {
        self.asset_mgr.get(key)
    }
    pub fn check(
        &self,
        key: &K,
    ) -> bool {
        self.datamap.contains_key(key)
    }
    pub fn add(
        &mut self,
        key: &K,
        value: A,
        param: Option<P>,
    ) {
        if !self.datamap.contains_key(key) {
            self.datamap.insert(key.clone(), (value, param));
        }
    }
    pub fn single_create(
        &mut self,
    ) -> XHashMap<K, (Handle<A>, Option<P>)> {
        let mut result = XHashMap::default();
        self.datamap.drain().for_each(|(key, data)| {
            if let Some(range) = self.asset_mgr.insert(key.clone(), data.0) {
                result.insert(key, (range, data.1));
            }
        });
        result
    }
}

pub struct AssetLoader<
    K: Debug + Clone + Hash + PartialEq + Eq,
    I: Clone + Hash + PartialEq + Eq,
    A: Asset<Key = K>,
    P: Clone,
    D: From<(Handle<A>, Option<P>)>
> {
    waits: XHashMap<K, XHashMap<I, I>>,
    p: PhantomData<(A, P, D)>
}
impl<
    K: Debug + Clone + Hash + PartialEq + Eq,
    I: Clone + Hash + PartialEq + Eq,
    A: Asset<Key = K>,
    P: Clone,
    D: From<(Handle<A>, Option<P>)>
> Default for AssetLoader<K, I, A, P, D> {
    fn default() -> Self {
        Self { waits: XHashMap::default(), p: PhantomData }
    }
}
impl<
    K: Debug + Clone + Hash + PartialEq + Eq,
    I: Clone + Hash + PartialEq + Eq,
    A: Asset<Key = K>,
    P: Clone,
    D: From<(Handle<A>, Option<P>)>
> AssetLoader<K, I, A, P, D> {
    pub fn request(
        &mut self,
        id: I,
        key: &K,
    ) {
        if !self.waits.contains_key(key) {
            self.waits.insert(key.clone(), XHashMap::default());
        }

        let list = self.waits.get_mut(key).unwrap();
        list.insert(id.clone(), id);
    }
    pub fn loaded(
        &mut self,
        key: &K,
        value: &(Handle<A>, Option<P>)
    ) -> Vec<(I, D)> {
        let mut result = vec![];
        if let Some(list) = self.waits.get_mut(&key) {
            list.drain().for_each(|(_, id)| {
                result.push((id, D::from(value.clone())))
            });
        }

        result
    }
}