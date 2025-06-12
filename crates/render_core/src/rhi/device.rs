use super::{
    bind_group::BindGroup,
    bind_group_layout::BindGroupLayout,
    buffer::Buffer,
    pipeline::RenderPipeline,
    texture::{Sampler, Texture}, options::{RenderOptions, RenderPriority}, RenderQueue,
};
use derive_deref_rs::Deref;
use pi_assets::allocator::Allocator;
use pi_share::Share;
use wgpu::{util::DeviceExt, PiWgpuAdapter};

/// This GPU device is responsible for the creation of most rendering and compute resources.
#[derive(Clone, Deref)]
pub struct RenderDevice(pub Share<wgpu::Device>);

// TODO Send问题， 临时解决
unsafe impl Send for RenderDevice {}
unsafe impl Sync for RenderDevice {}

impl From<Share<wgpu::Device>> for RenderDevice {
    fn from(device: Share<wgpu::Device>) -> Self {
        log::info!("=============== limits = {:?}", device.limits());

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
    // #[inline]
    // pub fn poll(&self, maintain: wgpu::Maintain) -> bool {
    //     self.0.poll(maintain)
    // }

    /// Creates an empty [`CommandEncoder`](wgpu::CommandEncoder).
    #[inline]
    pub fn create_command_encoder(
        &self,
        desc: &wgpu::CommandEncoderDescriptor,
    ) -> wgpu::CommandEncoder {
        self.0.create_command_encoder(desc)
    }

    /// Creates an empty [`RenderBundleEncoder`](wgpu::RenderBundleEncoder).
    // #[inline]
    // pub fn create_render_bundle_encoder(
    //     &self,
    //     desc: &wgpu::RenderBundleEncoderDescriptor,
    // ) -> wgpu::RenderBundleEncoder {
    //     self.0.create_render_bundle_encoder(desc)
    // }

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
    // #[inline]
    // pub fn create_compute_pipeline(
    //     &self,
    //     desc: &wgpu::ComputePipelineDescriptor,
    // ) -> ComputePipeline {
    //     let wgpu_compute_pipeline = self.0.create_compute_pipeline(desc);
    //     ComputePipeline::from(wgpu_compute_pipeline)
    // }

    /// Creates a [`Buffer`].
    pub fn create_buffer(&self, desc: &wgpu::BufferDescriptor) -> Buffer {
        let wgpu_buffer = self.0.create_buffer(desc);
        Buffer::from((wgpu_buffer, desc.size))
    }

    /// Creates a [`Buffer`] and initializes it with the specified data.
    pub fn create_buffer_with_data(&self, desc: &wgpu::util::BufferInitDescriptor) -> Buffer {
        let wgpu_buffer = self.0.create_buffer_init(desc);
        Buffer::from((wgpu_buffer, desc.contents.len() as wgpu::BufferAddress))
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

    // pub async fn map_buffer(
    //     &self,
    //     buffer: &wgpu::BufferSlice<'_>,
    //     map_mode: wgpu::MapMode,
    // ) -> Result<(), BufferAsyncError> {
	// 	let var = pi_async_rt::prelude::AsyncValue::new();
	// 	let var1 = var.clone();
    //     buffer.map_async(map_mode, move |r| {
	// 		var1.set(r);
	// 	});
	// 	var.await
    // }

    pub fn align_copy_bytes_per_row(row_bytes: usize) -> usize {
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - row_bytes % align) % align;
        row_bytes + padded_bytes_per_row_padding
    }
}

pub async fn initialize_renderer(
	instance: &wgpu::Instance,
	options: &RenderOptions,
	request_adapter_options: &wgpu::RequestAdapterOptions<'_, '_>,
	alloter: &mut Allocator,
) -> (RenderDevice, RenderQueue, wgpu::AdapterInfo) {
	let adapter = instance
		.request_adapter(request_adapter_options)
		.await
		.expect("Unable to find a GPU! Make sure you have installed required drivers!");

	let adapter_info = adapter.get_info();

	// #[cfg(not(feature = "trace"))]
	let trace_path = None;

	// Maybe get features and limits based on what is supported by the adapter/backend
	let mut features = wgpu::Features::empty();
	let mut limits = options.limits.clone();
	if matches!(options.priority, RenderPriority::Functionality) {
		features = adapter.features() | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
		if adapter_info.device_type == wgpu::DeviceType::DiscreteGpu {
			// `MAPPABLE_PRIMARY_BUFFERS` can have a significant, negative performance impact for
			// discrete GPUs due to having to transfer data across the PCI-E bus and so it
			// should not be automatically enabled in this case. It is however beneficial for
			// integrated GPUs.
			features = features & (features ^ wgpu::Features::MAPPABLE_PRIMARY_BUFFERS);
		}
		limits = adapter.limits();
	}

	// Enforce the disabled features
	if let Some(disabled_features) = options.disabled_features {
		features = features & (features ^ disabled_features);
	}
	// NOTE: |= is used here to ensure that any explicitly-enabled features are respected.
	features |= options.features;

	// Enforce the limit constraints
	if let Some(constrained_limits) = options.constrained_limits.as_ref() {
		// NOTE: Respect the configured limits as an 'upper bound'. This means for 'max' limits, we
		// take the minimum of the calculated limits according to the adapter/backend and the
		// specified max_limits. For 'min' limits, take the maximum instead. This is intended to
		// err on the side of being conservative. We can't claim 'higher' limits that are supported
		// but we can constrain to 'lower' limits.
		limits = wgpu::Limits {
			max_texture_dimension_1d: limits
				.max_texture_dimension_1d
				.min(constrained_limits.max_texture_dimension_1d),
			max_texture_dimension_2d: limits
				.max_texture_dimension_2d
				.min(constrained_limits.max_texture_dimension_2d),
			max_texture_dimension_3d: limits
				.max_texture_dimension_3d
				.min(constrained_limits.max_texture_dimension_3d),
			max_texture_array_layers: limits
				.max_texture_array_layers
				.min(constrained_limits.max_texture_array_layers),
			max_bind_groups: limits
				.max_bind_groups
				.min(constrained_limits.max_bind_groups),
			max_dynamic_uniform_buffers_per_pipeline_layout: limits
				.max_dynamic_uniform_buffers_per_pipeline_layout
				.min(constrained_limits.max_dynamic_uniform_buffers_per_pipeline_layout),
			max_dynamic_storage_buffers_per_pipeline_layout: limits
				.max_dynamic_storage_buffers_per_pipeline_layout
				.min(constrained_limits.max_dynamic_storage_buffers_per_pipeline_layout),
			max_sampled_textures_per_shader_stage: limits
				.max_sampled_textures_per_shader_stage
				.min(constrained_limits.max_sampled_textures_per_shader_stage),
			max_samplers_per_shader_stage: limits
				.max_samplers_per_shader_stage
				.min(constrained_limits.max_samplers_per_shader_stage),
			max_storage_buffers_per_shader_stage: limits
				.max_storage_buffers_per_shader_stage
				.min(constrained_limits.max_storage_buffers_per_shader_stage),
			max_storage_textures_per_shader_stage: limits
				.max_storage_textures_per_shader_stage
				.min(constrained_limits.max_storage_textures_per_shader_stage),
			max_uniform_buffers_per_shader_stage: limits
				.max_uniform_buffers_per_shader_stage
				.min(constrained_limits.max_uniform_buffers_per_shader_stage),
			max_uniform_buffer_binding_size: limits
				.max_uniform_buffer_binding_size
				.min(constrained_limits.max_uniform_buffer_binding_size),
			max_storage_buffer_binding_size: limits
				.max_storage_buffer_binding_size
				.min(constrained_limits.max_storage_buffer_binding_size),
			max_vertex_buffers: limits
				.max_vertex_buffers
				.min(constrained_limits.max_vertex_buffers),
			max_vertex_attributes: limits
				.max_vertex_attributes
				.min(constrained_limits.max_vertex_attributes),
			max_vertex_buffer_array_stride: limits
				.max_vertex_buffer_array_stride
				.min(constrained_limits.max_vertex_buffer_array_stride),
			max_push_constant_size: limits
				.max_push_constant_size
				.min(constrained_limits.max_push_constant_size),
			min_uniform_buffer_offset_alignment: limits
				.min_uniform_buffer_offset_alignment
				.max(constrained_limits.min_uniform_buffer_offset_alignment),
			min_storage_buffer_offset_alignment: limits
				.min_storage_buffer_offset_alignment
				.max(constrained_limits.min_storage_buffer_offset_alignment),
			max_inter_stage_shader_components: limits
				.max_inter_stage_shader_components
				.min(constrained_limits.max_inter_stage_shader_components),
			max_compute_workgroup_storage_size: limits
				.max_compute_workgroup_storage_size
				.min(constrained_limits.max_compute_workgroup_storage_size),
			max_compute_invocations_per_workgroup: limits
				.max_compute_invocations_per_workgroup
				.min(constrained_limits.max_compute_invocations_per_workgroup),
			max_compute_workgroup_size_x: limits
				.max_compute_workgroup_size_x
				.min(constrained_limits.max_compute_workgroup_size_x),
			max_compute_workgroup_size_y: limits
				.max_compute_workgroup_size_y
				.min(constrained_limits.max_compute_workgroup_size_y),
			max_compute_workgroup_size_z: limits
				.max_compute_workgroup_size_z
				.min(constrained_limits.max_compute_workgroup_size_z),
			max_compute_workgroups_per_dimension: limits
				.max_compute_workgroups_per_dimension
				.min(constrained_limits.max_compute_workgroups_per_dimension),
			max_buffer_size: limits
				.max_buffer_size
				.min(constrained_limits.max_buffer_size),
			max_bindings_per_bind_group: limits
				.max_bindings_per_bind_group
				.min(constrained_limits.max_bindings_per_bind_group),
			max_non_sampler_bindings: limits
			.max_non_sampler_bindings
			.min(constrained_limits.max_non_sampler_bindings),
			max_binding_array_elements_per_shader_stage: limits.max_binding_array_elements_per_shader_stage.min(constrained_limits.max_binding_array_elements_per_shader_stage),
			max_binding_array_sampler_elements_per_shader_stage: limits.max_binding_array_sampler_elements_per_shader_stage.min(constrained_limits.max_binding_array_sampler_elements_per_shader_stage),
			max_color_attachments: limits.max_color_attachments.min(constrained_limits.max_color_attachments),
			max_color_attachment_bytes_per_sample: limits.max_color_attachment_bytes_per_sample.min(constrained_limits.max_color_attachment_bytes_per_sample),
			min_subgroup_size: limits.min_subgroup_size.max(constrained_limits.min_subgroup_size),
			max_subgroup_size: limits.max_subgroup_size.min(constrained_limits.max_subgroup_size),
		};
	}

	let (device, queue) = PiWgpuAdapter::request_device(
			&adapter,
			&wgpu::DeviceDescriptor {
				label: options.device_label.as_ref().map(|a| a.as_ref()),
				required_features: features,
				required_limits: limits,
				memory_hints: wgpu::MemoryHints::default(),
				trace: wgpu::Trace::default(),
			},
			trace_path,
		)
		.await
		.unwrap();
	let device = Share::new(device);
	let queue = Share::new(queue);

	(RenderDevice::from(device), queue, adapter_info)
}
