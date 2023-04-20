use std::{marker::PhantomData, hash::Hash, sync::Arc};

use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_hash::XHashMap;
use pi_share::Share;

use crate::rhi::{device::RenderDevice, RenderQueue};

use super::{vertex_buffer::{KeyVertexBuffer, EVertexBufferRange, VertexBufferAllocator}, vertices::EVerticesBufferUsage};

#[derive(Debug, Default)]
pub struct SingleVertexBufferDataMap {
    vertices: XHashMap<KeyVertexBuffer, Vec<u8>>,
    instance: XHashMap<KeyVertexBuffer, Vec<u8>>,
}
impl SingleVertexBufferDataMap {
    pub fn add(&mut self, key: &KeyVertexBuffer, data: Vec<u8>) {
        if !self.vertices.contains_key(key) {
            self.vertices.insert(key.clone(), data);
        }
    }
    pub fn single_create(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        allocator: &mut VertexBufferAllocator,
        asset_mgr: &Share<AssetMgr<EVertexBufferRange>>,
    ) -> XHashMap<KeyVertexBuffer, EVerticesBufferUsage> {
        let mut result = XHashMap::default();
        self.vertices.drain().for_each(|(key, data)| {
            if let Some(bufferrange) = allocator.create_not_updatable_buffer(device, queue, &data) {
                if let Ok(range) = asset_mgr.insert(key.clone(), bufferrange) {
                    result.insert(key, EVerticesBufferUsage::Other(range));
                }
            }
        });
        result
    }
    pub fn add_instance(&mut self, key: &KeyVertexBuffer, data: Vec<u8>) {
        if !self.instance.contains_key(key) {
            self.instance.insert(key.clone(), data);
        }
    }
    pub fn single_create_instance(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        allocator: &mut VertexBufferAllocator,
    ) -> XHashMap<KeyVertexBuffer, EVerticesBufferUsage> {
        let mut result = XHashMap::default();
        self.instance.drain().for_each(|(key, data)| {
            if let Some(bufferrange) = allocator.create_not_updatable_buffer(device, queue, &data) {
                result.insert(key, EVerticesBufferUsage::EVBRange(Arc::new(bufferrange)) );
            }
        });
        result
    }
}

#[derive(Debug)]
pub struct VertexBufferLoader<T: Clone + Hash + PartialEq + Eq, D: From<EVerticesBufferUsage>> {
    range_waits: XHashMap<KeyVertexBuffer, XHashMap<T, T>>,
    p: PhantomData<D>,
}
impl<T: Clone + Hash + PartialEq + Eq, D: From<EVerticesBufferUsage>> Default for VertexBufferLoader<T, D> {
    fn default() -> Self {
        Self { range_waits: XHashMap::default(), p: PhantomData }
    }
}
impl<T: Clone + Hash + PartialEq + Eq, D: From<EVerticesBufferUsage>> VertexBufferLoader<T, D> {
    pub fn request(
        &mut self,
        id: T,
        key: &KeyVertexBuffer,
        data: Option<Vec<u8>>,
        datamap: &mut SingleVertexBufferDataMap,
    ) {
        if let Some(data) = data {
            datamap.add(key, data);
        }
        if !self.range_waits.contains_key(key) {
            self.range_waits.insert(key.clone(), XHashMap::default());
        }

        let list = self.range_waits.get_mut(key).unwrap();
        log::info!("request >>>>>>>>>>>>>>>>>>>>> {:?}", key);
        list.insert(id.clone(), id);
    }
    pub fn request_instance(
        &mut self,
        id: T,
        key: &KeyVertexBuffer,
        data: Option<Vec<u8>>,
        datamap: &mut SingleVertexBufferDataMap,
    ) {
        if let Some(data) = data {
            datamap.add_instance(key, data);
        }
        if !self.range_waits.contains_key(key) {
            self.range_waits.insert(key.clone(), XHashMap::default());
        }

        let list = self.range_waits.get_mut(key).unwrap();
        log::info!("request >>>>>>>>>>>>>>>>>>>>> {:?}", key);
        list.insert(id.clone(), id);
    }
    pub fn loaded(
        &mut self,
        key: &KeyVertexBuffer,
        range: &EVerticesBufferUsage,
    ) -> Vec<(T, D)> {
        let mut result = vec![];
        if let Some(list) = self.range_waits.get_mut(&key) {
            log::info!(" success  {:?}", list.len());
            list.drain().for_each(|(_, id)| {
                result.push((id, D::from(range.clone())))
            });
        }

        result
    }
}
