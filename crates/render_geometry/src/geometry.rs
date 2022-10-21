
use std::hash::Hash;
use render_data_container::{TGeometryBufferID, TVertexDataKindKey, GeometryBufferPool, EVertexDataFormat};
use pi_share::Share;
use crate::error::EGeometryError;
use crate::vertex_data::{VertexBufferU8, VertexBufferU16, VertexBufferU32, VertexBufferF32, VertexBufferF64};

// #[derive(Debug, Clone)]
// pub struct GeometryKindBuffer<GBID: TGeometryBufferID> {
//     pub u8: Option<GBID>,
//     pub u16: Option<GBID>,
//     pub u32: Option<GBID>,
//     pub f32: Option<GBID>,
//     pub f64: Option<GBID>,
// }

// impl<GBID: TGeometryBufferID> Default for GeometryKindBuffer<GBID> {
//     fn default() -> Self {
//         Self { u8: None, u16: None, u32: None, f32: None, f64: None }
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct GeometryDataDesc<VDK: TVertexBufferKindKey>{
//     format: EVertexDataFormat,
//     kind: VDK,
// }

pub trait VertexAttributeMeta {
    const SLOT: u32;
    const SIZE_PER_VERTEX: u32;
    const FORMAT: EVertexDataFormat;
    const STEP_MODE: wgpu::VertexStepMode;
    fn layout<'a>(attributes: &'a [wgpu::VertexAttribute]) -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: Self::SIZE_PER_VERTEX as wgpu::BufferAddress,
            step_mode: Self::STEP_MODE,
            attributes,
        }
    }
}

pub struct VertexAttributeBufferMeta<GBID: TGeometryBufferID> {
    pub buffer_id: GBID,
    pub start: usize,
    pub end: usize,
    pub data_bytes_size: usize,
}