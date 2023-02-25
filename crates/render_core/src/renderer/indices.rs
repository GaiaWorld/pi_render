use std::ops::{Range, DerefMut};

use derive_deref_rs::Deref;
use pi_assets::asset::Handle;

use super::vertex_buffer::{KeyVertexBuffer, EVertexBufferRange};

#[derive(Debug)]
pub struct IndicesBufferDesc {
    pub format: wgpu::IndexFormat,
    pub buffer_range: Option<Range<wgpu::BufferAddress>>,
    pub buffer: KeyVertexBuffer,
}

#[derive(Debug, Deref, Clone, Hash)]
pub struct AssetKeyBufferIndices(pub KeyVertexBuffer);

#[derive(Deref)]
pub struct AssetResBufferIndices(pub Handle<EVertexBufferRange>);
impl From<Handle<EVertexBufferRange>> for AssetResBufferIndices {
    fn from(value: Handle<EVertexBufferRange>) -> Self {
        Self(value)
    }
}
