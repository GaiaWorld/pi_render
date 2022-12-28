use crate::vertex_data::EVertexDataKind;



pub trait TVertexShaderCode {
    fn vs_defines_code(&self) -> String;
    fn vs_running_code(&self) -> String;
}

pub trait TVertexFormatCode {
    fn vs_code(&self) -> String;
}

impl TVertexFormatCode for wgpu::VertexFormat {
    fn vs_code(&self) -> String {
        match self {
            wgpu::VertexFormat::Uint8x2     => String::from(""),
            wgpu::VertexFormat::Uint8x4     => String::from(""),
            wgpu::VertexFormat::Sint8x2     => String::from(""),
            wgpu::VertexFormat::Sint8x4     => String::from(""),
            wgpu::VertexFormat::Unorm8x2    => String::from(""),
            wgpu::VertexFormat::Unorm8x4    => String::from(""),
            wgpu::VertexFormat::Snorm8x2    => String::from(""),
            wgpu::VertexFormat::Snorm8x4    => String::from(""),
            wgpu::VertexFormat::Uint16x2    => String::from(""),
            wgpu::VertexFormat::Uint16x4    => String::from(""),
            wgpu::VertexFormat::Sint16x2    => String::from(""),
            wgpu::VertexFormat::Sint16x4    => String::from(""),
            wgpu::VertexFormat::Unorm16x2   => String::from(""),
            wgpu::VertexFormat::Unorm16x4   => String::from(""),
            wgpu::VertexFormat::Snorm16x2   => String::from(""),
            wgpu::VertexFormat::Snorm16x4   => String::from(""),
            wgpu::VertexFormat::Float16x2   => String::from(""),
            wgpu::VertexFormat::Float16x4   => String::from(""),
            wgpu::VertexFormat::Float32     => String::from("float"),
            wgpu::VertexFormat::Float32x2   => String::from("vec2"),
            wgpu::VertexFormat::Float32x3   => String::from("vec3"),
            wgpu::VertexFormat::Float32x4   => String::from("vec4"),
            wgpu::VertexFormat::Uint32      => String::from("uint"),
            wgpu::VertexFormat::Uint32x2    => String::from("uvec2"),
            wgpu::VertexFormat::Uint32x3    => String::from("uvec3"),
            wgpu::VertexFormat::Uint32x4    => String::from("uvec4"),
            wgpu::VertexFormat::Sint32      => String::from("int"),
            wgpu::VertexFormat::Sint32x2    => String::from("ivec2"),
            wgpu::VertexFormat::Sint32x3    => String::from("ivec3"),
            wgpu::VertexFormat::Sint32x4    => String::from("ivec4"),
            wgpu::VertexFormat::Float64     => String::from(""),
            wgpu::VertexFormat::Float64x2   => String::from(""),
            wgpu::VertexFormat::Float64x3   => String::from(""),
            wgpu::VertexFormat::Float64x4   => String::from(""),
        }
    }
}

impl TVertexFormatCode for wgpu::VertexAttribute {
    fn vs_code(&self) -> String {
        let mut result = String::from("layout(location = ");
        result += self.shader_location.to_string().as_str();
        result += ") in ";
        result += self.format.vs_code().as_str();
        result += " ";

        result
    }
}