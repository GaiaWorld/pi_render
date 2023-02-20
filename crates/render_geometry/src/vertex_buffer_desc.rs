use std::{ops::Range, hash::Hash, fmt::Debug, mem::replace};

use render_data_container::KeyVertexBuffer;
use render_shader::attributes::EVertexDataKind;

use crate::{vertex_attribute::VertexAttribute, vertex_format::TVertexFormatByteSize};


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EInstanceKind {
    None,
    WorldMatrix,
    Color,
    TillOffset,
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