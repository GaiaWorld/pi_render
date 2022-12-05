pub mod vertex_data;
pub mod error;
pub mod geometry;


pub trait TVertexbBufferMeta {
    const DATA_FORMAT: EVertexDataFormat;
    const STEP_MODE: wgpu::VertexStepMode;
    fn size_per_vertex(&self) -> usize;
    fn slot(&self) -> usize;
    fn attributes(&self) -> &[wgpu::VertexAttribute];
    fn layout<'a>(&'a self) -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: self.size_per_vertex() as wgpu::BufferAddress,
            step_mode: Self::STEP_MODE,
            attributes: self.attributes(),
        }
    }
}