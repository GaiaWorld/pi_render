use std::ops::Range;

use derive_deref::{Deref, DerefMut};
use pi_assets::asset::Handle;
use render_data_container::{KeyVertexBuffer, VertexBuffer};

#[derive(Debug)]
pub struct IndicesBufferDesc {
    pub format: wgpu::IndexFormat,
    pub buffer_range: Option<Range<wgpu::BufferAddress>>,
    pub buffer: KeyVertexBuffer,
}

#[derive(Debug, Deref, DerefMut, Clone, Hash)]
pub struct AssetKeyBufferIndices(pub KeyVertexBuffer);

#[derive(Deref, DerefMut)]
pub struct AssetResBufferIndices(pub Handle<VertexBuffer>);
impl From<Handle<VertexBuffer>> for AssetResBufferIndices {
    fn from(value: Handle<VertexBuffer>) -> Self {
        Self(value)
    }
}
