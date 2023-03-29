use std::{fmt::Debug, ops::Range, sync::atomic::{AtomicU32, Ordering}};

use bitvec::vec::BitVec;
use derive_deref_rs::Deref;
use pi_map::vecmap::VecMap;
use pi_share::{Share, ShareMutex};
use smallvec::{smallvec, SmallVec};
use thiserror::Error;
use wgpu::{util::BufferInitDescriptor, BufferUsages};

use crate::renderer::draw_obj::DrawBindGroup;

use super::{
    bind_group::BindGroup,
    bind_group_layout::BindGroupLayout,
    buffer::Buffer,
    device::RenderDevice,
    shader::{BindLayout, Uniform, WriteBuffer},
    RenderQueue,
};
/// 用于管理一类bindgroup分配的buffer
/// 这些bindgroup需要满足如下要求：
/// * binggroup中的所有binding都是buffer类型
/// * 该group中的所有binding的buffer都通过本管理器分配
pub struct GroupAlloter {
    buffers: Share<GroupBuffer>,
    info: DynGroupBufferInfo,
}

impl GroupAlloter {
    /// 相同布局的bindgroup可以多次添加，这取决于外部需要将该布局的bindgroup如何分类
    /// # example
    /// 通常按更新频率将bindgroup分类，可以将某类型的bindgroup分为两类：可变Buffer、不可变Buffer。
    /// 大部分情况下，不可变buffer无须更新到wgpu， 可变buffer更新时，又不至于更新的数据量过大
    /// ```
    /// // 同一种bindgroup， 可以添加不可变的buffer分配器和可变的buffer分配器，甚至更多类型的分配器
    /// let imut_index = GroupBufferMgr::new(...);
    /// let mut_index = GroupBufferMgr::new(...);
    /// ```
    pub fn new(
        label: Option<String>,
        min_alignment: u32,
        limit_size: u32,
        init_size: Option<u32>, // 单位： 字节
        entrys: Vec<wgpu::BindGroupLayoutEntry>,
		layout: Share<BindGroupLayout>,
    ) -> Result<Self, DynBufferError> {
        let mut binding_offset_map = VecMap::with_capacity(entrys.len());
        // let mut buffer0 = Vec::with_capacity(1);
        let mut binding_size_list = Vec::with_capacity(entrys.len());
        let mut max_size = 0;
        for (i, entry) in entrys.iter().enumerate() {
            if let wgpu::BindingType::Buffer {
                min_binding_size, ..
            } = &entry.ty
            {
                match min_binding_size {
                    Some(r) => {
                        let mut size = entry.count.map_or(1, |r| r.get()) * r.get() as u32;
                        // 对齐
                        let remain = size % min_alignment;
                        if remain > 0 {
                            size += min_alignment - remain;
                        }

                        if size as usize > max_size {
                            max_size = size as usize;
                        }
                        binding_size_list.push(size as u32);
                        binding_offset_map.insert(entry.binding as usize, i);
                        // 是否需要对齐？TODO
                        // block_size += r.get() as u32 * count;
                    }
                    None => return Err(DynBufferError::MissMinSize),
                }
            } else {
                return Err(DynBufferError::NotNuffer(format!("{:?}", &entry.ty)));
            }
        }
        let group_label = match &label {
            Some(r) => Some(r.clone() + " group"),
            None => None,
        };
        let limit_count = (limit_size as usize) / max_size;
        let init_size = match init_size {
            Some(init_size) => init_size as usize / max_size,
            None => 6400 / max_size,
        };
        let buffer_maps = GroupBuffersAlloter::new(init_size, binding_size_list.as_slice());

        Ok(GroupAlloter {
            buffers: Share::new(GroupBuffer {
                values: smallvec![buffer_maps],
                lately_use_buffer: 0,
                binding_offset_map,
                mutex: ShareMutex::new(()),
            }),
            info: DynGroupBufferInfo {
                entrys,
                layout,
                limit_count,
                binding_size_list,
                label,
                group_label,
            },
        })
    }

    /// 为bindgroup分配索引
    pub fn alloc(&self) -> BufferGroup {
        // 寻找一个可以分配的buffer
        fn alloc(context: &GroupAlloter) -> (Index, usize) {

            // 如果最近分配过的buffer能继续分配，则直接返回分配结果
            if let Some(r) = context.buffers.values[context.buffers.lately_use_buffer].alloc() {
                return (r, context.buffers.lately_use_buffer);
            }

			// 否则，解锁，找到一个有空位的buffer分配，会创建新的buffer，使用新的buffer分配
			// SAFE: 这里转为可变，立即解锁，保证alloc在多线程下不冲突
			let buffers =
			unsafe { &mut *(Share::as_ptr(&context.buffers) as usize as *mut GroupBuffer) };
			let _lock = buffers.mutex.lock();

			let info = &context.info;

            // 找到一个存在空闲位置的buffer组
            for (index, buffer) in buffers.values.iter_mut().enumerate() {
                if let Some(r) = buffer.alloc() {
                    buffers.lately_use_buffer = index;
                    return (r, index);
                }
            }

			// 如果未找到，则创建新的
            let next_count = info
                .limit_count
                .min(buffers.values.last().unwrap().capacity() as usize * 2);
            buffers.lately_use_buffer = context.buffers.values.len();
            let buffer_maps = GroupBuffersAlloter::new(next_count, info.binding_size_list.as_slice());
            let alloc_index = buffer_maps.alloc().unwrap();
            buffers.values.push(buffer_maps);

            (alloc_index, context.buffers.lately_use_buffer)
        }
        let (alloc_index, lately_use_buffer) = alloc(self);

        // 返回分配的索引
        BufferGroup {
            index: alloc_index,
            bindings_index: lately_use_buffer,
            context: self.buffers.clone(),
            group_offsets: self.buffers.values[lately_use_buffer].group_offsets.clone(),
        }
    }

    /// 写入buffer到现存，返回是否重新创建了buffer
    pub fn write_buffer(&self, device: &RenderDevice, queue: &RenderQueue) {
        // SAFE
		let this =
            unsafe { &mut *(self as *const Self as usize as *mut Self) };
        let buffer_lock =
            unsafe { &mut *(Share::as_ptr(&self.buffers) as usize as *mut GroupBuffer) };
        let _lock = self.buffers.mutex.lock();

        let buffer_lock = &mut *buffer_lock;
        for buffers in buffer_lock.values.iter_mut() {
            buffers.write_buffer(device, queue, &this.info, &this.info.layout);
        }
    }
}

/// group索引
pub struct BufferGroup {
    index: Index,
    bindings_index: usize,
    context: Share<GroupBuffer>,
    group_offsets: Share<GroupOffsets>,
}

impl Into<DrawBindGroup> for BufferGroup {
	#[inline]
    fn into(self) -> DrawBindGroup {
        DrawBindGroup::Offset(self)
    }
}

impl BufferGroup {
    pub fn get_group(&self) -> OffsetGroup {
        debug_assert!(self.group_offsets.bind_group.is_some());
        OffsetGroup {
            bind_group: self.group_offsets.bind_group.as_ref().unwrap(),
            offsets: self.group_offsets.offsets.get_offsets(self.index.index() as usize),
        }
    }

    /// 设置uniform
    #[inline]
    pub fn set_uniform<T: Uniform>(&mut self, t: &T) -> Result<(), DynBufferError> {
        self.update_buffer(t, T::Binding::binding())
    }

    /// 更新buffer
    pub fn update_buffer<T: WriteBuffer>(
        &mut self,
        t: &T,
        binding: u32,
    ) -> Result<(), DynBufferError> {
        // SAFE:此处更新一个bindgroup自身的bufer区域，而不会在该区域以外的地方写入， 并且buffer是不扩容的，因此安全
        let context = unsafe { &mut *(Share::as_ptr(&self.context) as usize as *mut GroupBuffer) };
        let index = match context.binding_offset_map.get(binding as usize) {
            Some(r) => *r,
            None => return Err(DynBufferError::BindingNotFind(binding)),
        };
        let buffers = &mut context.values[self.bindings_index];
        let offset = buffers.group_offsets.offsets.get_offsets(self.index.index() as usize)[index];
        context.values[self.bindings_index].fill(index, offset, t);
        Ok(())
    }
}

impl Drop for BufferGroup {
    fn drop(&mut self) {
        self.context.values[self.bindings_index].recycle(self.index);
    }
}

impl Debug for BufferGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynBufferIndex")
            .field("buffer_offset", &self.index)
            .field("buffer_index", &self.bindings_index)
            .finish()
    }
}

pub struct OffsetGroup<'a> {
    pub bind_group: &'a BindGroup,
    pub offsets: &'a [u32],
}

struct BufferOffsets {
    values: Vec<u32>,
    item_size: usize,
}

impl BufferOffsets {
    fn new(block_size: &[u32], count: usize) -> Self {
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            for size in block_size.iter() {
                values.push(size * i as u32);
            }
        }
        Self {
            values,
            item_size: block_size.len(),
        }
    }

    fn get_offsets(&self, index: usize) -> &[u32] {
        let start = index * self.item_size;
        &self.values[start..start + self.item_size]
    }
}

// Option<BindGroup>
struct GroupOffsets {
    bind_group: Option<BindGroup>,
    offsets: BufferOffsets,
}

pub struct DynGroupBufferInfo {
    entrys: Vec<wgpu::BindGroupLayoutEntry>,
    layout: Share<BindGroupLayout>,
    // 每个元素表示一组buffer, 一组buffer对应一个bindgroup
    // 占用内存最大的bingding的索引（以此索引中的buffer的内存来判断是否超过内存限制）
    // max_size_binding_index: usize,
    limit_count: usize,
    binding_size_list: Vec<u32>,
    // 每个buffer的限制长度
    label: Option<String>,
    // layout_label: Option<String>,
    group_label: Option<String>,
}

struct GroupBuffer {
    mutex: ShareMutex<()>,
    values: SmallVec<[GroupBuffersAlloter; 1]>,
    // 最近使用的buffer索引（在buffers字段中的索引）
    lately_use_buffer: usize,
    // entry中描述的多个bingding将作为整体，分配一个块， 每个bingding在该块中，会对应一个偏移，该偏移存储在binding_offset_map中（VecMap的key为entry的binding值）
    binding_offset_map: VecMap<usize>,
}


/// BindGroup buffer分配器
#[derive(Deref)]
pub struct GroupBuffersAlloter {
	#[deref]
    buffers: MulBufferAlloter,
	// 偏移
	group_offsets: Share<GroupOffsets> 
}

impl GroupBuffersAlloter {
	/// 创建 BindGroup Buffer分配器
	/// -block_count：块数量
	/// -block_size_list： 每块的大小（bindgroup中可能包含多个binding，每个binding的块大小不一样）
    pub fn new(block_count: usize, block_size_list: &[u32]) -> Self {
        Self {
            buffers: MulBufferAlloter::new(block_count, block_size_list, BufferUsages::COPY_DST | BufferUsages::UNIFORM),
            group_offsets: Share::new(GroupOffsets {
                bind_group: None,
                offsets: BufferOffsets::new(block_size_list, block_count),
            }),
        }
    }

	/// 更新buffer到显存
	/// 通常每帧调用一次
	#[inline]
    pub fn write_buffer(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        info: &DynGroupBufferInfo,
        layout: &BindGroupLayout,
    ) {
        let buffer_is_create = self.buffers.write_buffer(device, queue, info);

        // 如果有buffer扩容，则重新创建bindgroup
        if buffer_is_create {
            let mut entries = Vec::new();
            for (i, entry) in info.entrys.iter().enumerate() {
                let buffer = self.buffer_maps[i].wgpu_buffer().unwrap();
                entries.push(wgpu::BindGroupEntry {
                    binding: entry.binding,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset: 0,
                        size: std::num::NonZeroU64::new(info.binding_size_list[i] as u64),
                    }),
                })
            }

			// 只有此处会写入，其他地方不会写入。 此处应该是安全的？
			let group =
			unsafe { &mut *(Share::as_ptr(&self.group_offsets) as usize as *mut GroupOffsets) };
			group.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: entries.as_slice(),
			label: match &info.group_label {
				Some(r) => Some(r.as_str()),
				None => None,
			},
			}));
        }
    }
}


/// 多个buffer的分配器（通常这多个buffer的每个对应一个bindgroup的一个binding）
#[derive(Deref)]
pub struct MulBufferAlloter {
    buffer_maps: Vec<BufferMap>,
	// 空位标识
	#[deref]
    id_alloter: IdAlloterWithCountLimit,
}

impl MulBufferAlloter {
	/// 创建 BindGroup Buffer分配器
	/// -block_count：块数量
	/// -block_size_list： 每块的大小（bindgroup中可能包含多个binding，每个binding的块大小不一样）
    pub fn new(block_count: usize, block_size_list: &[u32], usage: wgpu::BufferUsages) -> Self {
        let mut buffer_maps = Vec::with_capacity(block_size_list.len());
        for block_size in block_size_list.iter() {
            buffer_maps.push(BufferMap::new(*block_size as usize * block_count, usage));
        }

        Self {
            buffer_maps,
            id_alloter: IdAlloterWithCountLimit::new(block_count as u32),
        }
    }

	/// 在buffer中填充数据
	#[inline]
    pub fn fill<T: WriteBuffer>(&mut self, binding: usize, offset: u32, value: &T) {
        self.buffer_maps[binding].cache_buffer.full(offset, value);
    }

	/// 更新buffer到显存
	#[inline]
    pub fn write_buffer(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        info: &DynGroupBufferInfo,
    ) -> bool {
        let mut buffer_is_create = false;
        for i in self.buffer_maps.iter_mut() {
            buffer_is_create = i.write_buffer(device, queue, &info.label) || buffer_is_create;
        }
		buffer_is_create
    }
}

/// 单个buffer的分配器
#[derive(Deref)]
pub struct SingleBufferAlloter {
    buffer_map: BufferMap,
	// 空位标识
	#[deref]
    occupied_mark: IdAlloterWithCountLimit,
}

impl SingleBufferAlloter {
	/// 创建 BindGroup Buffer分配器
	/// -block_count：块数量
	/// -block_size： 每块的大小（单位：字节）
    pub fn new(block_count: usize, block_size: u32, usage: wgpu::BufferUsages) -> Self {
        Self {
            buffer_map: BufferMap::new(block_size as usize * block_count, usage),
            occupied_mark: IdAlloterWithCountLimit::new(block_count as u32),
        }
    }

	/// 在buffer中填充数据
	#[inline]
    pub fn fill<T: WriteBuffer>(&mut self, offset: u32, value: &T) {
        self.buffer_map.cache_buffer.full(offset, value);
    }

	/// 更新代码到显存
	#[inline]
    pub fn write_buffer(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        info: &Option<String>,
    ) -> bool {
        self.buffer_map.write_buffer(device, queue, info)
    }

	pub fn wgpu_buffer(&self) -> Option<&Buffer> {
		self.buffer_map.wgpu_buffer()
	}
}
use crossbeam::queue::SegQueue;

#[derive(Debug, Clone, Copy)]
pub struct Index {
	version: u32,
	index: u32,
}

impl Index {
	#[inline]
	pub fn version(&self) -> u32 {
		self.version
	}
	#[inline]
	pub fn index(&self) -> u32 {
		self.index
	}
}

/// `IdAlloter` 结构体用于线程安全地分配和回收索引。
/// 结构体包含两个字段，`max_index`表示已分配索引的最大值，`recycled`用于存储曾经分配出去，后又被回收的索引
/// 分配索引时， 如果recycled长度大于0，将从recycled中弹出一个索引，否则，分配的索引值为`max_index`,并且`max_index`会自增1
pub struct IndexAlloter {
	max_index: AtomicU32,
	recycled: SegQueue<Index>
}

impl IndexAlloter {
	/// 构造方法
	pub fn new() -> Self {
		IndexAlloter {
			max_index: AtomicU32::new(0),
			recycled: SegQueue::new(),
		}
	}

	/// 分配一个索引
	pub fn alloc(&self) -> Index {
		// 如果recycled中存在回收索引，将从recycled中弹出一个索引，否则，分配的索引值为`max_index`,并且`max_index`会自增1
		match self.recycled.pop() {
			Some(r) => Index{
				index: r.index,
				version: r.version + 1,
			},
			None => Index {
				version: 0,
				index: self.max_index.fetch_add(1, Ordering::Relaxed)
			},
		}
	}

	/// 回收一个索引
	pub fn recycle(&self, id: Index) {
		self.recycled.push(id);
	}

	
	/// 已回收的索引个数
	pub fn recycle_len(&self) -> u32 {
		self.recycled.len() as u32
	}

	/// 当前已分配索引的最大值
	pub fn cur_max(&self) -> u32 {
		self.max_index.load(Ordering::Relaxed)
	}
}

/// `IdAlloterWithCountLimit`一个有数量限制的索引分配器
pub struct IdAlloterWithCountLimit {
	id_alloter: IndexAlloter,
	capacity: u32,
}

impl IdAlloterWithCountLimit {
	/// 构造方法
	pub fn new(capacity: u32) -> Self {
		Self {
			id_alloter: IndexAlloter::new(),
			capacity,
		}
	}

	/// 分配一个索引
	pub fn alloc(&self) -> Option<Index> {
		let id = self.id_alloter.alloc();
		if id.index() >= self.capacity { 
			// 如果已经分配到最大的id， 则返回None
			// 并且分配到的id永不释放（无所谓，该id不占用内存）
			None
		} else {
			Some(id)
		}
	}

	/// 回收一个索引
	#[inline]
	pub fn recycle(&self, id: Index) {
		self.id_alloter.recycle(id);
	}
	
	// 是否为空（一个id也未分配, 或分配过但又全部回收了）
	pub fn is_empty(&self) -> bool {
		let cur_max = self.id_alloter.cur_max();
		if cur_max >= self.capacity && self.id_alloter.recycle_len() == self.capacity{
			return true;
		}
		false
	}

	pub fn capacity(&self) -> u32 {
		self.capacity
	}
}


/// `OccupiedMark` 结构体用于跟踪一个数据结构的占用情况，其中占用情况由位向量表示。
/// 位向量有一个位的数组，每个位代表一个元素是否被占用，为 0 表示未被占用，为 1 表示已被占用。
pub struct OccupiedMarker {
    occupied_mark: BitVec,
}

impl OccupiedMarker {
    /// 创建一个新的 `OccupiedMark` 结构体实例。
    ///
    /// # Arguments
    ///
    /// * `count` - 需要被跟踪占用情况的元素数量。
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_allocator::occupied_mark::OccupiedMark;
    ///
    /// let mut mark = OccupiedMark::new(10);
    /// ```
    pub fn new(count: usize) -> Self {
        // 创建一个指定容量的 BitVec，BitVec 内部使用一个 Vec<u64> 对象来存储位信息
        let mut occupied_mark = BitVec::with_capacity(count);
        // 设置 BitVec 的长度为 count，因为 BitVec 内部使用一个 Vec<u64> 对象来存储位信息，所以这里必须使用 unsafe，因为实际上，BitVec 内部存储的长度可能远远超过 count
        unsafe { occupied_mark.set_len(count) };
        // 初始化 BitVec 中所有的位都是 false，即没有被占用
        occupied_mark[0..count].fill(false);
        Self {
            occupied_mark,
        }
    }

    /// 分配一个未被占用的元素，将其占用并返回其下标。
    ///
    /// # Returns
    ///
    /// 返回未被占用的元素下标，如果不存在未被占用的元素，则返回 None。
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_render::rhi::dyn_uniform_buffer::OccupiedMark;
    ///
    /// let mut mark = OccupiedMark::new(10);
    /// let index = mark.alloc();
    /// ```
    #[inline]
    pub fn alloc(&mut self) -> Option<usize> {
        // 找到第一个为 0 的位，表示这个位可用，标记为 1，并返回该位的索引
        let r = self.occupied_mark.first_zero();
        if let Some(r) = r {
            self.occupied_mark.set(r, true);
        }
        r
    }

    /// 释放指定下标的元素，将其标记为未被占用。
    ///
    /// # Arguments
    ///
    /// * `index` - 需要被释放的元素下标。
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_allocator::occupied_mark::OccupiedMark;
    ///
    /// let mut mark = OccupiedMark::new(10);
    /// let index = mark.alloc();
    /// mark.free(index.unwrap());
    /// ```
    pub fn free(&mut self, index: usize) {
        self.occupied_mark.set(index, false);
    }

    /// 返回跟踪占用情况的元素数量。
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_allocator::occupied_mark::OccupiedMark;
    ///
    /// let mark = OccupiedMark::new(10);
    /// let count = mark.len();
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.occupied_mark.len() as usize
    }
}

/// buffer映射
/// 内存buffer到wgpu buffer的映射
pub struct BufferMap {
    cache_buffer: BufferCache,
    buffer: Option<Buffer>,
	usage: wgpu::BufferUsages,
}

impl BufferMap {
    pub fn new(len: usize, usage: wgpu::BufferUsages) -> Self {
        Self {
            cache_buffer: BufferCache::new(len),
            buffer: None,
			usage,
        }
    }

    #[inline]
    pub fn wgpu_buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    /// 写入buffer到显存，返回是否重新创建了buffer
    pub fn write_buffer(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        label: &Option<String>,
    ) -> bool {
        // 什么也没改变， 直接返回
        if self.cache_buffer.change_range.len() == 0 {
            return false;
        }

        let r = match &self.buffer {
            Some(buffer) => {
                queue.write_buffer(
                    buffer,
                    self.cache_buffer.change_range.start as u64,
                    &self.cache_buffer.buffer()[self.cache_buffer.change_range.start as usize
                        ..self.cache_buffer.change_range.end as usize],
                );

                false
            }
            None => {
                self.buffer = Some(device.create_buffer_with_data(&BufferInitDescriptor {
                    label: label.as_deref(),
                    usage: self.usage,
                    contents: self.cache_buffer.buffer(),
                }));
                // self.old_len = size;
                true
            }
        };
		self.cache_buffer.reset_change_range();
        r
    }
}

/// buffer缓存
/// 包含一个内存的buffer和这段buffer的修改范围
#[derive(Debug)]
struct BufferCache {
    buffer: Vec<u8>, // buffer
    change_range: Range<u32>, // 修改范围
}

impl BufferCache {
    #[inline]
    pub fn new(len: usize) -> Self {
        let mut buffer = Vec::with_capacity(len);
        unsafe { buffer.set_len(len) };
        Self {
            buffer,
            change_range: 0..0,
        }
    }

    /// 在已分配位置上填充buffer
    pub fn full<T: WriteBuffer>(&mut self, index: u32, t: &T) {
        debug_assert!(self.buffer.len() >= (index + t.byte_len()) as usize);

        t.write_into(index, self.buffer.as_mut_slice());

        // 设置数据变化范围
        let start = index + t.offset();
        let end = start + t.byte_len();
        if self.change_range.len() == 0 {
            self.change_range.start = start;
            self.change_range.end = end;
            // println!(
            //     "full1======{:?}, {:?}",
            //     self.change_range,
            //     bytemuck::cast_slice::<_, f32>(&self.buffer)
            // );
            return;
        }
        if self.change_range.start > start {
            self.change_range.start = start;
        }
        if self.change_range.end < end {
            self.change_range.end = end;
        }

        // println!(
        //     "full======{:?}, {:?}, {}, {:?}",
        //     self.change_range,
        //     start,
        //     end,
        //     bytemuck::cast_slice::<_, f32>(&self.buffer),
        // );
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        self.buffer.as_slice()
    }

	#[inline]
	pub fn reset_change_range(&mut self) {
		self.change_range = 0..0;
	}
}

#[derive(Error, Debug)]
pub enum DynBufferError {
    #[error("binding is not exist: {0:?}")]
    BindingNotFind(u32),
    #[error("min_binding_size is miss")]
    MissMinSize,
    #[error("entry must be of type buffer, actual: {0:?}")]
    NotNuffer(String),
    // #[error("import key is not exist: {0:?}")]
    // ImportNotFind(ShaderImport),

    // #[error("var type is not support: {0:?}")]
    // TypeNotSupport(String),

    // #[error("validation var fail, expect: {0:?}, actual is: {1:?}")]
    // ValidationVarFail(String, String),

    // #[error("invalid import path: {0:?}")]
    // InvalidImportPath(String),

    // #[error(transparent)]
    // WgslParse(#[from] naga::front::wgsl::ParseError),

    // #[error("GLSL Parse Error: {0:?}")]
    // GlslParse(Vec<naga::front::glsl::Error>),

    // #[error(transparent)]
    // SpirVParse(#[from] naga::front::spv::Error),

    // #[error(transparent)]
    // Validation(#[from] naga::WithSpan<naga::valid::ValidationError>),

    // #[error(transparent)]
    // WgslParse1(#[from] pi_naga::front::wgsl::ParseError),

    // #[error("GLSL Parse Error: {0:?}")]
    // GlslParse1(Vec<pi_naga::front::glsl::Error>),

    // #[error(transparent)]
    // SpirVParse1(#[from] pi_naga::front::spv::Error),

    // #[error(transparent)]
    // Validation1(#[from] pi_naga::WithSpan<naga::valid::ValidationError>),
}

#[cfg(test)]
mod test {
    use crate::{self as pi_render, rhi::{dyn_uniform_buffer::GroupAlloter, options::{RenderOptions, RenderPriority}, device::RenderDevice, RenderQueue}};
    use pi_async::rt::AsyncRuntime;
    use pi_share::Share;
    use render_derive::{BindLayout, BindingType, BufferSize, Uniform};
	use winit::{event_loop::{EventLoopBuilder}, platform::windows::EventLoopBuilderExtWindows};
	use std::sync::{Arc, atomic::AtomicBool};

	/// Initializes the renderer by retrieving and preparing the GPU instance, device and queue
	/// for the specified backend.
	async fn initialize_renderer(
		instance: &wgpu::Instance,
		options: &RenderOptions,
		request_adapter_options: &wgpu::RequestAdapterOptions<'_>,
	) -> (RenderDevice, RenderQueue, wgpu::AdapterInfo) {
		let adapter = instance
			.request_adapter(request_adapter_options)
			.await
			.expect("Unable to find a GPU! Make sure you have installed required drivers!");

		let adapter_info = adapter.get_info();

		// #[cfg(not(feature = "trace"))]
		let trace_path = None;

		// Maybe get features and limits based on what is supported by the adapter/backend
		let mut features = wgpu::Features::empty();
		let mut limits = options.limits.clone();
		if matches!(options.priority, RenderPriority::Functionality) {
			features = adapter.features() | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
			if adapter_info.device_type == wgpu::DeviceType::DiscreteGpu {
				// `MAPPABLE_PRIMARY_BUFFERS` can have a significant, negative performance impact for
				// discrete GPUs due to having to transfer data across the PCI-E bus and so it
				// should not be automatically enabled in this case. It is however beneficial for
				// integrated GPUs.
				features -= wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
			}
			limits = adapter.limits();
		}

		// Enforce the disabled features
		if let Some(disabled_features) = options.disabled_features {
			features -= disabled_features;
		}
		// NOTE: |= is used here to ensure that any explicitly-enabled features are respected.
		features |= options.features;

		// Enforce the limit constraints
		if let Some(constrained_limits) = options.constrained_limits.as_ref() {
			// NOTE: Respect the configured limits as an 'upper bound'. This means for 'max' limits, we
			// take the minimum of the calculated limits according to the adapter/backend and the
			// specified max_limits. For 'min' limits, take the maximum instead. This is intended to
			// err on the side of being conservative. We can't claim 'higher' limits that are supported
			// but we can constrain to 'lower' limits.
			limits = wgpu::Limits {
				max_texture_dimension_1d: limits
					.max_texture_dimension_1d
					.min(constrained_limits.max_texture_dimension_1d),
				max_texture_dimension_2d: limits
					.max_texture_dimension_2d
					.min(constrained_limits.max_texture_dimension_2d),
				max_texture_dimension_3d: limits
					.max_texture_dimension_3d
					.min(constrained_limits.max_texture_dimension_3d),
				max_texture_array_layers: limits
					.max_texture_array_layers
					.min(constrained_limits.max_texture_array_layers),
				max_bind_groups: limits
					.max_bind_groups
					.min(constrained_limits.max_bind_groups),
				max_dynamic_uniform_buffers_per_pipeline_layout: limits
					.max_dynamic_uniform_buffers_per_pipeline_layout
					.min(constrained_limits.max_dynamic_uniform_buffers_per_pipeline_layout),
				max_dynamic_storage_buffers_per_pipeline_layout: limits
					.max_dynamic_storage_buffers_per_pipeline_layout
					.min(constrained_limits.max_dynamic_storage_buffers_per_pipeline_layout),
				max_sampled_textures_per_shader_stage: limits
					.max_sampled_textures_per_shader_stage
					.min(constrained_limits.max_sampled_textures_per_shader_stage),
				max_samplers_per_shader_stage: limits
					.max_samplers_per_shader_stage
					.min(constrained_limits.max_samplers_per_shader_stage),
				max_storage_buffers_per_shader_stage: limits
					.max_storage_buffers_per_shader_stage
					.min(constrained_limits.max_storage_buffers_per_shader_stage),
				max_storage_textures_per_shader_stage: limits
					.max_storage_textures_per_shader_stage
					.min(constrained_limits.max_storage_textures_per_shader_stage),
				max_uniform_buffers_per_shader_stage: limits
					.max_uniform_buffers_per_shader_stage
					.min(constrained_limits.max_uniform_buffers_per_shader_stage),
				max_uniform_buffer_binding_size: limits
					.max_uniform_buffer_binding_size
					.min(constrained_limits.max_uniform_buffer_binding_size),
				max_storage_buffer_binding_size: limits
					.max_storage_buffer_binding_size
					.min(constrained_limits.max_storage_buffer_binding_size),
				max_vertex_buffers: limits
					.max_vertex_buffers
					.min(constrained_limits.max_vertex_buffers),
				max_vertex_attributes: limits
					.max_vertex_attributes
					.min(constrained_limits.max_vertex_attributes),
				max_vertex_buffer_array_stride: limits
					.max_vertex_buffer_array_stride
					.min(constrained_limits.max_vertex_buffer_array_stride),
				max_push_constant_size: limits
					.max_push_constant_size
					.min(constrained_limits.max_push_constant_size),
				min_uniform_buffer_offset_alignment: limits
					.min_uniform_buffer_offset_alignment
					.max(constrained_limits.min_uniform_buffer_offset_alignment),
				min_storage_buffer_offset_alignment: limits
					.min_storage_buffer_offset_alignment
					.max(constrained_limits.min_storage_buffer_offset_alignment),
				max_inter_stage_shader_components: limits
					.max_inter_stage_shader_components
					.min(constrained_limits.max_inter_stage_shader_components),
				max_compute_workgroup_storage_size: limits
					.max_compute_workgroup_storage_size
					.min(constrained_limits.max_compute_workgroup_storage_size),
				max_compute_invocations_per_workgroup: limits
					.max_compute_invocations_per_workgroup
					.min(constrained_limits.max_compute_invocations_per_workgroup),
				max_compute_workgroup_size_x: limits
					.max_compute_workgroup_size_x
					.min(constrained_limits.max_compute_workgroup_size_x),
				max_compute_workgroup_size_y: limits
					.max_compute_workgroup_size_y
					.min(constrained_limits.max_compute_workgroup_size_y),
				max_compute_workgroup_size_z: limits
					.max_compute_workgroup_size_z
					.min(constrained_limits.max_compute_workgroup_size_z),
				max_compute_workgroups_per_dimension: limits
					.max_compute_workgroups_per_dimension
					.min(constrained_limits.max_compute_workgroups_per_dimension),
				max_buffer_size: limits
					.max_buffer_size
					.min(constrained_limits.max_buffer_size),
    			max_bindings_per_bind_group: limits
					.max_bindings_per_bind_group
					.min(constrained_limits.max_bindings_per_bind_group),
			};
		}

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: options.device_label.as_ref().map(|a| a.as_ref()),
					features,
					limits,
				},
				trace_path,
			)
			.await
			.unwrap();
		let device = Share::new(device);
		let queue = Share::new(queue);

		(RenderDevice::from(device), queue, adapter_info)
	}

	#[derive(BindLayout, BufferSize, BindingType)]
    #[layout(set(0), binding(0))]
    #[min_size(144)]
    #[uniformbuffer]
    pub struct CameraMatrixBind;

    #[derive(Uniform)]
    #[uniform(offset(0), len(64), bind(CameraMatrixBind))]
    pub struct ProjectUniform<'a>(pub &'a [f32]);

    #[derive(Uniform)]
    #[uniform(offset(64), len(64), bind(CameraMatrixBind))]
    pub struct ViewUniform<'a>(pub &'a [f32]);

    #[derive(BindLayout, BufferSize, BindingType)]
    #[layout(set(0), binding(1))]
    #[min_size(32)]
    #[uniformbuffer]
    pub struct ColorMaterialBind;

    #[derive(Uniform)]
    #[uniform(offset(0), len(16), bind(ColorMaterialBind))]
    pub struct Color<'a>(pub &'a [f32]);

    #[derive(Uniform)]
    #[uniform(offset(16), len(16), bind(ColorMaterialBind))]
    pub struct Color1<'a>(pub &'a [f32]);

    #[test]
    fn test() {
		let is_end = Arc::new(AtomicBool::new(false));
		let is_end1 = is_end.clone();

		let options = RenderOptions::default();
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			/// Which `Backends` to enable.
			backends: options.backends,
			/// Which DX12 shader compiler to use.
			dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
		});
		let event_loop =  EventLoopBuilder::new().with_any_thread(true).build();
		let window = winit::window::Window::new(&event_loop).unwrap();

		let surface = unsafe {instance.create_surface(&window).unwrap()};
		

		pi_hal::runtime::MULTI_MEDIA_RUNTIME.spawn(pi_hal::runtime::MULTI_MEDIA_RUNTIME.alloc(), async move {
			let request_adapter_options = wgpu::RequestAdapterOptions {
				power_preference: options.power_preference,
				compatible_surface: Some(&surface),
				..Default::default()
			};
			
			let (device, _queue, _adapter_info) =
			initialize_renderer(&instance, &options, &request_adapter_options).await;

			let entry = vec![
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: true,
						min_binding_size: wgpu::BufferSize::new(128),
					},
					count: None, // TODO
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: true,
						min_binding_size: wgpu::BufferSize::new(32),
					},
					count: None, // TODO
				}
			];
			let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				label: None,
				entries: entry.as_slice(),
			});

			let group_alloter = GroupAlloter::new(
				None,
				64,
				64 * 1024 * 1024,
				Some(64 * 10),
				vec![
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: true,
							min_binding_size: wgpu::BufferSize::new(128),
						},
						count: None, // TODO
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: true,
							min_binding_size: wgpu::BufferSize::new(32),
						},
						count: None, // TODO
					},
				],
				Share::new(layout),
			).unwrap();

			let mut r = group_alloter.alloc();
			r.set_uniform(&ProjectUniform(&[
				1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
			]))
			.unwrap();
			r.set_uniform(&ViewUniform(&[
				11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 110.0, 111.0, 112.0, 113.0,
				114.0, 115.0, 116.0,
			]))
			.unwrap();
			r.set_uniform(&Color(&[21.0, 22.0, 23.0, 24.0])).unwrap();
			r.set_uniform(&Color1(&[31.0, 32.0, 33.0, 34.0])).unwrap();

			let mut r = group_alloter.alloc();
			r.set_uniform(&ProjectUniform(&[
				1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
			]))
			.unwrap();
			r.set_uniform(&ViewUniform(&[
				11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 110.0, 111.0, 112.0, 113.0,
				114.0, 115.0, 116.0,
			]))
			.unwrap();
			r.set_uniform(&Color(&[21.0, 22.0, 23.0, 24.0])).unwrap();
			r.set_uniform(&Color1(&[31.0, 32.0, 33.0, 34.0])).unwrap();

			is_end1.store(true, std::sync::atomic::Ordering::Relaxed);
		}).unwrap();

		loop {
			if is_end.load(std::sync::atomic::Ordering::Relaxed) {
				break;
			}
		}
		
    }
}

#[cfg(test)]
mod test_occupied_mark {
    use super::*;

    #[test]
    fn test_alloc() {
        let mut occupied_mark = OccupiedMarker::new(100);

        for i in 0..100 {
            let r = occupied_mark.alloc().unwrap();
            assert_eq!(i, r);
        }
    }

    #[test]
    fn test_free() {
        let mut occupied_mark = OccupiedMarker::new(100);

        for i in 0..100 {
            let r = occupied_mark.alloc().unwrap();
            assert_eq!(i, r);
        }

        for i in (0..100).step_by(2) {
            occupied_mark.free(i);
        }

        for i in (0..100).step_by(2) {
            let r = occupied_mark.alloc().unwrap();
            assert_eq!(i, r);
        }

		assert_eq!(None, occupied_mark.alloc());
    }
}

#[cfg(test)]
mod test_id_alloc {
    use super::*;

    #[test]
    fn test_alloc() {
        let id_alloter = IdAlloterWithCountLimit::new(100);

        for i in 0..100 {
            let r = id_alloter.alloc().unwrap();
            assert_eq!(i, r.index());
        }
    }

    #[test]
    fn test_free() {
        let id_alloter = IdAlloterWithCountLimit::new(100);

		let mut indexs = Vec::new();
        for i in 0..100 {
            let r = id_alloter.alloc().unwrap();
			assert_eq!(i, r.index());
			indexs.push(r);
            
        }

        for i in indexs.into_iter().step_by(2) {
            id_alloter.recycle(i);
        }

        for i in (0..100).step_by(2) {
            let r = id_alloter.alloc().unwrap();
            assert_eq!(i, r.index());
        }

		assert_eq!(true, id_alloter.alloc().is_none());
    }
}
