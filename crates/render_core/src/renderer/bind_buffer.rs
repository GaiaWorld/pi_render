use std::{hash::Hash, fmt::Debug, sync::Arc};

use pi_assets::{asset::GarbageEmpty, mgr::AssetMgr};
use pi_share::Share;

use crate::rhi::{device::RenderDevice, RenderQueue};

use super::buffer::{AssetRWBuffer, RWBufferRange, FixedSizeBufferPool};


pub struct BindBufferAllocator {
    base_size: u32,
    base_size_large: u32,
    pub block_size: u32,
    pool_slots: Vec<FixedSizeBufferPool>,
    smallsizelevel: u32,
    asset_mgr: Share<AssetMgr<AssetRWBuffer>>,
}
impl BindBufferAllocator {
    pub fn new(device: &RenderDevice) -> Self {
        Self::create(device, true)
    }
    pub fn create(device: &RenderDevice, buffer_common_use: bool) -> Self {
        let base_size = device.limits().min_uniform_buffer_offset_alignment;
        let block_size = device.limits().max_uniform_buffer_binding_size;
        let base_size_large = u32::min(base_size * 16, block_size / 16).max(base_size);

        let smallsizelevel = base_size_large / base_size;
        let mut pool_slots = vec![];
        for i in 0..smallsizelevel {
            let fixed_size = base_size * (i + 1);
            pool_slots.push(
                FixedSizeBufferPool::create(
                    fixed_size,
                    fixed_size,
                    wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    buffer_common_use
                )
            );
        }

        let large_count = (block_size + base_size_large - 1) / base_size_large;
        for i in 0..large_count {
            let fixed_size = base_size_large * (i + 1);
            pool_slots.push(
                FixedSizeBufferPool::create(
                    fixed_size,
                    fixed_size,
                    wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    buffer_common_use
                )
            );
        }

        let asset_mgr = AssetMgr::<AssetRWBuffer>::new(GarbageEmpty(), false, 16 * 1024 * 1024, 60 * 1000);

        Self {
            base_size,
            base_size_large,
            block_size,
            pool_slots,
            smallsizelevel,
            asset_mgr
        }
    }
    pub fn allocate(&mut self, size: wgpu::DynamicOffset) -> Option<BindBufferRange> {
        let slot_index = if size <= self.base_size_large {
            let slot_index = (size + self.base_size - 1) / self.base_size - 1;
            slot_index
        } else if size <= self.block_size {
            let mut slot_index = (size + self.base_size_large - 1) / self.base_size_large - 1;
            slot_index += self.smallsizelevel;
            slot_index
        } else {
            return None;
        };
        
        if let Some(pool) = self.pool_slots.get_mut(slot_index as usize) {
            if let Some(range) = pool.allocate(&self.asset_mgr) {
                let offset = range.offset();
                Some(BindBufferRange(Arc::new(range), offset))
            } else {
                log::error!("bbbb");
                None
            }
        } else {
            log::error!("AAAA {:?}", (self.pool_slots.len(), slot_index));
            None
        }
    }
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.pool_slots.iter_mut().for_each(|pool| {
            pool.write_buffer(device, queue);
        });
    }
    pub fn asset_mgr(&self) -> &AssetMgr<AssetRWBuffer> {
        &self.asset_mgr
    }
    pub fn size(&self) -> usize {
        let mut result = 0;
        self.pool_slots.iter().for_each(|item| {
            result += item.size();
        });
        result += 4 + 4 + 4 + 4 + 24 + 8;
        result + self.asset_mgr.size()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindBufferRange(pub Arc<RWBufferRange>, pub wgpu::DynamicOffset);
impl std::ops::Deref for BindBufferRange {
    type Target = Arc<RWBufferRange>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
