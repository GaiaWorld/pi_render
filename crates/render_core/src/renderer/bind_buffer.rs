use std::{hash::Hash, fmt::Debug, sync::Arc};

use derive_deref_rs::Deref;
use pi_assets::{asset::{GarbageEmpty}, mgr::AssetMgr};
use pi_share::{Share};

use crate::rhi::{device::RenderDevice, RenderQueue};

use super::{buffer::{AssetRWBuffer, RWBufferRange, FixedSizeBufferPool}, shader_stage::EShaderStage};


pub struct BindBufferAllocator {
    base_size: u32,
    base_size_large: u32,
    block_size: u32,
    pool_slots: Vec<FixedSizeBufferPool>,
    small_count: u32,
    asset_mgr: Share<AssetMgr<AssetRWBuffer>>,
}
impl BindBufferAllocator {
    pub fn new(device: &RenderDevice) -> Self {
        let base_size = device.limits().min_uniform_buffer_offset_alignment;
        let block_size = device.limits().max_uniform_buffer_binding_size;
        let base_size_large = u32::min(base_size * 16, block_size / 16);

        let small_count = base_size_large / base_size;
        let mut pool_slots = vec![];
        for i in 0..small_count {
            pool_slots.push(
                FixedSizeBufferPool::new(
                    block_size,
                    base_size * (i + 1),
                    wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM
                )
            );
        }

        let large_count = block_size / base_size_large - 1;
        for i in 0..large_count {
            pool_slots.push(
                FixedSizeBufferPool::new(
                    block_size,
                    base_size_large * (i + 1),
                    wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM
                )
            );
        }

        let asset_mgr = AssetMgr::<AssetRWBuffer>::new(GarbageEmpty(), false, 16 * 1024 * 1024, 60 * 1000);

        Self {
            base_size,
            base_size_large,
            block_size,
            pool_slots,
            small_count,
            asset_mgr
        }
    }
    pub fn allocate(&mut self, size: wgpu::DynamicOffset) -> Option<BindBufferRange> {
        let slot_index = if size <= self.base_size_large {
            let mut slot_index = size / self.base_size;
            if slot_index * self.base_size == size {
                slot_index -= 1;
            }
            slot_index
        } else if size <= self.block_size {
            let mut slot_index = size / self.base_size_large;
            if slot_index * self.base_size == size {
                slot_index -= 1;
            }

            slot_index += self.small_count;
            slot_index
        } else {
            return None;
        };
        
        if let Some(pool) = self.pool_slots.get_mut(slot_index as usize) {
            if let Some(range) = pool.allocate(&self.asset_mgr) {
                Some(BindBufferRange(Arc::new(range)))
            } else {
                None
            }
        } else {
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BindBufferRange(pub Arc<RWBufferRange>);
impl std::ops::Deref for BindBufferRange {
    type Target = Arc<RWBufferRange>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
