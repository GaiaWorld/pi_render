use std::{ops::Range, hash::Hash, fmt::Debug};

use super::{vertex_buffer::KeyVertexBuffer, attributes::{VertexAttribute, EVertexAttribute}, vertex_format::TVertexFormatByteSize};


#[derive(Debug, Clone, Copy)]
pub enum EVertexBufferSlot {
    Slot01,
    Slot02,
    Slot03,
    Slot04,
    Slot05,
    Slot06,
    Slot07,
    Slot08,
    Slot09,
    Slot10,
    Slot11,
    Slot12,
    Slot13,
    Slot14,
    Slot15,
    Slot16,
}
impl EVertexBufferSlot {
    pub fn from_u8_unsafe(index: u8) -> Self {
        if index == 0 {
            Self::Slot01
        }
        else if index == 1 {
            Self::Slot02
        }
        else if index == 2 {
            Self::Slot03
        }
        else if index == 3 {
            Self::Slot04
        }
        else if index == 4 {
            Self::Slot05
        }
        else if index == 5 {
            Self::Slot06
        }
        else if index == 6 {
            Self::Slot07
        }
        else if index == 7 {
            Self::Slot08
        }
        else if index == 8 {
            Self::Slot09
        }
        else if index == 9 {
            Self::Slot10
        }
        else if index == 10 {
            Self::Slot11
        }
        else if index == 11 {
            Self::Slot12
        }
        else if index == 12 {
            Self::Slot13
        }
        else if index == 13 {
            Self::Slot14
        }
        else if index == 14 {
            Self::Slot15
        }
        else {
            Self::Slot16
        }
    }
}

pub type VertexBufferRangeVType = u32;
#[derive(Debug, Clone, Copy)]
pub struct VertexBufferDescRange(pub(crate) VertexBufferRangeVType, pub(crate) VertexBufferRangeVType);
impl Default for VertexBufferDescRange {
    fn default() -> Self {
        Self(0, 0)
    }
}
impl VertexBufferDescRange {
    pub fn new(start: VertexBufferRangeVType, end: VertexBufferRangeVType) -> Self {
        Self(start, end)
    }
    pub fn range(&self) -> Option<Range<wgpu::BufferAddress>> {
        if self.1 <= self.0 {
            None
        } else {
            Some(Range { start: self.0 as wgpu::BufferAddress, end: self.1 as wgpu::BufferAddress })
        }
    }
}

#[derive(Debug, Clone)]
///
/// 
/// Range<wgpu::BufferAddress> : byte数据范围
pub struct VertexBufferDesc {
    pub key: KeyVertexBuffer,
    range: VertexBufferDescRange,
    attrs: Vec<EVertexAttribute>,
    instance: bool,
}
impl VertexBufferDesc {
    pub fn update_range(&mut self, value: VertexBufferDescRange) {
        self.range = value;
        // let _ = replace(&mut self.range, value);
    }
    pub fn new(bufferkey: KeyVertexBuffer, range: VertexBufferDescRange, attrs: Vec<EVertexAttribute>, instance: bool) -> Self {
        Self {
            key: bufferkey,
            range,
            attrs,
            instance,
        }
    }
    pub fn vertices(bufferkey: KeyVertexBuffer, range: VertexBufferDescRange, attrs: Vec<EVertexAttribute>) -> Self {
        Self {
            key: bufferkey,
            range,
            attrs,
            instance: false,
        }
    }
    pub fn bufferkey(&self) -> &KeyVertexBuffer {
        &self.key
    }
    pub fn range(&self) -> Option<Range<wgpu::BufferAddress>> {
        self.range.range()
    }
    pub fn instance(&self) -> bool {
        self.instance
    }
    pub fn attributes(&self) -> &Vec<EVertexAttribute> {
        &self.attrs
    }
    pub fn stride(&self) -> wgpu::BufferAddress {
        
        let mut result = 0;
        self.attributes().iter().for_each(|attr| {
            result += attr.format().use_bytes()
        });

        result
    }

    pub fn step_mode(&self) -> wgpu::VertexStepMode {
        if self.instance { wgpu::VertexStepMode::Instance } else { wgpu::VertexStepMode::Vertex }
    }
}
impl Hash for VertexBufferDesc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.attrs.hash(state);
        self.instance.hash(state);
    }
}