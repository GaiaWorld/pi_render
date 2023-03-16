use std::{sync::Arc, hash::Hash, fmt::Debug};

use pi_assets::{asset::{Asset, Handle}, mgr::AssetMgr};
use pi_share::{Share, ShareMutex};

use crate::rhi::{dyn_uniform_buffer::SingleBufferAlloter, shader::WriteBuffer, device::RenderDevice, RenderQueue, buffer::Buffer};

use crate::asset::bytes_write_to_memory;

pub struct AssetRWBuffer(SingleBufferAlloter, u32);
impl Asset for AssetRWBuffer {
    type Key = IDRWBuffer;
    fn size(&self) -> usize {
        self.1 as usize
    }
}
impl AssetRWBuffer {
    fn write_buffer(&self, device: &RenderDevice, queue: &RenderQueue,  info: &Option<String>) -> bool {
        unsafe {
            let temp = &mut *(self as *const Self as usize as *mut Self);
            temp.0.write_to_buffer(device, queue, info)
        }
    }
    fn alloc(&self) -> Option<usize> {
        unsafe {
            let temp = &mut *(self as *const Self as usize as *mut Self);
            temp.0.alloc()
        }
    }
    fn fill(&self, offset: u32, local_offset: usize, data: &[u8]) {
        unsafe {
            let temp = &mut *(self as *const Self as usize as *mut Self);
            let val = TempWriteBuffer(local_offset, data);
            temp.0.fill(offset, &val);
        }
    }
    fn free(&self, index: usize) {
        unsafe {
            let temp = &mut *(self as *const Self as usize as *mut Self);
            temp.0.free(index);
        }
    }
}

struct UseAssetRWBuffer(Option<Handle<AssetRWBuffer>>);

pub(crate) struct FixedSizeBufferPool {
    /// * 大内存块列表 (第i个的尺寸为 i*block_size)
    buffers: Vec<UseAssetRWBuffer>,
    /// * 大内存块的基础尺寸
    block_size: u32,
    /// * 目标区间尺寸
    pub(crate) fixed_size: u32,
    mutex: ShareMutex<()>,
    usage: wgpu::BufferUsages,
}
impl FixedSizeBufferPool {
    /// * `block_size` 大内存块的基础尺寸
    /// * `fixed_size` 目标区间尺寸
    pub(crate) fn new(
        mut block_size: u32,
        fixed_size: u32,
        usage: wgpu::BufferUsages,
    ) -> Self {
        if block_size < fixed_size {
            block_size = fixed_size;
        }
        Self {
            buffers: vec![],
            fixed_size,
            block_size,
            mutex: ShareMutex::new(()),
            usage
        }
    }
    pub(crate) fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.buffers.iter_mut().for_each(|item| {
            // log::info!("write_buffer: >>>>>>>>>> A {:?}", self.fixed_size);
            if let Some(asset_buffer) = &item.0 {
                // log::info!("write_buffer: >>>>>>>>>> B");
                if asset_buffer.write_buffer(device, queue, &None) == false {
                    // log::info!("write_buffer: {:?}", buffer.0.is_using());
                    if !asset_buffer.0.is_using() {
                        item.0 = None;
                    }
                }
            }
        })
    }
    pub(crate) fn allocate(&mut self, asset_mgr: &Share<AssetMgr<AssetRWBuffer>>) -> Option<RWBufferRange> {
        let len = self.buffers.len();
        let mut key_buffer = None;
        // 寻找可用区间
        for i in 0..len {
            if let Some(use_buffer) = self.buffers.get(i) {
                if let Some(asset_buffer) = &use_buffer.0 {
                    let _clock = self.mutex.lock();
                    if let Some(index) = asset_buffer.alloc() {
                        return Some(
                            RWBufferRange { index, id_buffer: IDRWBuffer { fixed_size: self.fixed_size, index: i as u32 }, buffer: asset_buffer.clone() }
                        );
                    }
                } else {
                    key_buffer = Some(IDRWBuffer { fixed_size: self.fixed_size, index: i as u32  });
                }
            }
        }

        // 寻找 是否有缓存 块
        let key_buffer = if let Some(key_buffer) = key_buffer {
            if let Some(asset_buffer) = asset_mgr.get(&key_buffer) {
                let use_buffer = UseAssetRWBuffer(Some(asset_buffer.clone()));
                self.buffers[key_buffer.index as usize] = use_buffer;

                if let Some(index) = asset_buffer.alloc() {
                    return Some(
                        RWBufferRange { index, id_buffer: key_buffer.clone(), buffer: asset_buffer.clone() }
                    );
                } else {
                    return None;
                }
            } else {
                key_buffer
            }
        } else {
            self.buffers.push(UseAssetRWBuffer(None));
            IDRWBuffer { fixed_size: self.fixed_size, index: len as u32  }
        };

        // 创建块
        let mut buffer = SingleBufferAlloter::new(
            (self.block_size * (key_buffer.index + 1) / self.fixed_size) as usize,
            self.fixed_size as u32,
            self.usage
        );
        if let Some(index) = buffer.alloc() {
            if let Some(asset_buffer) = asset_mgr.insert(key_buffer, AssetRWBuffer(buffer, self.block_size)) {
                let use_buffer = UseAssetRWBuffer(Some(asset_buffer.clone()));
                self.buffers[key_buffer.index as usize] = use_buffer;
                return Some(
                    RWBufferRange { index, id_buffer: key_buffer.clone(), buffer: asset_buffer.clone() }
                );
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}

pub(crate) struct TempWriteBuffer<'a>(usize, &'a [u8]);
impl<'a> WriteBuffer for TempWriteBuffer<'a> {
    fn write_into(&self, index: u32, buffer: &mut [u8]) {
        bytes_write_to_memory(self.1, (index + self.offset()) as usize, buffer);
    }

    fn byte_len(&self) -> u32 {
        self.1.len() as u32
    }

    fn offset(&self) -> u32 {
        self.0 as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IDRWBuffer {
    /// * Buffer 的固化大小 - (内存区间的对齐大小)
    pub fixed_size: u32,
    /// * 在该固化大小的BufferPool 中的序号
    pub index: u32,
}

/// * Hash, PartialEq, Eq 只判断 RWBufferRange 使用的 IDBindBuffer 是否相同 (也标识 Buffer 是否相同), 不判断数据是否相同
/// * 判断数据 是否相同需要 判断 offset, IDBindBuffer 是否都相同
#[derive(Clone)]
pub struct RWBufferRange {
    index: usize,
    id_buffer: IDRWBuffer,
    buffer: Handle<AssetRWBuffer>,
}
impl Debug for RWBufferRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BindBufferRange").field("index", &self.index).field("id_buffer", &self.id_buffer).finish()
    }
}
impl RWBufferRange {
    pub fn write_data(&self, local_offset: usize, data: &[u8]) {
        let offset = self.index as u32 * self.id_buffer.fixed_size;
        self.buffer.fill(offset as u32, local_offset, data);
    }
    pub fn buffer(&self) -> &Buffer {
        self.buffer.0.wgpu_buffer().unwrap()
    }
    pub fn size(&self) -> u32 {
        self.id_buffer.fixed_size
    }
    pub fn offset(&self) -> wgpu::DynamicOffset {
        (self.index as u32 * self.id_buffer.fixed_size) as wgpu::DynamicOffset
    }
    pub fn id_buffer(&self) -> IDRWBuffer {
        self.id_buffer
    }
}
impl Drop for RWBufferRange {
    fn drop(&mut self) {
        self.buffer.free(self.index);
    }
}
impl Hash for RWBufferRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id_buffer.hash(state);
    }
}
impl PartialEq for RWBufferRange {
    fn eq(&self, other: &Self) -> bool {
        self.id_buffer == other.id_buffer
    }
}
impl Eq for RWBufferRange {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}
