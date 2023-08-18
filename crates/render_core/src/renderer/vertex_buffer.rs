use std::{ops::Range, mem::size_of, hash::Hash, sync::Arc, fmt::Debug};

use pi_assets::{asset::{Asset, GarbageEmpty, Handle, Size}, mgr::AssetMgr};
use pi_atom::Atom;
use pi_share::{Share, ShareMutex};
use wgpu::util::BufferInitDescriptor;

use crate::{rhi::{device::RenderDevice, RenderQueue,  buffer::Buffer}, asset::TAssetKeyU64};

use super::{
    attributes::{EVertexDataKind, ShaderAttribute, TAsWgpuVertexAtribute, KeyShaderFromAttributes},
    vertex_buffer_desc::VertexBufferDesc,
    vertex_format::TVertexFormatByteSize,
    buffer::{FixedSizeBufferPool, AssetRWBuffer, RWBufferRange},
};

pub type IDAssetVertexBuffer = u64;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyVertexBuffer(Atom);
impl KeyVertexBuffer {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
impl From<&str> for KeyVertexBuffer {
    fn from(value: &str) -> Self {
        Self(Atom::from(value))
    }
}
impl From<&Atom> for KeyVertexBuffer {
    fn from(value: &Atom) -> Self {
        Self(value.clone())
    }
}
impl TAssetKeyU64 for KeyVertexBuffer {}
pub type AssetVertexBuffer = EVertexBufferRange;

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub type KeyPipelineFromAttributes = Arc<VertexBufferLayouts>;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VertexBufferLayout {
    pub kinds: Vec<EVertexDataKind>,
    pub list: Vec<wgpu::VertexAttribute>,
    pub stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VertexBufferLayouts {
    layout_list: Vec<VertexBufferLayout>,
    pub size: usize,
}
impl From<&Vec<VertexBufferDesc>> for VertexBufferLayouts {
    fn from(value: &Vec<VertexBufferDesc>) -> Self {
        let mut layouts = vec![];

        // 按 EVertexDataKind 排序确定 shader_location
        let mut temp_kinds = vec![];
        value.iter().for_each(|buffer_desc| {
            buffer_desc.attributes().iter().for_each(|attribute| {
                match temp_kinds.binary_search(&attribute.kind) {
                    Ok(_) => {
                        // 重复的顶点属性
                        log::error!("[{:?}] Can only be set once", attribute.kind);
                    },
                    Err(index) => {
                        temp_kinds.insert(index, attribute.kind);
                    },
                }
            });
        });

        let mut datasize = 0;
        value.iter().for_each(|buffer_desc| {
            let mut temp_attributes = VertexBufferLayout { list: vec![], kinds: vec![], stride: 0, step_mode: buffer_desc.step_mode() };

            buffer_desc.attributes().iter().for_each(|attribute| {
                match temp_kinds.binary_search(&attribute.kind) {
                    Ok(shader_location) => {
                        let temp = attribute.as_attribute(temp_attributes.stride, shader_location as u32);

                        temp_attributes.kinds.push(attribute.kind);
                        temp_attributes.list.push(temp);
                        temp_attributes.stride += attribute.format.use_bytes();
                    },
                    Err(_) => todo!(),
                }
            });

            datasize += size_of::<VertexBufferLayout>();
            layouts.push(temp_attributes);
        });

        Self { layout_list: layouts, size: datasize }
    }
}
impl VertexBufferLayouts {
    pub fn as_key_shader_from_attributes(&self) -> KeyShaderFromAttributes {
        let mut result = KeyShaderFromAttributes(vec![]);
        self.layout_list.iter().for_each(|layout| {
            let len = layout.list.len();

            for i in 0..len {
                result.0.push(
                    ShaderAttribute {
                        kind: layout.kinds.get(i).unwrap().clone(),
                        location: layout.list.get(i).unwrap().shader_location,
                    }
                );
            }
        });

        result.0.sort();

        result
    }
    pub fn as_key_pipeline_from_vertex_layout(&self) -> Vec<VertexBufferLayout> {
        self.layout_list.clone()
    }
    pub fn layouts(&self) -> Vec<wgpu::VertexBufferLayout> {
        let mut list = vec![];
        self.layout_list.iter().for_each(|item| {
            list.push(
                wgpu::VertexBufferLayout {
                    array_stride: item.stride,
                    step_mode: item.step_mode,
                    attributes: item.list.as_slice(),
                }
            );
        });

        list
    }
}

#[derive(Debug, Clone, Hash)]
pub enum EVertexBufferRange {
    /// * (BufferRange, 使用大小)
    Updatable(RWBufferRange, u32, wgpu::BufferUsages),
    NotUpdatable(Arc<NotUpdatableBufferRange>, u32, u32),
}
impl EVertexBufferRange {
    pub fn buffer(&self) -> &Buffer {
        match self {
            EVertexBufferRange::Updatable(val, _, _) => val.buffer(),
            EVertexBufferRange::NotUpdatable(val, ..) => val.buffer(),
        }
    }
    pub fn size(&self) -> u32 {
        match self {
            EVertexBufferRange::Updatable(_, size, _) => *size,
            EVertexBufferRange::NotUpdatable(_val, start, end) => end - start,
        }
    }
    pub fn range(&self) -> Range<wgpu::BufferAddress> {
        match self {
            EVertexBufferRange::Updatable(val, size, _) => {
                Range { start: val.offset() as u64, end: (val.offset() + size) as u64 }
            },
            EVertexBufferRange::NotUpdatable(_val, start, end) => {
                Range { start: *start as u64, end: *end as u64 }
                // Range { start: 0 as u64, end: val.size() as u64 }
            },
        }
    }
    pub fn active_range(&self) -> Range<wgpu::BufferAddress> {
        match self {
            EVertexBufferRange::Updatable(val, size, _) => {
                Range { start: val.offset() as u64, end: (val.offset() + size) as u64 }
            },
            EVertexBufferRange::NotUpdatable(_val, start, end) => {
                Range { start: 0 as u64, end: (end - start) as u64 }
                // Range { start: *start as u64, end: *end as u64 }
            },
        }
    }
}
impl PartialEq for EVertexBufferRange {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EVertexBufferRange::Updatable(buffer, s0, e0), EVertexBufferRange::Updatable(buffer2, s1, e1)) => {
                buffer == buffer2 && s0 == s1 && e0 == e1
            },
            (EVertexBufferRange::NotUpdatable(buffer, s0, e0), EVertexBufferRange::NotUpdatable(buffer2, s1, e1)) => {
                buffer.as_ref() == buffer2.as_ref() && s0 == s1 && e0 == e1
            },
            _ => false,
        }
    }
}
impl Eq for EVertexBufferRange {
    fn assert_receiver_is_total_eq(&self) {}
}
impl Asset for EVertexBufferRange {
    type Key = IDAssetVertexBuffer;
}

impl Size for EVertexBufferRange {
    fn size(&self) -> usize {
        32
    }
}

pub struct VertexBufferAllocator {
    /// * 最小对齐尺寸
    base_size: u32,
    // /// * 最大对齐尺寸, 超过该尺寸的独立创建Buffer
    // max_base_size: u32,
    // block_size: u32,
    pool_slots: [FixedSizeBufferPool;Self::LEVEL_COUNT],
    pool_slots_for_index: [FixedSizeBufferPool;Self::LEVEL_COUNT],
    pool_count: usize,
    asset_mgr: Share<AssetMgr<AssetRWBuffer>>,
    asset_mgr_2: Share<AssetMgr<NotUpdatableBuffer>>,
    unupdatables: Vec<FixedSizeBufferPoolNotUpdatable>,
    unupdatables_for_index: Vec<FixedSizeBufferPoolNotUpdatable>,
}

impl Default for VertexBufferAllocator {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY, Self::DEFAULT_TIMEOUT)
    }
}

impl VertexBufferAllocator {
    pub const DEFAULT_CAPACITY: usize = 20 * 1024 * 1024;
    pub const DEFAULT_TIMEOUT: usize = 60 * 1000;

    /// * 每 level 间 对齐尺寸比值为 2
    pub const LEVEL_COUNT: usize = 16;
    /// * 最小对齐尺寸
    /// * 一个 2D 三角形 顶点坐标 + UV 坐标: 3 * (2 + 2) * 4 = 48
    /// * 一个 2D 三角形 顶点坐标 + Color: 3 * (2 + 3) * 4 = 60
    pub const BAE_SIZE: u32 = 64;
    /// * LEVEL_COUNT 对应的 最大对齐尺寸 - 
    /// * 一个顶点 (Pos + UV + UV2 + Color4 + Normal + Tangent + BoneWeight + BoneIndice) = (3 + 2 + 2 + 4 + 3 + 4 + 4) * 4 + 4 * 2 = 96
    /// * u16::MAX 个顶点 = 96 * 65536 = 6 * 1024 * 1024
    /// * LEVEL_COUNT = 16; MAX_BASE_SIZE = 64 * 2^16 = 4 * 1024 * 1024
    pub const MAX_BASE_SIZE: u32 = 64 * 2_i32.pow(Self::LEVEL_COUNT as u32) as u32;
    pub fn new(capacity: usize, timeout: usize) -> Self {
        let base_size = Self::BAE_SIZE;
        let level = Self::LEVEL_COUNT;
        // let max_base_size = Self::MAX_BASE_SIZE;
        let block_size = base_size * 1024;

        let usage = wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX;
        let pool_slots = [
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(00) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(01) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(02) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(03) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(04) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(05) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(06) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(07) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(08) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(09) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(10) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(11) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(12) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(13) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(14) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(15) as u32, usage),
        ];
        let usage = wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX;
        let pool_slots_for_index = [
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(00) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(01) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(02) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(03) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(04) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(05) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(06) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(07) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(08) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(09) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(10) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(11) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(12) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(13) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(14) as u32, usage),
            FixedSizeBufferPool::new(block_size, base_size * 2_i32.pow(15) as u32, usage),
        ];

        let asset_mgr = AssetMgr::<AssetRWBuffer>::new(GarbageEmpty(), false, capacity / 8 * 1, timeout);
        let asset_mgr_2 = AssetMgr::<NotUpdatableBuffer>::new(GarbageEmpty(), false, capacity / 8 * 7, timeout);

        Self {
            base_size,
            // block_size,
            pool_slots,
            pool_slots_for_index,
            pool_count: level,
            // max_base_size,
            asset_mgr,
            asset_mgr_2,
            unupdatables: vec![],
            unupdatables_for_index: vec![]
        }
    }
    pub fn create_updatable_buffer(&mut self, data: &[u8]) -> Option<EVertexBufferRange> {
        let size = data.len() as u32;
        let index = match self.pool_slots.binary_search_by(|v| { v.fixed_size.cmp(&size)  }) {
            Ok(index) => index,
            Err(index) => index,
        };

        if index < self.pool_count {
            if let Some(pool) = self.pool_slots.get_mut(index) {
                if let Some(range) = pool.allocate(&self.asset_mgr) {
                    range.write_data(0, data);
                    Some(EVertexBufferRange::Updatable(range, size, wgpu::BufferUsages::VERTEX))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn create_updatable_buffer_for_index(&mut self, data: &[u8]) -> Option<EVertexBufferRange> {
        let size = data.len() as u32;
        let index = match self.pool_slots_for_index.binary_search_by(|v| { v.fixed_size.cmp(&size)  }) {
            Ok(index) => index,
            Err(index) => index,
        };

        if index < self.pool_count {
            if let Some(pool) = self.pool_slots_for_index.get_mut(index) {
                if let Some(range) = pool.allocate(&self.asset_mgr) {
                    range.write_data(0, data);
                    Some(EVertexBufferRange::Updatable(range, size, wgpu::BufferUsages::INDEX))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn update_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.pool_slots.iter_mut().for_each(|pool| {
            pool.write_buffer(device, queue);
        });
        self.pool_slots_for_index.iter_mut().for_each(|pool| {
            pool.write_buffer(device, queue);
        });
    }
    ///
    /// * `old` 仅当更新 Instance 实例化buffer时使用, 此时 NotUpdatableBufferRange 在逻辑上是唯一的,没有共用
    pub fn create_not_updatable_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue, data: &[u8], old_single_used: Option<&NotUpdatableBufferRange>) -> Option<EVertexBufferRange> {
        if let Some(buffer) = self.create_not_updatable_buffer_pre(device, queue, data, old_single_used) {
            Some(EVertexBufferRange::NotUpdatable(buffer, 0, data.len() as u32))
        } else {
            None
        }
    }

    /// * `old` 仅当更新 Instance 实例化buffer时使用, 此时 NotUpdatableBufferRange 在逻辑上是唯一的,没有共用
    pub fn create_not_updatable_buffer_pre(&mut self, device: &RenderDevice, queue: &RenderQueue, data: &[u8], old_single_used: Option<&NotUpdatableBufferRange>) -> Option<Arc<NotUpdatableBufferRange>> {
        let size = data.len() as u32;

        // 如果传入旧 NotUpdatableBufferRange, 且 对应 buffer 大小足够存放新数据 则重复利用该 NotUpdatableBufferRange 中的 buffer
        if let Some(old) = old_single_used {
            if old.id_buffer.size >= size {
                let buffer = old.unuse();
                if let Some(wbuffer) = &buffer.0 {
                    wbuffer.write_buffer(queue, data);
                    let buffer = NotUpdatableBufferRange {
                        used_size: data.len() as u32,
                        id_buffer: old.id_buffer,
                        buffer: buffer,
                        usage: old.usage,
                        unused: false,
                    };
                    return Some(Arc::new(buffer));
                }
            }
        }

        let mut level = 0;
        let mut level_size = self.base_size;
        loop {
            if level_size >= size {
                break;
            }
            level_size *= 2;
            level += 1;

            // 基础为 64 = 2^6, u32 还可以有 25 个 level - 最大 2048 M
            if level > 25 {
                return None;
            }
        }

        let old_count = self.unupdatables.len();
        let new_count = level + 1;
        if old_count < new_count {
            let usage = wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX;
            for level in old_count..new_count {
                self.unupdatables.push(
                    FixedSizeBufferPoolNotUpdatable::new(self.base_size * 2_i32.pow(level as u32) as u32, usage)
                );
            }
        }
        // log::info!("size: {}, level: {}, old_count: {}, new: {}", size, level, old_count, new_count);

        if let Some(range) = self.unupdatables.get_mut(level).unwrap().allocate(&self.asset_mgr_2, device, queue, data) {
            Some(Arc::new(range))
        } else {
            None
        }
    }

    pub fn create_not_updatable_buffer_for_index(&mut self, device: &RenderDevice, queue: &RenderQueue, data: &[u8]) -> Option<EVertexBufferRange> {
        let size = data.len() as u32;
        let mut level = 0;
        let mut level_size = self.base_size;
        loop {
            if level_size >= size {
                break;
            }
            level_size *= 2;
            level += 1;

            // 基础为 64 = 2^6, u32 还可以有 25 个 level - 最大 2048 M
            if level > 25 {
                return None;
            }
        }

        let old_count = self.unupdatables_for_index.len();
        let new_count = level + 1;
        if old_count < new_count {
            let usage = wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX;
            for level in old_count..new_count {
                self.unupdatables_for_index.push(
                    FixedSizeBufferPoolNotUpdatable::new(self.base_size * 2_i32.pow(level as u32) as u32, usage)
                );
                // log::warn!("New Buffer {:?}", usage);
            }
        }
        // log::info!("size: {}, level: {}, old_count: {}, new: {}", size, level, old_count, new_count);

        if let Some(range) = self.unupdatables_for_index.get_mut(level).unwrap().allocate(&self.asset_mgr_2, device, queue, data) {
            Some(EVertexBufferRange::NotUpdatable(Arc::new(range), 0, data.len() as u32))
        } else {
            None
        }
    }
}

pub struct FixedSizeBufferPoolNotUpdatable {
    /// * 大内存块列表
    buffers: Vec<Arc<UseNotUpdatableBuffer>>,
    /// * 此处分配出去的每个Buffer大小均为 block_size
    pub(crate) block_size: u32,
	#[allow(dead_code)]
    mutex: ShareMutex<()>,
    usage: wgpu::BufferUsages,
}
impl FixedSizeBufferPoolNotUpdatable {
    /// * `block_size` 大内存块的基础尺寸
    /// * `fixed_size` 目标区间尺寸
    pub fn new(
        block_size: u32,
        usage: wgpu::BufferUsages,
    ) -> Self {
        Self {
            buffers: vec![],
            block_size,
            mutex: ShareMutex::new(()),
            usage
        }
    }
    pub fn allocate(&mut self, asset_mgr: &Share<AssetMgr<NotUpdatableBuffer>>, device: &RenderDevice, queue: &RenderQueue, data: &[u8]) -> Option<NotUpdatableBufferRange> {
        let len = self.buffers.len();
        let mut key_buffer = None;
        // 寻找可用区间
        for i in 0..len {
            if let Some(use_buffer) = self.buffers.get(i) {
                // 有数据的情况一定是正在使用的
                if let Some(_) = &use_buffer.0 {
                    //
                } else {
                    key_buffer = Some(IDNotUpdatableBuffer { index: i as u32, size: self.block_size, usage: self.usage },);
                }
            }
        }

        // 寻找 是否有缓存 块
        let key_buffer = if let Some(key_buffer) = key_buffer {
            if let Some(asset_buffer) = asset_mgr.get(&key_buffer) {
                let use_buffer = UseNotUpdatableBuffer(Some(asset_buffer.clone()));
                let use_buffer = Arc::new(use_buffer);
                self.buffers[key_buffer.index as usize] = use_buffer.clone();

                let buffer = asset_buffer;
                buffer.flag(true);
                buffer.write_buffer(queue, data);
                return Some(
                    NotUpdatableBufferRange {
                        used_size: data.len() as u32,
                        id_buffer: key_buffer.clone(),
                        buffer: use_buffer,
                        usage: self.usage,
                        unused: false,
                    }
                );
            } else {
                key_buffer
            }
        } else {
            self.buffers.push(Arc::new(UseNotUpdatableBuffer(None)));
            IDNotUpdatableBuffer { index: len as u32, size: self.block_size, usage: self.usage }
        };

        // 创建块
        let buffer = NotUpdatableBuffer::new(device, self.block_size, self.usage);
        buffer.write_buffer(queue, data);
        if let Ok(asset_buffer) = asset_mgr.insert(key_buffer, buffer) {
            let use_buffer = UseNotUpdatableBuffer(Some(asset_buffer.clone()));
            let use_buffer = Arc::new(use_buffer);
            self.buffers[key_buffer.index as usize] = use_buffer.clone();
            return Some(
                NotUpdatableBufferRange {
                    used_size: data.len() as u32,
                    id_buffer: key_buffer.clone(),
                    buffer: use_buffer,
                    usage: self.usage,
                    unused: false,
                }
            );
        } else {
            return None;
        }
    }
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct IDNotUpdatableBuffer {
    /// 在 FixedSizeBufferPoolNotUpdatable 中是第几个
    pub index: u32,
    /// Buffer 大小 也是 所属 FixedSizeBufferPoolNotUpdatable 的 block_size
    pub size: u32,
    pub usage: wgpu::BufferUsages,
}

/// 在 FixedSizeBufferPoolNotUpdatable 中保存以记录 NotUpdatableBuffer
/// * FixedSizeBufferPoolNotUpdatable 中分配并包装为 NotUpdatableBufferRange
/// * 当 NotUpdatableBufferRange 释放时 置 UseNotUpdatableBuffer 内容为 None 以释放 Handle<NotUpdatableBuffer>
pub struct UseNotUpdatableBuffer(Option<Handle<NotUpdatableBuffer>>);
impl UseNotUpdatableBuffer {
    pub fn none(&self) -> Option<Handle<NotUpdatableBuffer>> {
        unsafe {
            let temp = &mut *(self as *const Self as usize as *mut Self);
            let result = temp.0.clone();
            temp.0 = None;
            result
        }
    }
}

pub struct NotUpdatableBuffer(Buffer, u32, bool, wgpu::BufferUsages);
impl Asset for NotUpdatableBuffer {
    type Key = IDNotUpdatableBuffer;
}
impl Size for NotUpdatableBuffer {
    fn size(&self) -> usize {
        self.1 as usize
    }
}
impl NotUpdatableBuffer {
    /// * `size` 请求的 buffer 大小 
    pub fn new(device: &RenderDevice, size: u32, usage: wgpu::BufferUsages) -> Self {
        let mut data = vec![];
        for _ in 0..size {
            data.push(0)
        }
        let buffer = device.create_buffer_with_data(
            &BufferInitDescriptor {
                label: None,
                contents: &data,
                usage,
            }
        );

        Self(buffer, size, true, usage)
    }
    /// 写入Buffer 数据
    /// * `data` data 长度 不应超过 self.size()
    pub(crate) fn write_buffer(&self, queue: &RenderQueue, data: &[u8]) {
        // let mut temp = vec![];
        // data.iter().for_each(|v| { temp.push(*v) });
        // for _ in data.len()..self.size() {
        //     temp.push(0);
        // }
        // queue.write_buffer(&self.0, 0, &temp);
        if data.len() > 0 {
            queue.write_buffer(&self.0, 0, data);
        }
    }
    pub fn flag(&self, val: bool) {
        unsafe {
            let temp = &mut *(self as *const Self as usize as *mut Self);
            temp.2 = val;
        }
    }
}

/// NotUpdatableBufferRange 在外部应当具有唯一性, 但由于应用层使用限制, 可能被包装为 Arc, 所以 只能是逻辑上保证维护其唯一性,
/// * 
pub struct NotUpdatableBufferRange {
    used_size: u32,
    id_buffer: IDNotUpdatableBuffer,
    buffer: Arc<UseNotUpdatableBuffer>,
    usage: wgpu::BufferUsages,
    unused: bool,
}
impl Debug for NotUpdatableBufferRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotUpdatableBufferRange").field("used_size", &self.used_size).field("id_buffer", &self.id_buffer).finish()
    }
}
impl NotUpdatableBufferRange {
    pub fn buffer(&self) -> &Buffer {
        &self.buffer.0.as_ref().unwrap().0
    }
    pub fn size(&self) -> u32 {
        self.used_size
    }
    pub fn offset(&self) -> u32 {
        0
    }
    pub fn id_buffer(&self) -> IDNotUpdatableBuffer {
        self.id_buffer
    }
    pub fn unuse(&self) -> Arc<UseNotUpdatableBuffer> {
        unsafe {
            let temp = &mut *(self as *const Self as usize as *mut Self);
            temp.unused = true;
            temp.buffer.clone()
        }
    }
}
impl Drop for NotUpdatableBufferRange {
    fn drop(&mut self) {
        if !self.unused {
            self.buffer.none();
        }
    }
}
impl Hash for NotUpdatableBufferRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id_buffer.hash(state);
        self.usage.hash(state);
    }
}
impl PartialEq for NotUpdatableBufferRange {
    fn eq(&self, other: &Self) -> bool {
        self.id_buffer == other.id_buffer && self.usage == other.usage
    }
}
impl Eq for NotUpdatableBufferRange {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}



// #[cfg(test)]
// mod vertex_code_test {
//     use std::hash::Hash;

//     use render_data_container::KeyVertexBuffer;
//     use render_shader::shader::TShaderSetCode;

//     use crate::vertex_buffer_desc::{VertexBufferDesc, EInstanceKind};

//     use super::{KeyVertexBuffer, VertexAttribute, VertexBufferLayouts};

//     /// .
//     #[test]
//     fn test() {
//         let meshdes = vec![
//             VertexBufferDesc {
//                 key: KeyVertexBuffer::from("a1"),
//                 range: None,
//                 kind: EInstanceKind::None,
//                 attrs: vec![
//                     VertexAttribute { kind: super::EVertexDataKind::Position, format: wgpu::VertexFormat::Float32x3 },
//                     VertexAttribute { kind: super::EVertexDataKind::Normal, format: wgpu::VertexFormat::Float32x3 },
//                     VertexAttribute { kind: super::EVertexDataKind::UV, format: wgpu::VertexFormat::Float32x2 }
//                 ],
//                 step_mode: wgpu::VertexStepMode::Vertex,
//             },
//             VertexBufferDesc {
//                 key: KeyVertexBuffer::from("a0"),
//                 range: None,
//                 kind: EInstanceKind::None,
//                 attrs: vec![
//                     VertexAttribute { kind: super::EVertexDataKind::Color4, format: wgpu::VertexFormat::Float32x4 }
//                 ],
//                 step_mode: wgpu::VertexStepMode::Instance,
//             }
//         ];

//         let reslayouts = VertexBufferLayouts::from(&meshdes);
//         let keyshader_attribute = reslayouts.as_key_shader_from_attributes();
        
//         println!("{}", keyshader_attribute.vs_define_code());
//         println!("{}", keyshader_attribute.vs_running_code());
//         println!("{:?}", reslayouts.layouts());
//         println!("{:?}", reslayouts.size);
//     }
// }