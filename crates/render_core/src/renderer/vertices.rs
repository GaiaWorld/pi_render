use std::{ops::Range, hash::Hash, fmt::Debug, sync::Arc};

use lazy_static::__Deref;
use pi_assets::asset::Handle;

use crate::rhi::{asset::RenderRes, buffer::Buffer};

use super::{vertex_buffer::EVertexBufferRange, vertex_format::TVertexFormatByteSize};

pub trait TKeyAttributes: Debug + Clone + PartialEq + Eq + Hash {

}

#[derive(Clone)]
pub enum EVerticesBufferUsage {
    GUI(Handle<RenderRes<Buffer>>),
    Other(Handle<EVertexBufferRange>),
    EVBRange(Arc<EVertexBufferRange>),
}
impl EVerticesBufferUsage {
    pub fn range(&self) -> Range<wgpu::BufferAddress> {
        match self {
            EVerticesBufferUsage::GUI(val) => Range { start: 0, end: val.size() },
            EVerticesBufferUsage::Other(val) => val.range(),
            EVerticesBufferUsage::EVBRange(val) => val.range(),
        }
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        match self {
            EVerticesBufferUsage::GUI(val) => val,
            EVerticesBufferUsage::Other(val) => val.buffer(),
            EVerticesBufferUsage::EVBRange(val) => val.buffer(),
        }
    }
}
impl PartialEq for EVerticesBufferUsage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::GUI(l0), Self::GUI(r0)) => {
                l0.key() == r0.key()
            },
            (Self::Other(l0), Self::Other(r0)) => {
                l0.key() == r0.key() && l0.range() == r0.range()
            },
            (Self::EVBRange(_), Self::EVBRange(_)) => {
                false
            },
            _ => false,
        }
    }
}
impl Eq for EVerticesBufferUsage {
    fn assert_receiver_is_total_eq(&self) {}
}
impl Debug for EVerticesBufferUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GUI(arg0) => f.debug_tuple("GUI").field(arg0).finish(),
            Self::Other(arg0) => f.debug_tuple("Other").field(arg0).finish(),
            Self::EVBRange(arg0) => f.debug_tuple("Other").field(arg0).finish(),
        }
    }
}


#[derive(Clone)]
pub struct RenderVertices {
    pub slot: u32,
    /// 使用的Buffer数据
    pub buffer: EVerticesBufferUsage,
    /// 使用了Buffer的哪个部分
    pub buffer_range: Option<Range<wgpu::BufferAddress>>,
    /// Buffer 打组的数据单位尺寸
    pub size_per_value: wgpu::BufferAddress,
}
impl RenderVertices {
    pub fn value_range(&self) -> Range<u32> {
        let mut range0 = self.buffer.range();

        if let Some(range) = self.buffer_range.as_ref() {
            range0.start    += range.start;
            range0.end      = range0.start + (range.end - range.start);
        }

        Range {
            start: (range0.start / self.size_per_value) as u32,
            end: (range0.end / self.size_per_value) as u32,
        }
    }
    pub fn slice<'a>(&'a self) -> wgpu::BufferSlice {
        let mut range0 = self.buffer.range();

        if let Some(range) = self.buffer_range.as_ref() {
            range0.start    += range.start;
            range0.end      = range0.start + (range.end - range.start);
        }
        log::info!("slice {:?}", range0);

        self.buffer.buffer().slice(range0)
    }
}
impl PartialEq for RenderVertices {
    fn eq(&self, other: &Self) -> bool {
        &self.slot == &other.slot && &self.buffer == &other.buffer && &self.buffer_range == &other.buffer_range
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
        f.debug_struct("RenderVertices").field("slot", &self.slot).field("buffer", &self.buffer).field("buffer_range", &self.buffer_range).field("size_per_value", &self.size_per_value).finish()
    }
}

#[derive(Debug, Clone)]
pub struct RenderIndices {
    /// 使用的Buffer数据
    pub buffer: EVerticesBufferUsage,
    /// 使用了Buffer的哪个部分
    pub buffer_range: Option<Range<wgpu::BufferAddress>>,
    pub format: wgpu::IndexFormat,
}
impl RenderIndices {
    pub fn value_range(&self) -> Range<u32> {
        let mut range0 = self.buffer.range();
    
        if let Some(range) = self.buffer_range.as_ref() {
            range0.end = range.end - range.start;
            range0.start = range.start;
        } else {
            range0.end = range0.end - range0.start;
            range0.start = 0;
        }
        
        Range {
            start: (range0.start / self.format.use_bytes()) as u32,
            end: (range0.end / self.format.use_bytes()) as u32,
        }
    }
    pub fn slice<'a>(&'a self) -> wgpu::BufferSlice {
        let range0 = self.buffer.range();
    
        // log::info!("RenderIndices buffer Range: {:?}", range0);
        // if let Some(range) = self.buffer_range.as_ref() {
        //     range0.start += range.start;
        //     range0.end = range0.start + range.end - range.start;
        // }

        // log::info!("RenderIndices Range: {:?}", range0);

        self.buffer.buffer().slice(range0)
    }
}
impl PartialEq for RenderIndices {
    fn eq(&self, other: &Self) -> bool {
        &self.buffer == &other.buffer && self.format == other.format
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl Eq for RenderIndices {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}