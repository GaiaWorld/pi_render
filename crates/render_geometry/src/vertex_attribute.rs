use render_shader::attributes::EVertexDataKind;

pub(crate) trait TAsWgpuVertexAtribute {
    fn as_attribute(&self, offset: wgpu::BufferAddress, shader_location: u32) ->wgpu::VertexAttribute;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexAttribute {
    pub kind: EVertexDataKind,
    pub format: wgpu::VertexFormat,
}
impl PartialOrd for VertexAttribute {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.kind.partial_cmp(&other.kind)
    }
}
impl Ord for VertexAttribute {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl TAsWgpuVertexAtribute for VertexAttribute {
    fn as_attribute(&self, offset: wgpu::BufferAddress, shader_location: u32) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: self.format,
            offset,
            shader_location,
        }
    }
}
