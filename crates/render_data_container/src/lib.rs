use std::{ops::{Deref, Range}, fmt::Debug, sync::Arc};

use bytemuck::Pod;
use pi_assets::asset::{Asset, Handle};
use pi_atom::Atom;
use pi_hash::XHashMap;
use pi_share::Share;
use pi_slotmap::DefaultKey;
use render_core::rhi::{pipeline::{RenderPipeline, VertexBufferLayout}, buffer::BufferSlice};
use vertex_layout_key::KeyVertexLayouts;
use wgpu::util::DeviceExt;

use nalgebra::{RealField, Vector2 as NVector2, Vector3 as NVector3, Dim, SimdValue, Vector4 as NVector4, UnitQuaternion as NQuaternion, 
    Matrix4 as NMatrix4, SimilarityMatrix3 as NSimilarityMatrix3, Translation3 as NTranslation3, Transform3 as NTransform3,
    Affine3 as NAffine3, Projective3 as NProjective3, Isometry3 as NIsometry3, Rotation3 as NRotation3,
    Matrix2 as NMatrix2, Point3 as NPoint3, Perspective3 as NPerspective3, Transform as NTransform
};

pub mod vertex_layout_key;

pub type Number = f32;

pub type Vector2 = NVector2<Number>;
pub type Vector3 = NVector3<Number>;
pub type Vector4 = NVector4<Number>;
pub type Color3 = NVector3<Number>;
pub type Color4 = NVector4<Number>;
/// 单位四元数旋转
pub type Quaternion = NQuaternion<Number>;
/// 位移
pub type Translation3 = NTranslation3<Number>;
/// 旋转矩阵
pub type Rotation3 = NRotation3<Number>;
/// 等距变换 - 旋转&位移 - 相机节点
pub type Isometry3 = NIsometry3<Number>;
/// 相似变换 - 旋转&位移&缩放
pub type SimilarityMatrix3 = NSimilarityMatrix3<Number>;
/// 仿射变换
pub type Affine3 = NAffine3<Number>;
/// 投影变换
pub type Projective3 = NProjective3<Number>;
pub type Matrix     = NMatrix4<Number>;
pub type Matrix2    = NMatrix2<Number>;
pub type Point3 = NPoint3<Number>;
pub type Perspective3 = NPerspective3<Number>;


pub trait FKey: Clone + PartialEq + Eq + std::hash::Hash {}
impl FKey for DefaultKey {}
impl FKey for u8 {}
impl FKey for u16 {}
impl FKey for u32 {}
impl FKey for u64 {}
impl FKey for u128 {}
impl FKey for usize {}
impl FKey for i8 {}
impl FKey for i16 {}
impl FKey for i32 {}
impl FKey for i64 {}
impl FKey for i128 {}
impl FKey for isize {}
impl FKey for &str {}

pub trait FContainer<T: Clone, K: FKey> {
    fn insert(&mut self, value: T) -> K;
    fn remove(&mut self, key: &K) -> Option<T>;
    fn get(&mut self, key: &K) -> Option<&T>;
    fn get_mut(&mut self, key: &K) -> Option<&mut T>;
}

pub trait TextureID: Clone + PartialEq + Eq + std::hash::Hash {}
impl TextureID for DefaultKey {}
impl TextureID for u8 {}
impl TextureID for u16 {}
impl TextureID for u32 {}
impl TextureID for u64 {}
impl TextureID for usize {}
impl TextureID for u128 {}
impl TextureID for i8 {}
impl TextureID for i16 {}
impl TextureID for i32 {}
impl TextureID for i64 {}
impl TextureID for i128 {}
impl TextureID for isize {}
impl TextureID for &str {}

pub trait TexturePool<TID: TextureID> {
    fn get(&self, key: TID) -> Option<& wgpu::TextureView>;
}

pub trait TMaterialKey: Clone + PartialEq + Eq + std::hash::Hash {}
impl TMaterialKey for DefaultKey {}
impl TMaterialKey for u8 {}
impl TMaterialKey for u16 {}
impl TMaterialKey for u32 {}
impl TMaterialKey for u64 {}
impl TMaterialKey for u128 {}
impl TMaterialKey for usize {}
impl TMaterialKey for i8 {}
impl TMaterialKey for i16 {}
impl TMaterialKey for i32 {}
impl TMaterialKey for i64 {}
impl TMaterialKey for i128 {}
impl TMaterialKey for isize {}
impl TMaterialKey for &str {}

pub trait TMaterialBlockKindKey: Clone + PartialEq + Eq + std::hash::Hash {}
impl TMaterialBlockKindKey for DefaultKey {}
impl TMaterialBlockKindKey for u8 {}
impl TMaterialBlockKindKey for u16 {}
impl TMaterialBlockKindKey for u32 {}
impl TMaterialBlockKindKey for u64 {}
impl TMaterialBlockKindKey for u128 {}
impl TMaterialBlockKindKey for usize {}
impl TMaterialBlockKindKey for i8 {}
impl TMaterialBlockKindKey for i16 {}
impl TMaterialBlockKindKey for i32 {}
impl TMaterialBlockKindKey for i64 {}
impl TMaterialBlockKindKey for i128 {}
impl TMaterialBlockKindKey for isize {}
impl TMaterialBlockKindKey for &str {}

pub trait TVertexDataKindKey: Clone + PartialEq + Eq + std::hash::Hash {}
impl TVertexDataKindKey for DefaultKey {}
impl TVertexDataKindKey for u8 {}
impl TVertexDataKindKey for u16 {}
impl TVertexDataKindKey for u32 {}
impl TVertexDataKindKey for u64 {}
impl TVertexDataKindKey for usize {}
impl TVertexDataKindKey for u128 {}
impl TVertexDataKindKey for i8 {}
impl TVertexDataKindKey for i16 {}
impl TVertexDataKindKey for i32 {}
impl TVertexDataKindKey for i64 {}
impl TVertexDataKindKey for i128 {}
impl TVertexDataKindKey for isize {}
impl TVertexDataKindKey for &str {}

pub trait TVertexBufferKindKey: Clone + PartialEq + Eq + std::hash::Hash {}
impl TVertexBufferKindKey for DefaultKey {}
impl TVertexBufferKindKey for u8 {}
impl TVertexBufferKindKey for u16 {}
impl TVertexBufferKindKey for u32 {}
impl TVertexBufferKindKey for u64 {}
impl TVertexBufferKindKey for usize {}
impl TVertexBufferKindKey for u128 {}
impl TVertexBufferKindKey for i8 {}
impl TVertexBufferKindKey for i16 {}
impl TVertexBufferKindKey for i32 {}
impl TVertexBufferKindKey for i64 {}
impl TVertexBufferKindKey for i128 {}
impl TVertexBufferKindKey for isize {}
impl TVertexBufferKindKey for &str {}

pub trait TVertexBufferID: Clone + PartialEq + Eq + std::hash::Hash {}
impl TVertexBufferID for DefaultKey {}
impl TVertexBufferID for u8 {}
impl TVertexBufferID for u16 {}
impl TVertexBufferID for u32 {}
impl TVertexBufferID for u64 {}
impl TVertexBufferID for usize {}
impl TVertexBufferID for u128 {}
impl TVertexBufferID for i8 {}
impl TVertexBufferID for i16 {}
impl TVertexBufferID for i32 {}
impl TVertexBufferID for i64 {}
impl TVertexBufferID for i128 {}
impl TVertexBufferID for isize {}
impl TVertexBufferID for &str {}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EVertexDataFormat {
    U8,
    U16,
    U32,
    I32,
    F32,
    F64,
}
impl EVertexDataFormat {
    pub fn size(&self) -> usize {
        match self {
            EVertexDataFormat::U8 => 1,
            EVertexDataFormat::U16 => 2,
            EVertexDataFormat::U32 => 4,
            EVertexDataFormat::I32 => 4,
            EVertexDataFormat::F32 => 4,
            EVertexDataFormat::F64 => 8,
        }
    }
}

pub trait IndexFormatBytes {
    fn bytes(&self) -> wgpu::BufferAddress;
}
impl IndexFormatBytes for wgpu::IndexFormat {
    fn bytes(&self) -> wgpu::BufferAddress {
        match self {
            wgpu::IndexFormat::Uint16 => 2,
            wgpu::IndexFormat::Uint32 => 4,
        }
    }
}

#[derive(Debug, Default)]
pub struct VertexBufferPool {
    pub map: XHashMap<KeyVertexBuffer, VertexBuffer>,
}

pub trait TAttributeMeta {
    fn format(&self) -> wgpu::VertexFormat;
    fn offset(&self) -> wgpu::BufferAddress;
    fn shader_location(&self) -> u32;
    fn attribute(&self) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: self.format(),
            offset: self.offset(),
            shader_location: self.shader_location(),
        }
    }
}

pub trait TIndicesMeta {
    fn format(&self) -> wgpu::IndexFormat;
}

#[derive(Debug)]
pub struct VertexBuffer {
    dirty: bool,
    resize: bool,
    kind: EVertexDataFormat,
    updateable: bool,
    as_indices: bool,
    u8: Vec<u8>,
    u16: Vec<u16>,
    u32: Vec<u32>,
    i32: Vec<i32>,
    f32: Vec<f32>,
    f64: Vec<f64>,
    _size: usize,
    buffer: Option<Share<wgpu::Buffer>>,
}
impl VertexBuffer {
    pub fn new(updateable: bool, kind: EVertexDataFormat, as_indices: bool) -> Self {
        Self {
            dirty: true,
            resize: true,
            kind,
            as_indices,
            updateable,
            u8: vec![],
            u16: vec![],
            u32: vec![],
            i32: vec![],
            f32: vec![],
            f64: vec![],
            _size: 0,
            buffer: None,
        }
    }
    pub fn reset(&mut self) -> bool {
        if self.updateable {
            self.u8  = vec![];
            self.u16 = vec![];
            self.u32 = vec![];
            self.i32 = vec![];
            self.f32 = vec![];
            self.f64 = vec![];
            self._size = 0;
            self.dirty = true;
            true
        } else {
            false
        }
    }
    pub fn update_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.resize == false {
            if let Some(buffer) = self.buffer.as_ref() {
                if self.updateable && self.dirty {
                    match self.kind {
                        EVertexDataFormat::U8 => {
                            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.u8 ))
                        },
                        EVertexDataFormat::U16 => {
                            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.u16))
                        },
                        EVertexDataFormat::U32 => {
                            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.u32))
                        },
                        EVertexDataFormat::I32 => {
                            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.i32))
                        },
                        EVertexDataFormat::F32 => {
                            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.f32))
                        },
                        EVertexDataFormat::F64 => {
                            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.f64))
                        },
                    }

                    self.dirty = false;
                }
            }
        } else {
            let usage = if self.as_indices { wgpu::BufferUsages::INDEX } else { wgpu::BufferUsages::VERTEX };
            let usage = if self.updateable {
                usage | wgpu::BufferUsages::COPY_DST
            } else {
                usage
            };
            self.buffer = match self.kind {
                EVertexDataFormat::U8 => {
                    Some( Share::from(device.create_buffer_init( &wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&self.u8), usage, } )) )
                },
                EVertexDataFormat::U16 => {
                    println!("{:?}", self.u16);
                    Some( Share::from(device.create_buffer_init( &wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&self.u16), usage, } )) )
                },
                EVertexDataFormat::U32 => {
                    Some( Share::from(device.create_buffer_init( &wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&self.u32), usage, } )) )
                },
                EVertexDataFormat::I32 => {
                    Some( Share::from(device.create_buffer_init( &wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&self.i32), usage, } )) )
                },
                EVertexDataFormat::F32 => {
                    println!("{:?}", self.f32);
                    Some( Share::from(device.create_buffer_init( &wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&self.f32), usage, } )) )
                },
                EVertexDataFormat::F64 => {
                    Some( Share::from(device.create_buffer_init( &wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&self.f64), usage, } )) )
                },
            };
            
            self.dirty = false;
            self.resize = false;
        }
    }
    pub fn uninstall_buffer(&mut self) {
        self.buffer = None;
    }
    pub fn update_u8(&mut self, data: &[u8], offset: usize) -> bool {
        if (self.updateable || self.buffer.is_none()) && self.kind == EVertexDataFormat::U8 {
            let size = update(&mut self.u8, data, offset);
            self.resize = size > self._size;
            self._size = size;
            self.dirty = true;
    
            true
        } else {
            false
        }
    }
    pub fn update_u16(&mut self, data: &[u16], offset: usize) -> bool {
        if (self.updateable || self.buffer.is_none()) && self.kind == EVertexDataFormat::U16 {
            let size = update(&mut self.u16, data, offset);
            self.resize = size > self._size;
            self._size = size;
            self.dirty = true;
    
            true
        } else {
            false
        }
    }
    pub fn update_u32(&mut self, data: &[u32], offset: usize) -> bool {
        if (self.updateable || self.buffer.is_none()) && self.kind == EVertexDataFormat::U32 {
            let size = update(&mut self.u32, data, offset);
            self.resize = size > self._size;
            self._size = size;
            self.dirty = true;
    
            true
        } else {
            false
        }
    }
    pub fn update_i32(&mut self, data: &[i32], offset: usize) -> bool {
        if (self.updateable || self.buffer.is_none()) && self.kind == EVertexDataFormat::I32 {
            let size = update(&mut self.i32, data, offset);
            self.resize = size > self._size;
            self._size = size;
            self.dirty = true;
    
            true
        } else {
            false
        }
    }
    pub fn update_f32(&mut self, data: &[f32], offset: usize) -> bool {
        if (self.updateable || self.buffer.is_none()) && self.kind == EVertexDataFormat::F32 {
            let size = update(&mut self.f32, data, offset);
            self.resize = size > self._size;
            self._size = size;
            self.dirty = true;
    
            true
        } else {
            false
        }
    }
    pub fn update_f64(&mut self, data: &[f64], offset: usize) -> bool {
        if (self.updateable || self.buffer.is_none()) && self.kind == EVertexDataFormat::F64 {
            let size = update(&mut self.f64, data, offset);
            self.resize = size > self._size;
            self._size = size;
            self.dirty = true;
    
            true
        } else {
            false
        }
    }
    pub fn get_buffer(&self) -> Option<Share<&wgpu::Buffer>> {
        match self.buffer.as_ref() {
            Some(buffer) => Some(Share::from(buffer.as_ref())),
            None => { None },
        }
    }
    pub fn size(&self) -> usize {
        self._size * self.kind.size()
    }
}

// For Asset
pub type KeyVertexBuffer = pi_atom::Atom;
impl Asset for VertexBuffer {
    type Key = KeyVertexBuffer;
    fn size(&self) -> usize {
        self._size * self.kind.size()
    }
}

pub trait TVertexBufferMeta {
    const DATA_FORMAT: EVertexDataFormat;
    const STEP_MODE: wgpu::VertexStepMode;
    fn size_per_vertex(&self) -> wgpu::BufferAddress;
    fn number_per_vertex(&self) -> wgpu::BufferAddress;
    // fn slot(&self) -> usize;
    fn layout<'a,>(&'a self, attributes: &'a [wgpu::VertexAttribute]) -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: self.size_per_vertex() as wgpu::BufferAddress,
            step_mode: Self::STEP_MODE,
            attributes,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VertexBufferUse {
    Handle(Handle<VertexBuffer>),
    Arc(KeyVertexBuffer),
}
impl VertexBufferUse {
    pub fn buffer<'a>(&'a self, pool: &'a VertexBufferPool) -> &'a VertexBuffer {
        match self {
            VertexBufferUse::Handle(buffer) => buffer,
            VertexBufferUse::Arc(key) => pool.map.get(key).unwrap(),
        }
    }
    pub fn key(&self) -> &KeyVertexBuffer {
        match self {
            VertexBufferUse::Handle(buffer) => buffer.key(),
            VertexBufferUse::Arc(key) => key,
        }
    }
}

#[derive(Clone)]
pub struct RenderVertices {
    pub slot: u32,
    pub buffer: VertexBufferUse,
    pub buffer_range: Option<Range<wgpu::BufferAddress>>,
    pub size_per_value: wgpu::BufferAddress,
}
impl RenderVertices {
    pub fn value_range(&self, pool: &VertexBufferPool) -> Range<u32> {
        if let Some(range) = self.buffer_range.as_ref() {
            let start = (range.start / self.size_per_value) as u32;
            let end = (range.end / self.size_per_value) as u32;
            Range { start, end }
        } else {
            let size = (self.buffer.buffer(pool).size() as wgpu::BufferAddress / self.size_per_value) as u32;
            Range { start: 0, end: size }
        }
    }
    pub fn slice<'a>(&'a self, pool: &'a VertexBufferPool) -> wgpu::BufferSlice {
        if let Some(range) = self.buffer_range.clone() {
            self.buffer.buffer(pool).get_buffer().unwrap().slice(range)
        } else {
            self.buffer.buffer(pool).get_buffer().unwrap().slice(..)
        }
    }
}
impl PartialEq for RenderVertices {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot && self.buffer.key() == other.buffer.key() && self.buffer_range == other.buffer_range
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl Eq for RenderVertices {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}
impl Debug for RenderVertices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderVertices").field("slot", &self.slot).field("buffer", &self.buffer.key()).field("buffer_range", &self.buffer_range).field("size_per_value", &self.size_per_value).finish()
    }
}

#[derive(Debug, Clone)]
pub struct RenderIndices {
    pub buffer: VertexBufferUse,
    pub buffer_range: Option<Range<wgpu::BufferAddress>>,
    pub format: wgpu::IndexFormat,
}
impl RenderIndices {
    pub fn value_range(&self, pool: &VertexBufferPool) -> Range<u32> {
        if let Some(range) = self.buffer_range.as_ref() {
            let start = (range.start / self.format.bytes()) as u32;
            let end = (range.end / self.format.bytes()) as u32;
            Range { start, end }
        } else {
            let size = (self.buffer.buffer(pool).size() as wgpu::BufferAddress / self.format.bytes()) as u32;
            Range { start: 0, end: size }
        }
    }
    pub fn slice<'a>(&'a self, pool: &'a VertexBufferPool) -> wgpu::BufferSlice {
        if let Some(range) = self.buffer_range.clone() {
            self.buffer.buffer(pool).get_buffer().unwrap().slice(range)
        } else {
            self.buffer.buffer(pool).get_buffer().unwrap().slice(..)
        }
    }
}
impl PartialEq for RenderIndices {
    fn eq(&self, other: &Self) -> bool {
        self.buffer.key() == other.buffer.key() && self.buffer_range == other.buffer_range && self.format == other.format
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl Eq for RenderIndices {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}

pub trait TRenderGeometry {
    fn vertices(&self) -> Vec<RenderVertices>;
    fn instances(&self) -> Vec<RenderVertices>;
}

pub type UniformValueBindKey = u128;
pub type DefinesKey = u128;

pub fn update<T: Clone + Copy + Pod>(pool: &mut Vec<T>, data: &[T], offset: usize) -> usize {
    let len = data.len();
    let old_size = pool.len();

    let mut index = 0;
    if offset < old_size {
        for i in offset..old_size {
            pool[i] = data[index];
            index += 1;
            if len <= index  {
                break;
            }
        }
    }

    if index < len {
        for i in index..len {
            pool.push(data[i]);
        }
    }

    pool.len()
}

pub fn calc_uniform_size(
    device: &wgpu::Device,
    used_size: u64,
) -> u64 {
    let limit = device.limits().min_uniform_buffer_offset_alignment as u64;
    let t = used_size / limit;
    if used_size - t * limit > 0 {
        limit * (t + 1)
    } else {
        limit * t
    }
}