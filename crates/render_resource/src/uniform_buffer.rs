use render_core::rhi::device::RenderDevice;


pub struct UniformBufferStatic {

}

pub struct SingleUniformBufferDynamic {
    buffer: wgpu::Buffer,
    data: Vec<u8>,
    data_alignment_byte_size: u32,
}
impl SingleUniformBufferDynamic {
    // pub fn new(
    //     device: &RenderDevice,
    // ) -> Self {
    //     let data_alignment_byte_size = device.limits().min_uniform_buffer_offset_alignment;
    //     device.create_buffer(
    //         &wgpu::BufferDescriptor {
    //             label: todo!(),
    //             size: todo!(),
    //             usage: wgpu::BufferUsages::,
    //             mapped_at_creation: todo!(),
    //         }
    //     );
    // }
}