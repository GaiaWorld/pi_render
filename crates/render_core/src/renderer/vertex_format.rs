
pub trait TVertexFormatByteSize {
    fn use_bytes(&self) -> wgpu::BufferAddress ;
}

impl TVertexFormatByteSize for wgpu::IndexFormat {
    fn use_bytes(&self) -> wgpu::BufferAddress  {
        match self {
            wgpu::IndexFormat::Uint16 => 2,
            wgpu::IndexFormat::Uint32 => 4,
        }
    }
}

impl TVertexFormatByteSize for wgpu::VertexFormat {
    fn use_bytes(&self) -> wgpu::BufferAddress {
        match self {
            wgpu::VertexFormat::Uint8x2     => 1 * 2,
            wgpu::VertexFormat::Uint8x4     => 1 * 4,
            wgpu::VertexFormat::Sint8x2     => 1 * 2,
            wgpu::VertexFormat::Sint8x4     => 1 * 4,
            wgpu::VertexFormat::Unorm8x2    => 1 * 2,
            wgpu::VertexFormat::Unorm8x4    => 1 * 4,
            wgpu::VertexFormat::Snorm8x2    => 1 * 2,
            wgpu::VertexFormat::Snorm8x4    => 1 * 4,
            wgpu::VertexFormat::Uint16x2    => 2 * 2,
            wgpu::VertexFormat::Uint16x4    => 2 * 4,
            wgpu::VertexFormat::Sint16x2    => 2 * 2,
            wgpu::VertexFormat::Sint16x4    => 2 * 4,
            wgpu::VertexFormat::Unorm16x2   => 2 * 2,
            wgpu::VertexFormat::Unorm16x4   => 2 * 4,
            wgpu::VertexFormat::Snorm16x2   => 2 * 2,
            wgpu::VertexFormat::Snorm16x4   => 2 * 4,
            wgpu::VertexFormat::Float16x2   => 2 * 2,
            wgpu::VertexFormat::Float16x4   => 2 * 4,
            wgpu::VertexFormat::Float32     => 4 * 1,
            wgpu::VertexFormat::Float32x2   => 4 * 2,
            wgpu::VertexFormat::Float32x3   => 4 * 3,
            wgpu::VertexFormat::Float32x4   => 4 * 4,
            wgpu::VertexFormat::Uint32      => 4 * 1,
            wgpu::VertexFormat::Uint32x2    => 4 * 2,
            wgpu::VertexFormat::Uint32x3    => 4 * 3,
            wgpu::VertexFormat::Uint32x4    => 4 * 4,
            wgpu::VertexFormat::Sint32      => 4 * 1,
            wgpu::VertexFormat::Sint32x2    => 4 * 2,
            wgpu::VertexFormat::Sint32x3    => 4 * 3,
            wgpu::VertexFormat::Sint32x4    => 4 * 4,
            wgpu::VertexFormat::Float64     => 8 * 1,
            wgpu::VertexFormat::Float64x2   => 8 * 2,
            wgpu::VertexFormat::Float64x3   => 8 * 3,
            wgpu::VertexFormat::Float64x4   => 8 * 4,
        }
    }
}