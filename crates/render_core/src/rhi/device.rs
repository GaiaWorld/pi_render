use super::{
    bind_group::BindGroup,
    bind_group_layout::BindGroupLayout,
    buffer::Buffer,
    pipeline::{ComputePipeline, RenderPipeline},
    texture::{Sampler, Texture},
};
use derive_deref_rs::Deref;
use futures::Future;
use pi_share::Share;
use wgpu::{util::DeviceExt, BufferAsyncError};

/// This GPU device is responsible for the creation of most rendering and compute resources.
#[derive(Clone, Deref)]
pub struct RenderDevice(Share<wgpu::Device>);

impl From<Share<wgpu::Device>> for RenderDevice {
    fn from(device: Share<wgpu::Device>) -> Self {
        log::warn!("=============== limits = {:?}", device.limits());

        Self(device)
    }
}

impl RenderDevice {
    /// List all [`Features`](wgpu::Features) that may be used with this device.
    ///
    /// Functions may panic if you use unsupported features.
    #[inline]
    pub fn features(&self) -> wgpu::Features {
        self.0.features()
    }

    /// List all [`Limits`](wgpu::Limits) that were requested of this device.
    ///
    /// If any of these limits are exceeded, functions may panic.
    #[inline]
    pub fn limits(&self) -> wgpu::Limits {
        self.0.limits()
    }

    /// Creates a [`ShaderModule`](wgpu::ShaderModule) from either SPIR-V or WGSL source code.
    #[inline]
    pub fn create_shader_module(&self, desc: wgpu::ShaderModuleDescriptor) -> wgpu::ShaderModule {
        self.0.create_shader_module(desc)
    }

    /// Check for resource cleanups and mapping callbacks.
    ///
    /// no-op on the web, device is automatically polled.
    #[inline]
    pub fn poll(&self, maintain: wgpu::Maintain) -> bool {
        self.0.poll(maintain)
    }

    /// Creates an empty [`CommandEncoder`](wgpu::CommandEncoder).
    #[inline]
    pub fn create_command_encoder(
        &self,
        desc: &wgpu::CommandEncoderDescriptor,
    ) -> wgpu::CommandEncoder {
        self.0.create_command_encoder(desc)
    }

    /// Creates an empty [`RenderBundleEncoder`](wgpu::RenderBundleEncoder).
    #[inline]
    pub fn create_render_bundle_encoder(
        &self,
        desc: &wgpu::RenderBundleEncoderDescriptor,
    ) -> wgpu::RenderBundleEncoder {
        self.0.create_render_bundle_encoder(desc)
    }

    /// Creates a new [`BindGroup`](wgpu::BindGroup).
    #[inline]
    pub fn create_bind_group(&self, desc: &wgpu::BindGroupDescriptor) -> BindGroup {
        let wgpu_bind_group = self.0.create_bind_group(desc);
        BindGroup::from(wgpu_bind_group)
    }

    /// Creates a [`BindGroupLayout`](wgpu::BindGroupLayout).
    #[inline]
    pub fn create_bind_group_layout(
        &self,
        desc: &wgpu::BindGroupLayoutDescriptor,
    ) -> BindGroupLayout {
        BindGroupLayout::from(self.0.create_bind_group_layout(desc))
    }

    /// Creates a [`PipelineLayout`](wgpu::PipelineLayout).
    #[inline]
    pub fn create_pipeline_layout(
        &self,
        desc: &wgpu::PipelineLayoutDescriptor,
    ) -> wgpu::PipelineLayout {
        self.0.create_pipeline_layout(desc)
    }

    /// Creates a [`RenderPipeline`].
    #[inline]
    pub fn create_render_pipeline(&self, desc: &wgpu::RenderPipelineDescriptor) -> RenderPipeline {
        let wgpu_render_pipeline = self.0.create_render_pipeline(desc);
        RenderPipeline::from(wgpu_render_pipeline)
    }

    /// Creates a [`ComputePipeline`].
    #[inline]
    pub fn create_compute_pipeline(
        &self,
        desc: &wgpu::ComputePipelineDescriptor,
    ) -> ComputePipeline {
        let wgpu_compute_pipeline = self.0.create_compute_pipeline(desc);
        ComputePipeline::from(wgpu_compute_pipeline)
    }

    /// Creates a [`Buffer`].
    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> Buffer {
        let wgpu_buffer = self.0.create_buffer(desc);
        Buffer::from(wgpu_buffer)
    }

    /// Creates a [`Buffer`] and initializes it with the specified data.
    pub fn create_buffer_with_data(&self, desc: &wgpu::util::BufferInitDescriptor) -> Buffer {
        let wgpu_buffer = self.0.create_buffer_init(desc);
        Buffer::from(wgpu_buffer)
    }

    /// Creates a new [`Texture`].
    ///
    /// `desc` specifies the general format of the texture.
    pub fn create_texture(&self, desc: &wgpu::TextureDescriptor) -> Texture {
        let wgpu_texture = self.0.create_texture(desc);
        Texture::from(wgpu_texture)
    }

    /// Creates a new [`Sampler`].
    ///
    /// `desc` specifies the behavior of the sampler.
    pub fn create_sampler(&self, desc: &wgpu::SamplerDescriptor) -> Sampler {
        let wgpu_sampler = self.0.create_sampler(desc);
        Sampler::from(wgpu_sampler)
    }

    /// Initializes [`Surface`](wgpu::Surface) for presentation.
    ///
    /// # Panics
    ///
    /// - A old [`SurfaceTexture`](wgpu::SurfaceTexture) is still alive referencing an old surface.
    /// - Texture format requested is unsupported on the surface.
    pub fn configure_surface(&self, surface: &wgpu::Surface, config: &wgpu::SurfaceConfiguration) {
        surface.configure(&self.0, config)
    }

    /// Returns the wgpu [`Device`](wgpu::Device).
    pub fn wgpu_device(&self) -> &wgpu::Device {
        &self.0
    }

    pub async fn map_buffer(
        &self,
        buffer: &wgpu::BufferSlice<'_>,
        map_mode: wgpu::MapMode,
    ) -> Result<(), BufferAsyncError> {
		let var = pi_async::prelude::AsyncValue::new();
		let var1 = var.clone();
        buffer.map_async(map_mode, move |r| {
			var1.set(r);
		});
		var.await
    }

    pub fn align_copy_bytes_per_row(row_bytes: usize) -> usize {
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - row_bytes % align) % align;
        row_bytes + padded_bytes_per_row_padding
    }
}

pub struct Binding(usize);
