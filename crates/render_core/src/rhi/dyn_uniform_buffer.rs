use std::{fmt::Debug, ops::Range};

use bitvec::vec::BitVec;
use pi_map::vecmap::VecMap;
use pi_share::{Share, ShareMutex};
use smallvec::{smallvec, SmallVec};
use thiserror::Error;
use wgpu::{util::BufferInitDescriptor, BufferUsages};

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
///
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
        let (layout_label, group_label) = match &label {
            Some(r) => (Some(r.clone() + " layout"), Some(r.clone() + " group")),
            None => (None, None),
        };
        let limit_count = (limit_size as usize) / max_size;
        let init_size = match init_size {
            Some(init_size) => init_size as usize / max_size,
            None => 6400 / max_size,
        };
        let buffer_maps = BufferMaps::new(init_size, binding_size_list.as_slice());

        Ok(GroupAlloter {
            buffers: Share::new(GroupBuffer {
                values: smallvec![buffer_maps],
                lately_use_buffer: 0,
                binding_offset_map,
                mutex: ShareMutex::new(()),
            }),
            info: DynGroupBufferInfo {
                entrys,
                layout: None,
                limit_count,
                binding_size_list,
                label,
                layout_label,
                group_label,
            },
        })
    }

    /// 为bindgroup分配索引
    pub fn alloc(&self) -> BufferGroup {
        // 寻找一个可以分配的buffer
        fn alloc(context: &GroupAlloter) -> (usize, usize) {
            // SAFE: 这里转为可变，立即解锁，保证alloc在多线程下不冲突
            let buffers =
                unsafe { &mut *(Share::as_ptr(&context.buffers) as usize as *mut GroupBuffer) };
            let _lock = buffers.mutex.lock();

            let info = &context.info;

            // 如果最近分配过的buffer已经超出内存限制，则创建或找到一个有剩余空间的buffer
            if let Some(r) = buffers.values[buffers.lately_use_buffer].alloc() {
                return (r, buffers.lately_use_buffer);
            }

            // 找到一个存在空闲位置的buffer组
            for (index, buffer) in buffers.values.iter_mut().enumerate() {
                if let Some(r) = buffer.alloc() {
                    buffers.lately_use_buffer = index;
                    (r, index);
                }
            }

            let next_count = info
                .limit_count
                .min(buffers.values.last().unwrap().len() * 2);

            // 如果未找到，则创建新的
            // let mut buffer = Vec::with_capacity(info.binding_size_list.len());

            buffers.lately_use_buffer = context.buffers.values.len();
            let mut buffer_maps = BufferMaps::new(next_count, info.binding_size_list.as_slice());
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
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        // SAFE
        let buffer_lock =
            unsafe { &mut *(Share::as_ptr(&self.buffers) as usize as *mut GroupBuffer) };
        let _lock = self.buffers.mutex.lock();

        let buffer_lock = &mut *buffer_lock;
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
        for buffers in buffer_lock.values.iter_mut() {
            buffers.write_buffer(device, queue, &self.info, layout);
        }
    }
}

/// group索引
pub struct BufferGroup {
    index: usize,
    bindings_index: usize,
    context: Share<GroupBuffer>,
    group_offsets: Share<GroupOffsets>,
}

impl BufferGroup {
    pub fn get_group(&self) -> OffsetGroup {
        debug_assert!(self.group_offsets.bind_group.is_some());
        OffsetGroup {
            bind_group: self.group_offsets.bind_group.as_ref().unwrap(),
            offsets: self.group_offsets.offsets.get_offsets(self.index),
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
        let offset = buffers.group_offsets.offsets.get_offsets(self.index)[index];
        context.values[self.bindings_index].fill(index, offset, t);
        Ok(())
    }
}

impl Drop for BufferGroup {
    fn drop(&mut self) {
        let _lock = self.context.mutex.lock();
        let context = unsafe { &mut *(Share::as_ptr(&self.context) as usize as *mut GroupBuffer) };
        context.values[self.bindings_index].de_alloc(self.index);
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

/// buffer映射
/// 内存buffer到wgpu buffer的映射
pub struct BufferMap {
    cache_buffer: BufferCache,
    buffer: Option<Buffer>,
}

impl BufferMap {
    pub fn new(len: usize) -> Self {
        Self {
            cache_buffer: BufferCache::new(len),
            buffer: None,
            // old_len: 0,
            // block_size,
        }
    }

    #[inline]
    pub fn wgpu_buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    /// 写入buffer到现存，返回是否重新创建了buffer
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
                    usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                    contents: self.cache_buffer.buffer(),
                }));
                // self.old_len = size;
                true
            }
        };
        self.cache_buffer.change_range = 0..0;
        r
    }
}

struct GroupBuffer {
    mutex: ShareMutex<()>,
    values: SmallVec<[BufferMaps; 1]>,
    // 最近使用的buffer索引（在buffers字段中的索引）
    lately_use_buffer: usize,
    // entry中描述的多个bingding将作为整体，分配一个块， 每个bingding在该块中，会对应一个偏移，该偏移存储在binding_offset_map中（VecMap的key为entry的binding值）
    binding_offset_map: VecMap<usize>,
}

struct BufferMaps {
    values: Vec<BufferMap>,
    occupied_mark: BitVec,
    group_offsets: Share<GroupOffsets>,
}

impl BufferMaps {
    fn new(count: usize, block_size_list: &[u32]) -> Self {
        let mut occupied_mark = BitVec::with_capacity(count);
        unsafe { occupied_mark.set_len(count) };
        occupied_mark[0..count].fill(false);

        let mut buffers = Vec::with_capacity(block_size_list.len());

        for i in block_size_list.iter() {
            buffers.push(BufferMap::new(*i as usize * count));
        }
        Self {
            values: buffers,
            occupied_mark,
            group_offsets: Share::new(GroupOffsets {
                bind_group: None,
                offsets: BufferOffsets::new(block_size_list, count),
            }),
        }
    }
    /// 分配
    #[inline]
    fn alloc(&mut self) -> Option<usize> {
        let r = self.occupied_mark.first_zero();
        if let Some(r) = r {
            self.occupied_mark.set(r, true);
        }
        r
    }

    /// 移除分配
    pub fn de_alloc(&mut self, i: usize) {
        self.occupied_mark.set(i, false);
    }

    #[inline]
    fn fill<T: WriteBuffer>(&mut self, binding: usize, offset: u32, value: &T) {
        self.values[binding].cache_buffer.full(offset, value);
    }

    #[inline]
    fn len(&self) -> usize {
        self.occupied_mark.len() as usize
    }

    #[inline]
    fn write_buffer(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        info: &DynGroupBufferInfo,
        layout: &BindGroupLayout,
    ) {
        let mut buffer_is_create = false;
        for i in self.values.iter_mut() {
            buffer_is_create = i.write_buffer(device, queue, &info.label) || buffer_is_create;
        }

        // 如果有buffer扩容，则重新创建bindgroup
        if buffer_is_create {
            let mut entries = Vec::new();
            for (i, entry) in info.entrys.iter().enumerate() {
                let buffer = self.values[i].wgpu_buffer().unwrap();
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
// Option<BindGroup>
struct GroupOffsets {
    bind_group: Option<BindGroup>,
    offsets: BufferOffsets,
}

struct DynGroupBufferInfo {
    entrys: Vec<wgpu::BindGroupLayoutEntry>,
    layout: Option<BindGroupLayout>,
    // 每个元素表示一组buffer, 一组buffer对应一个bindgroup
    // 占用内存最大的bingding的索引（以此索引中的buffer的内存来判断是否超过内存限制）
    // max_size_binding_index: usize,
    limit_count: usize,
    binding_size_list: Vec<u32>,
    // 每个buffer的限制长度
    label: Option<String>,
    layout_label: Option<String>,
    group_label: Option<String>,
}

/// buffer分配器
///指定从一个大的buffer中分配一个区域
#[derive(Debug)]
struct BufferCache {
    buffer: Vec<u8>,
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
            println!(
                "full1======{:?}, {:?}",
                self.change_range,
                bytemuck::cast_slice::<_, f32>(&self.buffer)
            );
            return;
        }
        if self.change_range.start > start {
            self.change_range.start = start;
        }
        if self.change_range.end < end {
            self.change_range.end = end;
        }

        println!(
            "full======{:?}, {:?}, {}, {:?}",
            self.change_range,
            start,
            end,
            bytemuck::cast_slice::<_, f32>(&self.buffer),
        );
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        self.buffer.as_slice()
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
    use crate::{self as pi_render, rhi::dyn_uniform_buffer::GroupAlloter};
    use render_derive::{BindLayout, BindingType, BufferSize, Uniform};

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
        )
        .unwrap();

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
    }
}
