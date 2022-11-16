pub mod bind_group_layout;
pub mod bind_group;
pub mod uniform_buffer;
pub mod sampler;

pub fn bind_group_entry_buffer(
    binding: u32,
    buffer: &wgpu::Buffer,
    offset: wgpu::BufferAddress,
    size: wgpu::BufferAddress,
) -> wgpu::BindGroupEntry {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Buffer(
            wgpu::BufferBinding {
                buffer,
                offset,
                size: wgpu::BufferSize::new(size),
            }
        ),
    }
}