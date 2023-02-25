use std::{ops::Range, mem::{replace, size_of}, hash::Hash, sync::Arc, fmt::Debug};

use derive_deref_rs::Deref;
use pi_assets::{asset::{Asset, GarbageEmpty, Handle}, mgr::AssetMgr};
use pi_atom::Atom;
use pi_share::{Share, ShareMutex};

use crate::rhi::{device::RenderDevice, RenderQueue, shader::WriteBuffer, buffer::Buffer};

use super::{attributes::{VertexAttribute, EVertexDataKind, ShaderAttribute, TAsWgpuVertexAtribute, KeyShaderFromAttributes}, vertex_buffer_desc::VertexBufferDesc, vertex_format::TVertexFormatByteSize, buffer::{FixedSizeBufferPool, AssetRWBuffer, RWBufferRange}, ASSET_SIZE_FOR_UNKOWN};


pub type KeyVertexBuffer = Atom;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EVertexBufferRange {
    Updatable(RWBufferRange, u32),
    NotUpdatable(NotUpdatableBufferRange),
}
impl EVertexBufferRange {
    pub fn buffer(&self) -> &Buffer {
        match self {
            EVertexBufferRange::Updatable(val, _) => val.buffer(),
            EVertexBufferRange::NotUpdatable(val) => val.buffer(),
        }
    }
    pub fn size(&self) -> u32 {
        match self {
            EVertexBufferRange::Updatable(_, size) => *size,
            EVertexBufferRange::NotUpdatable(val) => val.used_size,
        }
    }
    pub fn range(&self) -> Range<u64> {
        match self {
            EVertexBufferRange::Updatable(val, size) => {
                Range { start: val.offset() as u64, end: (val.offset() + size) as u64 }
            },
            EVertexBufferRange::NotUpdatable(val) => {
                Range { start: 0, end: val.used_size as u64 }
            },
        }
    }
}
impl Asset for EVertexBufferRange {
    type Key = KeyVertexBuffer;
    fn size(&self) -> usize {
        32
    }
}

pub struct VertexBufferAllocator {
    /// * 最小对齐尺寸
    base_size: u32,
    /// * 最大对齐尺寸, 超过该尺寸的独立创建Buffer
    max_base_size: u32,
    block_size: u32,
    pool_slots: [FixedSizeBufferPool;Self::LEVEL_COUNT],
    pool_count: usize,
    asset_mgr: Share<AssetMgr<AssetRWBuffer>>,
    asset_mgr_2: Share<AssetMgr<NotUpdatableBuffer>>,
    unupdatables: Vec<FixedSizeBufferPoolNotUpdatable>,
    asset_mgr_vb: Share<AssetMgr<EVertexBufferRange>>,
}
impl VertexBufferAllocator {
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
    pub fn new() -> Self {
        let base_size = Self::BAE_SIZE;
        let level = Self::LEVEL_COUNT;
        let max_base_size = Self::MAX_BASE_SIZE;
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

        let asset_mgr = AssetMgr::<AssetRWBuffer>::new(GarbageEmpty(), false, 16 * 1024 * 1024, 60 * 1000);
        let asset_mgr_2 = AssetMgr::<NotUpdatableBuffer>::new(GarbageEmpty(), false, 32 * 1024 * 1024, 60 * 1000);
        let asset_mgr_vb = AssetMgr::<EVertexBufferRange>::new(GarbageEmpty(), false, 16 * 1024, 10 * 1000);

        Self {
            base_size,
            block_size,
            pool_slots,
            pool_count: level,
            max_base_size,
            asset_mgr,
            asset_mgr_2,
            asset_mgr_vb,
            unupdatables: vec![],
        }
    }
    pub fn get(&self, key: &KeyVertexBuffer) -> Option<Handle<EVertexBufferRange>> {
        self.asset_mgr_vb.get(key)
    }
    pub fn create_updatable_buffer(&mut self, key: KeyVertexBuffer, data: &[u8]) -> Option<Handle<EVertexBufferRange>> {
        let size = data.len() as u32;
        let index = match self.pool_slots.binary_search_by(|v| { v.fixed_size.cmp(&size)  }) {
            Ok(index) => index,
            Err(index) => index,
        };

        if index < self.pool_count {
            if let Some(pool) = self.pool_slots.get_mut(index) {
                if let Some(range) = pool.allocate(&self.asset_mgr) {
                    range.write_data(0, data);
                    self.asset_mgr_vb.insert(key, EVertexBufferRange::Updatable(range, size))
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
    }
    pub fn create_not_updatable_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue, key: KeyVertexBuffer, data: &[u8]) -> Option<Handle<EVertexBufferRange>> {
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

        let old_count = self.pool_slots.len();
        let new_count = level + 1;
        if old_count < new_count {
            let usage = wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX;
            for level in old_count..new_count {
                self.unupdatables.push(
                    FixedSizeBufferPoolNotUpdatable::new(self.base_size * 2_i32.pow(level as u32) as u32, usage)
                );
            }
        }

        if let Some(range) = self.unupdatables.get_mut(level).unwrap().allocate(&self.asset_mgr_2, device, queue, data) {
            self.asset_mgr_vb.insert(key, EVertexBufferRange::NotUpdatable(range))
        } else {
            None
        }
    }

}

pub(crate) struct FixedSizeBufferPoolNotUpdatable {
    /// * 大内存块列表 (第i个的尺寸为 i*block_size)
    buffers: Vec<Arc<UseNotUpdatableBuffer>>,
    /// * 目标尺寸
    pub(crate) block_size: u32,
    mutex: ShareMutex<()>,
    usage: wgpu::BufferUsages,
}
impl FixedSizeBufferPoolNotUpdatable {
    /// * `block_size` 大内存块的基础尺寸
    /// * `fixed_size` 目标区间尺寸
    pub(crate) fn new(
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
    pub(crate) fn allocate(&mut self, asset_mgr: &Share<AssetMgr<NotUpdatableBuffer>>, device: &RenderDevice, queue: &RenderQueue, data: &[u8]) -> Option<NotUpdatableBufferRange> {
        let len = self.buffers.len();
        let mut key_buffer = None;
        // 寻找可用区间
        for i in 0..len {
            if let Some(use_buffer) = self.buffers.get(i) {
                if let Some(asset_buffer) = &use_buffer.0 {
                    let _clock = self.mutex.lock();
                    let buffer = unsafe {
                        &mut *(Handle::as_ptr(asset_buffer) as usize as *mut NotUpdatableBuffer)
                    };

                    buffer.write_buffer(queue, data);
                    return Some(
                        NotUpdatableBufferRange {
                            used_size: data.len() as u32,
                            id_buffer: IDNotUpdatableBuffer { index: i as u32, size: self.block_size },
                            buffer: use_buffer.clone()
                        }
                    );
                } else {
                    key_buffer = Some(IDNotUpdatableBuffer { index: i as u32, size: self.block_size },);
                }
            }
        }

        // 寻找 是否有缓存 块
        let key_buffer = if let Some(key_buffer) = key_buffer {
            if let Some(asset_buffer) = asset_mgr.get(&key_buffer) {
                let use_buffer = UseNotUpdatableBuffer(Some(asset_buffer.clone()));
                let use_buffer = Arc::new(use_buffer);
                self.buffers[key_buffer.index as usize] = use_buffer.clone();

                let buffer = unsafe {
                    &mut *(Handle::as_ptr(&asset_buffer) as usize as *mut NotUpdatableBuffer)
                };
                buffer.write_buffer(queue, data);
                return Some(
                    NotUpdatableBufferRange {
                        used_size: data.len() as u32,
                        id_buffer: key_buffer.clone(),
                        buffer: use_buffer
                    }
                );
            } else {
                key_buffer
            }
        } else {
            self.buffers.push(Arc::new(UseNotUpdatableBuffer(None)));
            IDNotUpdatableBuffer { index: len as u32, size: self.block_size }
        };

        // 创建块
        let buffer = NotUpdatableBuffer::new(device, self.block_size, self.usage);
        buffer.write_buffer(queue, data);
        if let Some(asset_buffer) = asset_mgr.insert(key_buffer, buffer) {
            let use_buffer = UseNotUpdatableBuffer(Some(asset_buffer.clone()));
            let use_buffer = Arc::new(use_buffer);
            self.buffers[key_buffer.index as usize] = use_buffer.clone();
            return Some(
                NotUpdatableBufferRange {
                    used_size: data.len() as u32,
                    id_buffer: key_buffer.clone(),
                    buffer: use_buffer
                }
            );
        } else {
            return None;
        }
    }
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct IDNotUpdatableBuffer {
    pub index: u32,
    pub size: u32,
}

pub struct UseNotUpdatableBuffer(Option<Handle<NotUpdatableBuffer>>);

pub struct NotUpdatableBuffer(Buffer, u32);
impl Asset for NotUpdatableBuffer {
    type Key = IDNotUpdatableBuffer;
    fn size(&self) -> usize {
        self.1 as usize
    }
}
impl NotUpdatableBuffer {
    pub fn new(device: &RenderDevice, size: u32, usage: wgpu::BufferUsages) -> Self {
        let buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: None,
                size: size as u64,
                usage,
                mapped_at_creation: false,
            }
        );

        Self(buffer, size)
    }
    pub(crate) fn write_buffer(&self, queue: &RenderQueue, data: &[u8]) {
        queue.write_buffer(&self.0, 0, data);
    }
}

#[derive(Clone)]
pub struct NotUpdatableBufferRange {
    used_size: u32,
    id_buffer: IDNotUpdatableBuffer,
    buffer: Arc<UseNotUpdatableBuffer>,
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
}
impl Drop for NotUpdatableBufferRange {
    fn drop(&mut self) {
        let buffer = unsafe {
            &mut *(Arc::as_ptr(&self.buffer) as usize as *mut UseNotUpdatableBuffer)
        };
        buffer.0 = None;
    }
}
impl Hash for NotUpdatableBufferRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id_buffer.hash(state);
    }
}
impl PartialEq for NotUpdatableBufferRange {
    fn eq(&self, other: &Self) -> bool {
        self.id_buffer == other.id_buffer
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