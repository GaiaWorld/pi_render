pub struct UniformBuffer {
    buffer: wgpu::Buffer,
    size: wgpu::BufferAddress,
}

impl UniformBuffer {
    pub fn get_buffer(&self) -> &wgpu::Buffer {

    }
    pub fn update(&self, queue: &wgpu::Queue, offset: wgpu::BufferAddress, data: &[u8]) {
        queue.write_buffer(&self.buffer, offset, data);
    }
}