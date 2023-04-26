use std::{hash::Hash, marker::PhantomData, fmt::Debug};

use pi_assets::{asset::{Handle, Asset}, mgr::AssetMgr};
use pi_hash::XHashMap;
use pi_share::Share;

pub mod draw_obj;
pub mod draw_sort;
pub mod draw_obj_list;
pub mod vertices;
pub mod indices;
pub mod buffer;
pub mod bind_buffer;
pub mod bind;
pub mod bind_group;
pub mod attributes;
pub mod buildin_data;
pub mod buildin_var;
pub mod shader;
pub mod instance;
pub mod vertex_buffer;
pub mod vertex_buffer_loader;
pub mod vertex_buffer_desc;
pub mod vertex_format;
pub mod pipeline;
pub mod texture;
pub mod sampler;
pub mod shader_stage;
pub mod error;

#[derive(Debug, Default)]
pub struct AssetDataMap<K: Clone + Hash + PartialEq + Eq, A: Asset<Key = K>> {
    datamap: XHashMap<K, A>,
}
impl<K: Clone + Hash + PartialEq + Eq, A: Asset<Key = K>> AssetDataMap<K, A> {
    pub fn add(
        &mut self,
        key: &K,
        value: A,
    ) {
        if !self.datamap.contains_key(key) {
            self.datamap.insert(key.clone(), value);
        }
    }
    pub fn single_create(
        &mut self,
        asset_mgr: &Share<AssetMgr<A>>,
    ) -> XHashMap<K, Handle<A>> {
        let mut result = XHashMap::default();
        self.datamap.drain().for_each(|(key, data)| {
            if let Ok(range) = asset_mgr.insert(key.clone(), data) {
                result.insert(key, range);
            }
        });
        result
    }
}

pub struct AssetLoader<
    K: Debug + Clone + Hash + PartialEq + Eq,
    I: Clone + Hash + PartialEq + Eq,
    A: Asset<Key = K>,
> {
    waits: XHashMap<K, XHashMap<I, I>>,
    p: PhantomData<A>
}
impl<
    K: Debug + Clone + Hash + PartialEq + Eq,
    I: Clone + Hash + PartialEq + Eq,
    A: Asset<Key = K>,
> AssetLoader<K, I, A> {
    pub fn request(
        &mut self,
        id: I,
        key: &K,
        value: Option<A>,
        datamap: &mut AssetDataMap<K, A>,
    ) {
        if let Some(value) = value {
            datamap.add(key, value);
        }
        if !self.waits.contains_key(key) {
            self.waits.insert(key.clone(), XHashMap::default());
        }

        let list = self.waits.get_mut(key).unwrap();
        list.insert(id.clone(), id);
    }
    pub fn loaded(
        &mut self,
        key: &K,
        value: &Handle<A>,
    ) -> Vec<(I, Handle<A>)> {
        let mut result = vec![];
        if let Some(list) = self.waits.get_mut(&key) {
            list.drain().for_each(|(_, id)| {
                result.push((id, value.clone()))
            });
        }

        result
    }
}