use std::{ops::Range, hash::Hash};

use pi_share::{Share, ShareMutex};
use render_core::rhi::{device::RenderDevice, buffer::Buffer, RenderQueue, BufferInitDescriptor};

use crate::{base::{TMemoryAllocatorLimit, MemoryRange, bytes_write_to_memory}};

use super::{EErrorBuffer, TBufferLimit};

/// * 与 DynMergyBufferAllocator 绑定
pub type KeyDynBuffer = usize;

/// * 动态合并分配器(Buffer&内存)
/// * 首先分配一个较大的块, 在块内部 分配 指定需求大小的区间进行使用, 区间可回收, 并可以被合并到临近的未使用区间
/// * 没有任何块可以分配 指定需求大小的区间 时, 新分配一个块, 再进行区间分配
/// * 分配块时 检测是否超出 总大小限制, 超出时报错
pub struct DynMergyBufferAllocator {
    blocks: Vec<Share<DynMergyBufferBlock>>,
    size: usize,
    max_size: u64,
    block_size: usize,
}
impl DynMergyBufferAllocator {
    /// * 创建分配器
    /// * limit 指定 总大小限制 - example: 1 * 1024 * 1024 * 1024
    /// * block_size 指定 分配的块的大小 - example: 16 * 1024 * 1024
    pub fn new<T: TMemoryAllocatorLimit>(limit: &T, block_size: usize) -> Self {
        Self {
            blocks: vec![],
            size: 0,
            max_size: limit.max_size(),
            block_size,
        }
    }

    /// * 分配Buffer区间
    /// * size 指定 需求的尺寸 - example: bind_size
    /// * device 指定 设备
    /// * return
    ///   * EErrorBuffer::AllocatorOverSize 超过总大小限制, 需要使用其他分配器
    ///   * EErrorBuffer::SizeOverBlock 需求尺寸超过了块的尺寸, 需要使用其他分配器 或 使用一个可分配更大块的分配器
    pub fn allocate(&mut self, size: usize, device: &RenderDevice) -> Result<DynMergyBufferRange, EErrorBuffer> {
        let alignment = device.limits().min_uniform_buffer_offset_alignment as usize;

        let mut temp = size / alignment;
        if temp * alignment < size {
            temp = temp + 1;
        }
        let size = temp * alignment;

        let len = self.blocks.len();
        for i in 0..len {
            let item = self.blocks.get(i).unwrap();
            let _lock = item.mutex.lock();
            let block = unsafe { &mut *(Share::as_ptr(item) as usize as *mut DynMergyBufferBlock) };

            let temp = block.allocate(size);

            if let Some(temp) = temp {
                return Ok(DynMergyBufferRange::new(i, temp.start, temp.size, item.clone()));
            }
        }

        let new_size = self.size + self.block_size;
        if (new_size as u64) < self.max_size {
            let mut item = DynMergyBufferBlock::new(len, self.block_size, device);
            let temp = item.allocate(size);
            let item = Share::new(item);
            self.blocks.push(item.clone());
            if let Some(temp) = temp {
                return Ok(DynMergyBufferRange::new(len, temp.start, temp.size, item));
            } else {
                Err(EErrorBuffer::SizeOverBlock)
            }
        } else {
            Err(EErrorBuffer::AllocatorOverSize)
        }
    }

    /// * 更新数据到 Buffer
    /// * 使用 DynMergyBufferRange.buffer 前调用, 确保数据准确
    pub fn write_buffer(&self, queue: &RenderQueue, log_info: bool) {
        self.blocks.iter().for_each(|block| {
            let _lock = block.mutex.lock();
            let block = unsafe { &mut *(Share::as_ptr(block) as usize as *mut DynMergyBufferBlock) };
            block.write_buffer(queue, log_info);
        });
    }
}

#[derive(Debug)]
pub struct DynMergyBufferBlock {
    index: usize,
    buffer: Buffer,
    wait_list: Vec<MemoryRange>,
    data: Vec<u8>,
    mutex: ShareMutex<()>,
    dirty: Vec<MemoryRange>,
}
impl DynMergyBufferBlock {
    /// * 更新数据到 Buffer 时 超过 WRITE_CHECK 长度未更改,则分两次更新
    const WRITE_CHECK: usize = 64 * 1024;
    fn new(index: usize, size: usize, device: &RenderDevice) -> Self {
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(0);
        }
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            index,
            buffer,
            data,
            wait_list: vec![MemoryRange::new(0, size)],
            mutex: ShareMutex::new(()),
            dirty: vec![],
        }
    }

    fn allocate(&mut self, size: usize) -> Option<MemoryRange> {
        // 按区间尺寸排序
        self.wait_list.sort_by(|a, b| a.size.partial_cmp(&b.size).unwrap());

        // 寻找大于目标尺寸的最小区间
        let index = match self.wait_list.binary_search_by(|a| a.size.cmp(&size)) {
            Ok(index) => index,
            Err(index) => index,
        };

        if index >= self.wait_list.len() {
            None
        } else {
            let wait = self.wait_list.get_mut(index).unwrap();
            let useinfo = MemoryRange::new(wait.start, size);
            wait.start  = wait.start + size;
            wait.size   = wait.size  - size;

            Some(useinfo)
        }
    }

    fn de_allocate(&mut self, data: &DynMergyBufferRange) {
        self.wait_list.push(MemoryRange::new(data.start, data.size));
        self.mergy();
    }

    /// * 更新数据到 Buffer
    pub fn write_buffer(&mut self, queue: &RenderQueue, log_info: bool) {
        self.mergy_dirty();

        let len = self.dirty.len();
        if len > 0 {
            if log_info {
                log::info!("DynMergyBufferBlock dirty: {:?}", self.dirty);
            }
            // 按区间位置排序
            // self.dirty.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
            
            let item = self.dirty.get(0).unwrap();
            let mut start = item.start;
            let mut end = item.start + item.size;

            let item2 = self.dirty.get(len - 1).unwrap();
            let max_end = item2.start() + item2.size();
            
            if len > 1 {
                for i in 1..len {
                    let item = self.dirty.get(i).unwrap();
    
                    let new_end = item.start + item.size;
                    // 当前起点与上一个的终点距离超过 WRITE_CHECK
                    // log::info!("{:?}, {:?}", item.start, end);
                    if item.start - end >= Self::WRITE_CHECK {
                        // log::info!("write_buffer : {:?}, {:?}", start, end);
                        queue.write_buffer(&self.buffer, start as wgpu::BufferAddress, &self.data[start..end]);
                        start   = item.start;
                    }
                    end     = new_end;
                }
            }
            
            if end <= max_end {
                // log::info!("write_buffer E: {:?}, {:?}", start, end);
                queue.write_buffer(&self.buffer, start as wgpu::BufferAddress, &self.data[start..end]);
            }
        }
        self.dirty.clear();
    }

    fn mergy(&mut self) {
        // 按区间位置排序
        self.wait_list.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
        let mut result = vec![];
        let mut start = 0;
        let mut size = 0;
        self.wait_list.iter().for_each(|item| {
            if start + size == item.start {
                size += item.size;
            } else {
                result.push(MemoryRange::new(start, size));
                start = item.start;
                size = item.size;
            }
        });

        self.wait_list = result;
    }

    fn mergy_dirty(&mut self) {
        self.dirty.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
        // 按区间位置排序
        let mut result = vec![];
        let mut start = usize::MAX;
        let mut size = 0;
        self.dirty.iter().for_each(|item| {
            if start != item.start {
                result.push(item.clone());
                start = item.start;
            }
        });

        self.dirty = result;
    }
}
impl Hash for DynMergyBufferBlock {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.buffer.id().hash(state);
    }
}
impl PartialEq for DynMergyBufferBlock {
    fn eq(&self, other: &Self) -> bool {
        self.buffer.id() == other.buffer.id()
    }
}
impl Eq for DynMergyBufferBlock {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}

/// * Hash, PartialEq, Eq 只判断 DynMergyBufferRange 使用的 DynMergyBufferBlock 和 size 是否相同 (也标识 Buffer 是否相同), 不判断数据是否相同
/// * 判断数据 是否相同需要 判断 start, size, id_block 是否都相同
#[derive(Debug, Clone)]
pub struct DynMergyBufferRange {
    id_block: usize,
    start: usize,
    size: usize,
    context: Share<DynMergyBufferBlock>,
}
impl DynMergyBufferRange {
    pub fn new(id_block: usize, start: usize, size: usize, context: Share<DynMergyBufferBlock>) -> Self {
        Self {
            id_block,
            start,
            size,
            context
        }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn range(&self) -> Range<usize> {
        Range { start: self.start, end: self.start + self.size }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.context.buffer
    }

    pub fn id_buffer(&self) -> KeyDynBuffer {
        self.id_block
    }

    /// * 更新数据, 此时只更新到 内存
    pub fn write_data(&self, local_offset: usize, data: &[u8]) {
        if local_offset + data.len() <= self.size {
            let _lock = self.context.mutex.lock();
            let block = unsafe { &mut *(Share::as_ptr(&self.context) as usize as *mut DynMergyBufferBlock) };
            bytes_write_to_memory(data, self.start + local_offset, &mut block.data);
            block.dirty.push(
                MemoryRange::new(
                    self.start, 
                    self.size
                )
            );
        } else {
            log::error!("data size error !");
        }
    }
}
impl Drop for DynMergyBufferRange {
    fn drop(&mut self) {
        let _lock = self.context.mutex.lock();
        let block = unsafe { &mut *(Share::as_ptr(&self.context) as usize as *mut DynMergyBufferBlock) };
        block.de_allocate(self);
    }
}
impl Hash for DynMergyBufferRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.context.hash(state);
        self.size.hash(state);
    }
}
impl PartialEq for DynMergyBufferRange {
    fn eq(&self, other: &Self) -> bool {
        self.context == other.context && self.size == other.size
    }
}
impl Eq for DynMergyBufferRange {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}

#[cfg(test)]
mod test_dyn_mergy_buffer {
    use crate::base::TMemoryAllocatorLimit;

    use super::DynMergyBufferAllocator;

    pub struct BufferLimit;
    impl TMemoryAllocatorLimit for BufferLimit {
        fn max_size(&self) -> u64 {
            1 * 1024 * 1024 * 1024
        }
    }
    #[test]
    fn test() {
        let mut allocator = DynMergyBufferAllocator::new(&BufferLimit, 1 * 1024 * 1024);

        // allocator.allocate(256, device)
    }
}
