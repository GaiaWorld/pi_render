use std::{ops::Range, hash::Hash, fmt::Debug, mem::replace};

use super::{vertex_buffer::KeyVertexBuffer, attributes::{VertexAttribute, EVertexDataKind}, instance::EInstanceKind, vertex_format::TVertexFormatByteSize};


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

#[derive(Debug, Clone)]
///
/// 
/// Range<wgpu::BufferAddress> : byte数据范围
pub struct VertexBufferDesc {
    pub key: KeyVertexBuffer,
    pub range: Option<Range<wgpu::BufferAddress>>,
    pub attrs: Vec<VertexAttribute>,
    pub step_mode: wgpu::VertexStepMode,
    pub kind: EInstanceKind,
}
impl VertexBufferDesc {
    pub fn update_range(&mut self, value: Option<Range<wgpu::BufferAddress>>) {
        let _ = replace(&mut self.range, value);
    }
    pub fn instance_tilloff() -> Self {
        Self {
            key: KeyVertexBuffer::from("NullIntanceTillOff"),
            range: None,
            attrs: vec![
                VertexAttribute { kind: EVertexDataKind::InsTillOffset1, format: wgpu::VertexFormat::Float32x4 },
            ],
            step_mode: wgpu::VertexStepMode::Instance,
            kind: EInstanceKind::TillOffset,
        }
    }
    pub fn instance_color() -> Self {
        Self {
            key: KeyVertexBuffer::from("NullIntanceColor"),
            range: None,
            attrs: vec![
                VertexAttribute { kind: EVertexDataKind::InsColor, format: wgpu::VertexFormat::Float32x4 },
            ],
            step_mode: wgpu::VertexStepMode::Instance,
            kind: EInstanceKind::Color,
        }
    }
    pub fn instance_world_matrix() -> Self {
        Self {
            key: KeyVertexBuffer::from("NullIntanceWM"),
            range: None,
            attrs: vec![
                VertexAttribute { kind: EVertexDataKind::InsWorldRow1, format: wgpu::VertexFormat::Float32x4 },
                VertexAttribute { kind: EVertexDataKind::InsWorldRow2, format: wgpu::VertexFormat::Float32x4 },
                VertexAttribute { kind: EVertexDataKind::InsWorldRow3, format: wgpu::VertexFormat::Float32x4 },
                VertexAttribute { kind: EVertexDataKind::InsWorldRow4, format: wgpu::VertexFormat::Float32x4 },
            ],
            step_mode: wgpu::VertexStepMode::Instance,
            kind: EInstanceKind::WorldMatrix,
        }
    }
    pub fn vertices(bufferkey: KeyVertexBuffer, range: Option<Range<wgpu::BufferAddress>>, attrs: Vec<VertexAttribute>) -> Self {
        Self {
            key: bufferkey,
            range,
            attrs,
            step_mode: wgpu::VertexStepMode::Vertex,
            kind: EInstanceKind::None,
        }
    }
    pub fn bufferkey(&self) -> &KeyVertexBuffer {
        &self.key
    }
    pub fn range(&self) -> &Option<Range<wgpu::BufferAddress>> {
        &self.range
    }
    pub fn instance_kind(&self) -> EInstanceKind {
        self.kind
    }
    pub fn attributes(&self) -> &Vec<VertexAttribute> {
        &self.attrs
    }
    pub fn stride(&self) -> wgpu::BufferAddress {
        
        let mut result = 0;
        self.attributes().iter().for_each(|attr| {
            result += attr.format.use_bytes()
        });

        result
    }

    pub fn step_mode(&self) -> wgpu::VertexStepMode {
        self.step_mode
    }
}
impl Hash for VertexBufferDesc {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.attrs.hash(state);
        self.step_mode.hash(state);
    }
}