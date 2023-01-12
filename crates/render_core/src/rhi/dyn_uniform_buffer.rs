use std::{
    fmt::Debug,
    mem::transmute,
    ops::{Deref, Range},
};

use bitvec::vec::BitVec;
use derive_deref_rs::Deref;
use pi_map::vecmap::VecMap;
use pi_share::{Share, ShareMutex};
use wgpu::{util::BufferInitDescriptor, BufferUsages};

use super::{
    bind_group::BindGroup,
    bind_group_layout::BindGroupLayout,
    buffer::Buffer,
    device::RenderDevice,
    shader::{BindLayout, BufferSize, Uniform, WriteBuffer},
    RenderQueue,
};

/// 用于管理bingdgroup中的buffer
/// 这些bindgroup需要满足如下要求：
/// * binggroup中的所有binding都是buffer类型
/// * 该group中的所有binding的buffer都通过本管理器分配
pub struct DynGroupBuffersMgr {
    buffers: Vec<DynGroupBuffers>,
    alignment: u32,
    limit_size: u32,
}

impl DynGroupBuffersMgr {
    /// 创建DynGroupBuffersMgr
    pub fn new(alignment: u32, limit_size: u32) -> Self {
        Self {
            buffers: Vec::with_capacity(1),
            alignment,
            limit_size,
        }
    }
    /// 为一类bindgroup添加buffer分配
    /// 相同布局的bindgroup可以多次添加，这取决于外部需要将该布局的bindgroup如何分类
    /// # example
    /// 通常按更新频率将bindgroup分类，可以将某类型的bindgroup分为两类：可变Buffer、不可变Buffer。
    /// 是的大部分情况下，不可变buffer无须更新到显卡， 可变buffer更新时，有不至于更新的数据量过大
    /// ```
    /// let mut buffer_mgr = DynGroupBuffersMgr::new(16, 256);
    /// // 同一种bindgroup， 可以添加不可变的buffer分配器和可变的buffer分配器，甚至更多类型的分配器
    /// let imut_index = buffer_mgr.add_for_bind_group(None, ...);
    /// let mut_index = buffer_mgr.add_for_bind_group(None, ...);
    /// ```
    pub fn add_for_bind_group(
        &mut self,
        label: Option<String>,
        entrys: Vec<wgpu::BindGroupLayoutEntry>,
    ) -> Result<GroupBufferIndex, String> {
        let buffer = DynGroupBuffers::new(label, self.alignment, self.limit_size, entrys)?;
        self.buffers.push(buffer);
        Ok(GroupBufferIndex(self.buffers.len() - 1))
    }

    /// 将buffer更新到wgpu，将扩容了buffer的bindgroup重新创建
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        for b in self.buffers.iter_mut() {
            b.write_buffer(device, queue);
        }
    }

    /// 取到某种类型的buffer分配器
    pub fn get(&self, index: GroupBufferIndex) -> Option<DynGroupBuffersEntry> {
        if index.0 >= self.buffers.len() {
            return None;
        }
        Some(self.index(index))
    }
    /// 取到某种类型的buffer分配器
    /// 如果不存在，将panic
    pub fn index(&self, index: GroupBufferIndex) -> DynGroupBuffersEntry {
        DynGroupBuffersEntry {
            buffer: &self.buffers[index.0],
            index,
        }
    }
}

/// 某种bindgrop的buffer分配器的索引
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct GroupBufferIndex(usize);

/// buffer分配器 + 其在DynGroupBuffersMgr中的索引
pub struct DynGroupBuffersEntry<'a> {
    buffer: &'a DynGroupBuffers,
    index: GroupBufferIndex,
}

impl<'a> DynGroupBuffersEntry<'a> {
    /// 分配buffer
    #[inline]
    pub fn alloc(&self) -> DynBufferIndex {
        let mut r = self.buffer.alloc();
        r.group_index = self.index.clone();
        r
    }
}

impl<'a> Deref for DynGroupBuffersEntry<'a> {
    type Target = DynGroupBuffers;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.buffer
    }
}

/// 用于管理一个bindgroup分配的buffer
pub struct DynGroupBuffers {
    buffers: Share<ShareMutex<Buffers>>,
    info: DynGroupBufferInfo,
}

/// 可变buffer的索引
pub struct DynBufferIndex {
    pub buffer_offset: Vec<u32>,
    buffer_index: usize,
    buffers: Share<ShareMutex<Buffers>>,
    group_index: GroupBufferIndex,
}

impl Debug for DynBufferIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynBufferIndex")
            .field("buffer_offset", &self.buffer_offset)
            .field("buffer_index", &self.buffer_index)
            .field("group_index", &self.group_index)
            .finish()
    }
}

impl DynGroupBuffers {
    pub fn new(
        label: Option<String>,
        alignment: u32,
        limit_size: u32,
        entrys: Vec<wgpu::BindGroupLayoutEntry>,
    ) -> Result<Self, String> {
        let mut binding_offset_map = VecMap::with_capacity(entrys.len());
        let mut buffer0 = Vec::with_capacity(1);
        let mut binding_size_list = Vec::with_capacity(entrys.len());
        let (mut max_size_binding_index, mut max_size) = (0, 0);
        for (i, entry) in entrys.iter().enumerate() {
            if let wgpu::BindingType::Buffer {
                min_binding_size, ..
            } = &entry.ty
            {
                match min_binding_size {
                    Some(r) => {
                        let size = (entry.count.map_or(1, |r| r.get()) * r.get() as u32) as usize;
                        buffer0.push(DynBindingBuffer::new(label.clone(), alignment, size));
                        if size > max_size {
                            max_size = size;
                            max_size_binding_index = i;
                        }
                        binding_size_list.push(size);
                        binding_offset_map.insert(entry.binding as usize, i);
                        // 是否需要对齐？TODO
                        // block_size += r.get() as u32 * count;
                    }
                    None => {
                        return Err(
                            "DynUniformBuffers init fail, min_binding_size is none ".to_string()
                        )
                    }
                }
            } else {
                return Err("".to_string());
            }
        }
        let (layout_label, group_label) = match &label {
            Some(r) => (Some(r.clone() + " layout"), Some(r.clone() + " group")),
            None => (None, None),
        };

        Ok(DynGroupBuffers {
            buffers: Share::new(ShareMutex::new(Buffers {
                values: vec![buffer0],
                lately_use_buffer: 0,
            })),
            info: DynGroupBufferInfo {
                entrys,
                layout: None,
                bind_groups: VecMap::new(),
                binding_offset_map,
                // block_size: block_size,
                max_size_binding_index,
                limit_size,
                alignment,
                binding_size_list,
                label,
                layout_label,
                group_label,
            },
        })
    }

    #[inline]
    pub fn alloc(&self) -> DynBufferIndex {
        // 寻找一个可以分配的buffer

        fn find_or_create_buffers<'a, 'b>(
            buffers: &'a mut Vec<Vec<DynBindingBuffer>>,
            info: &'b DynGroupBufferInfo,
        ) -> (usize, &'a mut Vec<DynBindingBuffer>) {
            // 绕过生命周期误报，应该是编译器bug，这里先绕过
            let buff11: &'a mut Vec<Vec<DynBindingBuffer>> =
                unsafe { transmute(&mut *buffers as *mut Vec<Vec<DynBindingBuffer>>) };

            // 找到一个存在空闲位置的buffer组
            for (index, buffer) in buff11.iter_mut().enumerate() {
                let buffer1 = &mut buffer[info.max_size_binding_index];

                if buffer1.len() + buffer1.block_size() < info.limit_size as usize {
                    return (index, buffer);
                }
            }

            // 如果未找到，则创建新的
            let mut buffer = Vec::with_capacity(info.binding_size_list.len());
            for i in info.binding_size_list.iter() {
                buffer.push(DynBindingBuffer::new(
                    info.label.clone(),
                    info.alignment,
                    *i,
                ));
            }
            buffers.push(buffer);
            let last_index = buffers.len() - 1;
            (last_index, &mut buffers[last_index])
        }

        let mut buffer_lock = self.buffers.lock();
        let buffer_lock = &mut *buffer_lock;
        let info = &self.info;

        // 如果最近分配过的buffer已经超出内存限制，则创建或找到一个有剩余空间的buffer
        let mut buffers = &mut buffer_lock.values[buffer_lock.lately_use_buffer];
        if !buffers[info.max_size_binding_index].check_limit(info.limit_size) {
            let r = find_or_create_buffers(&mut buffer_lock.values, info);
            buffer_lock.lately_use_buffer = r.0;
            buffers = r.1;
        }

        // 分配
        let mut offsets = Vec::new();
        for buffer in buffers.iter_mut() {
            offsets.push(buffer.alloc());
        }

        // 返回分配的索引
        DynBufferIndex {
            buffer_offset: offsets,
            buffer_index: buffer_lock.lately_use_buffer,
            buffers: self.buffers.clone(),
            group_index: GroupBufferIndex(0),
        }
    }

    /// 设置uniform
    #[inline]
    pub fn set_uniform<T: Uniform>(
        &mut self,
        binding_offset: &DynBufferIndex,
        t: &T,
    ) -> Result<(), String> {
        self.update_buffer(binding_offset, t, T::Binding::binding())
    }

    /// 更新buffer
    pub fn update_buffer<T: WriteBuffer>(
        &self,
        binding_offset: &DynBufferIndex,
        t: &T,
        binding: u32,
    ) -> Result<(), String> {
        // 指针不等，无法更新
        if !Share::ptr_eq(&binding_offset.buffers, &self.buffers) {
            return Err("".to_string());
        }
        let offset = match self.info.binding_offset_map.get(binding as usize) {
            Some(r) => *r,
            None => return Err("".to_string()),
        };
        let mut buffer_lock = self.buffers.lock();
        buffer_lock.values[binding_offset.buffer_index][offset]
            .set_uniform::<T>(binding_offset.buffer_offset[offset], t);
        Ok(())
    }

    /// 写入buffer到现存，返回是否重新创建了buffer
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        let mut buffer_lock = self.buffers.lock();
        if let None = self.info.layout {
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: match &self.info.layout_label {
                    Some(r) => Some(r.as_str()),
                    None => None,
                },
                entries: self.info.entrys.as_slice(),
            });
            self.info.layout = Some(layout);
        }
        let layout = self.info.layout.as_ref().unwrap();
        for (i, buffers) in buffer_lock.values.iter_mut().enumerate() {
            let mut buffer_is_expand = false;
            for buffer in buffers.iter_mut() {
                buffer_is_expand = buffer.write_buffer(device, queue) || buffer_is_expand;
            }

            // 如果有buffer扩容，则重新创建bindgroup
            if buffer_is_expand {
                let mut entries = Vec::new();
                for (i, entry) in self.info.entrys.iter().enumerate() {
                    let buffer = buffers[i].buffer().unwrap();
                    entries.push(wgpu::BindGroupEntry {
                        binding: entry.binding,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer,
                            offset: 0,
                            size: std::num::NonZeroU64::new(self.info.binding_size_list[i] as u64),
                        }),
                    })
                }
                self.info.bind_groups.insert(
                    i,
                    device.create_bind_group(&wgpu::BindGroupDescriptor {
                        layout,
                        entries: entries.as_slice(),
                        label: match &self.info.group_label {
                            Some(r) => Some(r.as_str()),
                            None => None,
                        },
                    }),
                );
            }
        }
    }
}

pub struct DynBindingBuffer {
    cache_buffer: DynBuffer,
    buffer: Option<Buffer>,
    old_len: usize,
    block_size: usize,
    label: Option<String>,
}

impl DynBindingBuffer {
    pub fn new(label: Option<String>, alignment: u32, block_size: usize) -> Self {
        Self {
            cache_buffer: DynBuffer::new(alignment),
            buffer: None,
            old_len: 0,
            block_size,
            label,
        }
    }

    #[inline]
    pub fn buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    #[inline]
    pub fn alloc(&mut self) -> u32 {
        self.cache_buffer.alloc_with_size(self.block_size)
    }

    // #[inline]
    // pub fn alloc_with_size(&mut self, bind: usize) -> u32 {
    //     self.cache_buffer.alloc_with_size(bind)
    // }

    #[inline]
    pub fn set_uniform<T: WriteBuffer>(&mut self, binding_offset: u32, t: &T) {
        self.cache_buffer.full::<T>(binding_offset, t);
    }

    /// 写入buffer到现存，返回是否重新创建了buffer
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) -> bool {
        // 什么也没改变， 直接返回
        if self.cache_buffer.change_range.len() == 0 {
            return false;
        }

        let size = self.cache_buffer.buffer().len();

        if self.old_len < size {
            self.buffer = Some(device.create_buffer_with_data(&BufferInitDescriptor {
                label: self.label.as_deref(),
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                contents: self.cache_buffer.buffer(),
            }));
            self.old_len = size;
            self.cache_buffer.change_range = 0..0;
            return true;
        } else if let Some(buffer) = &self.buffer {
            queue.write_buffer(
                buffer,
                self.cache_buffer.change_range.start as u64,
                &self.cache_buffer.buffer()[self.cache_buffer.change_range.start as usize
                    ..self.cache_buffer.change_range.end as usize],
            );
            self.cache_buffer.change_range = 0..0;
        }
        false
    }

    pub fn capacity(&self) -> usize {
        self.old_len
    }

    // 当前buffer的长度
    pub fn len(&self) -> usize {
        self.cache_buffer.tail
    }

    pub fn block_size(&self) -> usize {
        self.block_size
    }

    fn check_limit(&self, limit: u32) -> bool {
        !(self.block_size + self.len() > limit as usize)
    }
}

struct Buffers {
    values: Vec<Vec<DynBindingBuffer>>,
    // 最近使用的buffer索引（在buffers字段中的索引）
    lately_use_buffer: usize,
}

struct DynGroupBufferInfo {
    entrys: Vec<wgpu::BindGroupLayoutEntry>,
    bind_groups: VecMap<BindGroup>,
    layout: Option<BindGroupLayout>,
    // 每个元素表示一组buffer, 一组buffer对应一个bindgroup
    // 占用内存最大的bingding的索引（以此索引中的buffer的内存来判断是否超过内存限制）
    max_size_binding_index: usize,
    // entry中描述的多个bingding将作为整体，分配一个块， 每个bingding在该块中，会对应一个偏移，该偏移存储在binding_offset_map中（VecMap的key为entry的binding值）
    binding_offset_map: VecMap<usize>,
    binding_size_list: Vec<usize>,
    alignment: u32,
    // 每个buffer的限制长度
    limit_size: u32,
    label: Option<String>,
    layout_label: Option<String>,
    group_label: Option<String>,
}

#[derive(Debug)]
pub struct BindOffset {
    offset: u32,
    context: Share<ShareMutex<DynBuffer>>,
}

impl std::ops::Deref for BindOffset {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.offset
    }
}

impl Drop for BindOffset {
    fn drop(&mut self) {
        self.context.lock().remove(self.offset);
    }
}

#[derive(Clone, Copy, Deref)]
pub struct BindIndex(usize);

impl BindIndex {
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

pub type GroupId = u32;

pub type BindId = u32;

pub trait AsBind {
    fn min_size(&self) -> usize;

    // 在bindings数组中的位置
    fn index(&self) -> BindIndex;
}

pub trait Group {
    fn id() -> GroupId;
    fn create_layout(device: &RenderDevice, has_dynamic_offset: bool) -> BindGroupLayout;
}

pub trait BufferGroup {
    fn create_bind_group(
        device: &RenderDevice,
        layout: &BindGroupLayout,
        buffer: &Buffer,
    ) -> BindGroup;
}

#[derive(Debug)]
pub struct DynBuffer {
    tail: usize,
    buffer: Vec<u8>,
    alignment: usize,

    full_bits: BitVec, // 填充位标识
    end_bits: BitVec,  // 结束索引

    change_range: Range<u32>, // 修改范围
}

impl DynBuffer {
    #[inline]
    pub fn buffer(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    #[inline]
    pub fn new(alignment: u32) -> Self {
        Self {
            buffer: Vec::default(),
            alignment: alignment as usize,
            full_bits: BitVec::default(),
            end_bits: BitVec::default(),
            tail: 0,
            change_range: 0..0,
        }
    }

    pub fn alloc<T: BufferSize>(&mut self) -> u32 {
        self.alloc_with_size((T::min_size() as f32 / self.alignment as f32).ceil() as usize)
    }

    pub fn alloc_with_size(&mut self, bind: usize) -> u32 {
        let count = (bind as f32 / self.alignment as f32).ceil() as usize;

        let mut iter = self.full_bits.iter_zeros();
        let mut item = iter.next();

        let len = self.full_bits.len();
        loop {
            let i = match item {
                Some(i) => i,
                None => self.full_bits.len(),
            };

            let end = i + count;
            let mut cur_end = end.min(len);

            if self.full_bits[i..cur_end].not_any() {
                self.full_bits[i..cur_end].fill(true);
                while cur_end < end {
                    self.full_bits.push(true);
                    cur_end += 1;
                }

                // 设置结束标识
                let l = self.end_bits.len();
                if l < end {
                    self.end_bits.reserve(end - l);
                    unsafe { self.end_bits.set_len(end) };
                    self.end_bits[l..end - 1].fill(false);
                }
                self.end_bits.set(end - 1, true);

                self.tail = self.tail.max((i + count) * self.alignment);
                return (i * self.alignment) as u32;
            }
            item = iter.next();
        }
    }

    /// 移除buffer分配
    pub fn remove(&mut self, mut i: u32) {
        i = i / self.alignment as u32;
        let len = self.end_bits.len();
        while (i as usize) < len {
            self.full_bits.set(i as usize, false);
            i = i + 1;
            if unsafe { self.end_bits.replace_unchecked((i - 1) as usize, false) } {
                return;
            }
        }
    }

    pub fn full<T: WriteBuffer>(&mut self, index: u32, t: &T) {
        if self.buffer.len() < self.tail {
            self.buffer.reserve(self.tail - self.buffer.len());
            unsafe { self.buffer.set_len(self.tail) }
        }

        t.write_into(index, self.buffer.as_mut_slice());

        // 设置数据变化范围
        let end = index + t.byte_len();
        if self.change_range.len() == 0 {
            self.change_range.start = index;
            self.change_range.end = end;
            return;
        }
        if self.change_range.start > index {
            self.change_range.start = index;
        }
        if self.change_range.end < end {
            self.change_range.end = end;
        }
    }
}
