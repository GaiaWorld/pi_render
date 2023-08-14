use std::{ops::Range, hash::Hash, fmt::Debug, sync::Arc};

use pi_assets::asset::Handle;
use pi_share::Share;

use crate::rhi::{asset::RenderRes, buffer::Buffer, buffer_alloc::BufferIndex};

use super::{vertex_buffer::EVertexBufferRange, vertex_format::TVertexFormatByteSize};

pub trait TKeyAttributes: Debug + Clone + PartialEq + Eq + Hash {

}

#[derive(Clone)]
pub enum EVerticesBufferUsage {
    GUI(Handle<RenderRes<Buffer>>),
	Part(Share<BufferIndex>), // 改为Arc<RefCell<BufferIndex>>？TODO
    /// 3D Buffer 不会更新
    Other(Handle<EVertexBufferRange>),
    /// 3D Buffer, 可以更新数据 - 应用于 粒子系统、实例化等情况
    EVBRange(Arc<EVertexBufferRange>),
    Temp(Arc<Buffer>),
}
impl EVerticesBufferUsage {
    pub fn range(&self) -> Range<wgpu::BufferAddress> {
        match self {
            EVerticesBufferUsage::GUI(val) => Range { start: 0, end: val.size() },
			EVerticesBufferUsage::Part(index) => index.range(),
            EVerticesBufferUsage::Other(val) => val.range(),
            EVerticesBufferUsage::EVBRange(val) => val.range(),
            EVerticesBufferUsage::Temp(val) => Range { start: 0, end: val.size() },
        }
    }
    pub fn active_range(&self) -> Range<wgpu::BufferAddress> {
        match self {
            EVerticesBufferUsage::GUI(val) => Range { start: 0, end: val.size() },
			EVerticesBufferUsage::Part(index) => index.range(),
            EVerticesBufferUsage::Other(val) => val.active_range(),
            EVerticesBufferUsage::EVBRange(val) => val.active_range(),
            EVerticesBufferUsage::Temp(val) => Range { start: 0, end: val.size() },
        }
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        match self {
            EVerticesBufferUsage::GUI(val) => val,
			EVerticesBufferUsage::Part(index) => index.buffer(),
            EVerticesBufferUsage::Other(val) => val.buffer(),
            EVerticesBufferUsage::EVBRange(val) => val.buffer(),
            EVerticesBufferUsage::Temp(val) => val,
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
            (Self::EVBRange(v), Self::EVBRange(v2)) => {
                v.as_ref() == v2.as_ref()
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
			Self::Part(arg0) =>  f.debug_tuple("Part").field(arg0).finish(),
            Self::Other(arg0) => f.debug_tuple("Other").field(arg0).finish(),
            Self::EVBRange(arg0) => f.debug_tuple("EVBRange").field(arg0).finish(),
            Self::Temp(arg0) => f.debug_tuple("Temp").field(arg0).finish(),
        }
    }
}

///
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
        let mut range0 = self.buffer.active_range();

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

        self.buffer.buffer().slice(range0)
    }
}
impl PartialEq for RenderVertices {
    fn eq(&self, other: &Self) -> bool {
        &self.slot == &other.slot && &self.buffer == &other.buffer  // && &self.buffer_range == &other.buffer_range
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
        let mut range0 = self.buffer.active_range();
    
        if let Some(range) = self.buffer_range.as_ref() {
            range0.end = range.end;
            range0.start = range.start;
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

#[test]
fn tt() {
	println!("!!!===={:?}", std::mem::size_of::<RenderVertices>());
}