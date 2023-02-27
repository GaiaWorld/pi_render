use std::{ops::Range, hash::Hash, fmt::Debug};

use lazy_static::__Deref;
use pi_assets::asset::Handle;

use super::{vertex_buffer::EVertexBufferRange, vertex_format::TVertexFormatByteSize};

pub trait TKeyAttributes: Debug + Clone + PartialEq + Eq + Hash {

}


#[derive(Clone)]
pub struct RenderVertices {
    pub slot: u32,
    pub buffer: Handle<EVertexBufferRange>,
    pub buffer_range: Option<Range<wgpu::BufferAddress>>,
    pub size_per_value: wgpu::BufferAddress,
}
impl RenderVertices {
    pub fn value_range(&self) -> Range<u32> {
        let mut range0 = self.buffer.range();
    
        if let Some(range) = self.buffer_range.as_ref() {
            range0.start += range.start;
            range0.end += range0.start + range.end - range.start;
        }
        
        Range {
            start: (range0.start / self.size_per_value) as u32,
            end: (range0.end / self.size_per_value) as u32,
        }
    }
    pub fn slice<'a>(&'a self) -> wgpu::BufferSlice {
        let mut range0 = self.buffer.range();
    
        if let Some(range) = self.buffer_range.as_ref() {
            range0.start += range.start;
            range0.end += range0.start + range.end - range.start;
        }

        self.buffer.buffer().deref().slice(range0)
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
    pub buffer: Handle<EVertexBufferRange>,
    pub buffer_range: Option<Range<wgpu::BufferAddress>>,
    pub format: wgpu::IndexFormat,
}
impl RenderIndices {
    pub fn value_range(&self) -> Range<u32> {
        let mut range0 = self.buffer.range();
    
        if let Some(range) = self.buffer_range.as_ref() {
            range0.start += range.start;
            range0.end += range0.start + range.end - range.start;
        }
        
        Range {
            start: (range0.start / self.format.use_bytes()) as u32,
            end: (range0.end / self.format.use_bytes()) as u32,
        }
    }
    pub fn slice<'a>(&'a self) -> wgpu::BufferSlice {
        let mut range0 = self.buffer.range();
    
        if let Some(range) = self.buffer_range.as_ref() {
            range0.start += range.start;
            range0.end += range0.start + range.end - range.start;
        }

        log::info!("RenderIndices Range: {:?}", range0);

        self.buffer.buffer().deref().slice(range0)
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