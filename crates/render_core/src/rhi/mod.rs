//! RHI: 渲染硬件接口 [Render Hardward Interface]
//! 对 wgpu 的 封装
//! 封装成 可 Clone 的 API 对象

pub mod bind_group;
pub mod buffer;
pub mod device;
pub mod options;
pub mod pipeline;
pub mod shader;
pub mod texture;
pub mod uniform_vec;

use self::{device::RenderDevice, options::RenderOptions};
use crate::rhi::options::RenderPriority;
use log::{debug, info};
use pi_ecs::prelude::World;
use pi_share::ShareRefCell;
use std::{ops::Deref, sync::Arc};

pub use render_crevice::*;
pub use wgpu::{
    util::BufferInitDescriptor,
    AddressMode,
    // Uniform
    BindGroupDescriptor,
    BindGroupEntry,
    BindGroupLayoutDescriptor,
    BindGroupLayoutEntry,
    BindingResource,

    BindingType,
    BlendComponent,
    BlendFactor,
    BlendOperation,
    BlendState,

    BufferAddress,
    BufferBinding,
    BufferBindingType,
    BufferDescriptor,
    BufferSize,
    BufferUsages,

    ColorTargetState,
    ColorWrites,
    CommandEncoder,
    CommandEncoderDescriptor,
    CompareFunction,
    ComputePassDescriptor,
    // Pipeline
    ComputePipelineDescriptor,
    DepthBiasState,
    DepthStencilState,
    Extent3d,
    Face,
    Features,
    // Sampler
    FilterMode,
    FrontFace,
    // Image
    ImageCopyBuffer,
    ImageCopyBufferBase,
    ImageCopyTexture,
    ImageCopyTextureBase,
    ImageDataLayout,
    ImageSubresourceRange,

    // Vertex
    IndexFormat,
    Limits,

    // Command
    LoadOp,
    // Buffer
    MapMode,
    MultisampleState,
    // State
    Operations,
    // Util
    Origin3d,
    PipelineLayout,
    PipelineLayoutDescriptor,

    PolygonMode,
    PresentMode,

    PrimitiveState,
    PrimitiveTopology,
    RenderPassColorAttachment,
    RenderPassDepthStencilAttachment,
    RenderPassDescriptor,
    SamplerBindingType,
    SamplerDescriptor,
    // Shader
    ShaderModule,
    ShaderModuleDescriptor,
    ShaderSource,
    ShaderStages,

    StencilFaceState,
    StencilOperation,
    StencilState,
    StorageTextureAccess,
    TextureAspect,

    TextureDescriptor,
    TextureDimension,
    TextureFormat,
    TextureSampleType,

    TextureUsages,
    // Texture
    TextureViewDescriptor,
    TextureViewDimension,
    VertexAttribute,
    VertexFormat,

    VertexStepMode,
};

pub type RenderQueue = Arc<wgpu::Queue>;
pub type RenderInstance = wgpu::Instance;

/// 初始化 渲染 环境
/// world 加入 Res: RenderInstance, RenderQueue, RenderDevice, RenderOptions, AdapterInfo
pub async fn setup_render_context(
    mut world: World,
    options: RenderOptions,
    window: ShareRefCell<winit::window::Window>,
) {
    let backends = options.backends;

    let instance = wgpu::Instance::new(backends);
    let surface = unsafe { instance.create_surface(window.deref()) };
    let request_adapter_options = wgpu::RequestAdapterOptions {
        power_preference: options.power_preference,
        compatible_surface: Some(&surface),
        ..Default::default()
    };
    let (device, queue, adapter_info) =
        initialize_renderer(&instance, &options, &request_adapter_options).await;

    debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
    debug!("Configured wgpu adapter Features: {:#?}", device.features());

    world.insert_resource(instance);
    world.insert_resource(options);
    world.insert_resource(device);
    world.insert_resource(queue);
    world.insert_resource(adapter_info);
}

/// Initializes the renderer by retrieving and preparing the GPU instance, device and queue
/// for the specified backend.
async fn initialize_renderer(
    instance: &wgpu::Instance,
    options: &RenderOptions,
    request_adapter_options: &wgpu::RequestAdapterOptions<'_>,
) -> (RenderDevice, RenderQueue, wgpu::AdapterInfo) {
    let adapter = instance
        .request_adapter(request_adapter_options)
        .await
        .expect("Unable to find a GPU! Make sure you have installed required drivers!");

    let adapter_info = adapter.get_info();
    info!("initialize_renderer {:?}", adapter_info);

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
            features -= wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
        }
        limits = adapter.limits();
    }

    // Enforce the disabled features
    if let Some(disabled_features) = options.disabled_features {
        features -= disabled_features;
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
        };
    }

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: options.device_label.as_ref().map(|a| a.as_ref()),
                features,
                limits,
            },
            trace_path,
        )
        .await
        .unwrap();
    let device = Arc::new(device);
    let queue = Arc::new(queue);

    (RenderDevice::from(device), queue, adapter_info)
}
