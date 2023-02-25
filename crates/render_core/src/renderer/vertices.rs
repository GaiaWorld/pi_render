use std::{ops::Range, hash::Hash, fmt::Debug};

use pi_assets::asset::Handle;

use crate::rhi::buffer::Buffer;

use super::attributes::EVertexDataKind;

pub trait TKeyAttributes: Debug + Clone + PartialEq + Eq + Hash {

}

pub struct Vertices {
    // pub buffer: Handle<Buffer>,
    pub range: Range<wgpu::BufferAddress>,
}