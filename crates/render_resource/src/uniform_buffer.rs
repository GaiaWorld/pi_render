use render_core::rhi::{device::RenderDevice, dyn_uniform_buffer::{DynUniformBuffer, Bind, BindOffset, AsBind}, buffer::Buffer, RenderQueue};

pub struct RenderDynUniformBuffer(DynUniformBuffer);
impl RenderDynUniformBuffer {
    pub fn new(device: &RenderDevice) -> Self {
        let limits = device.limits();
        let min_uniform_buffer_offset_alignment = limits.min_uniform_buffer_offset_alignment;
        let dynbuffer = render_core::rhi::dyn_uniform_buffer::DynUniformBuffer::new(
            Some("DynUniformBuffer".to_string()),
            min_uniform_buffer_offset_alignment.max(192),
        );

        Self(dynbuffer)
    }
    pub fn alloc_binding<T: Bind>(&mut self) -> BindOffset {
        self.0.alloc_binding::<T>()
    }
    pub fn alloc_binding_with_asbind<T: AsBind>(&mut self, bind: &T) -> BindOffset {
        self.0.alloc_binding_with_asbind::<T>(bind)
    }
    pub fn buffer(&self) -> Option<&Buffer> {
        self.0.buffer()
    }
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) -> bool {
        self.0.write_buffer(device, queue)
    }
}
impl AsRef<DynUniformBuffer> for RenderDynUniformBuffer {
    fn as_ref(&self) -> &DynUniformBuffer {
        &self.0
    }
}
impl AsMut<DynUniformBuffer> for RenderDynUniformBuffer {
    fn as_mut(&mut self) -> &mut DynUniformBuffer {
        &mut self.0
    }
}