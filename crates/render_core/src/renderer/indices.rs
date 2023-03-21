use std::ops::{Range};

use derive_deref_rs::Deref;
use pi_assets::asset::Handle;

use super::{vertex_buffer::{KeyVertexBuffer, EVertexBufferRange}, vertices::EVerticesBufferUsage};

#[derive(Debug)]
pub struct IndicesBufferDesc {
    pub format: wgpu::IndexFormat,
    pub buffer_range: Option<Range<wgpu::BufferAddress>>,
    pub buffer: KeyVertexBuffer,
}

#[derive(Debug, Deref, Clone, Hash)]
pub struct AssetKeyBufferIndices(pub KeyVertexBuffer);

#[derive(Deref)]
pub struct AssetResBufferIndices(pub EVerticesBufferUsage);
impl From<EVerticesBufferUsage> for AssetResBufferIndices {
    fn from(value: EVerticesBufferUsage) -> Self {
        Self(value)
    }
}
