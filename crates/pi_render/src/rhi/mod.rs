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

use log::{debug, info};
use raw_window_handle::HasRawWindowHandle;
use std::sync::Arc;

pub use pi_crevice::*;

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

use crate::rhi::options::RenderPriority;

use self::{device::RenderDevice, options::RenderOptions};

/// This queue is used to enqueue tasks for the GPU to execute asynchronously.
pub type RenderQueue = Arc<wgpu::Queue>;

/// The GPU instance is used to initialize the [`RenderQueue`] and [`RenderDevice`],
/// aswell as to create [`WindowSurfaces`](crate::view::window::WindowSurfaces).
pub type RenderInstance = wgpu::Instance;

// TODO
// 初始化 渲染 环境
pub async fn create_render_context(
    window: &impl HasRawWindowHandle,
    mut options: RenderOptions,
) -> (RenderDevice, RenderQueue, RenderOptions) {
    let instance = wgpu::Instance::new(options.backends);

    let surface = unsafe { instance.create_surface(window) };

    let request_adapter_options = wgpu::RequestAdapterOptions {
        power_preference: options.power_preference,
        compatible_surface: Some(&surface),
        ..Default::default()
    };

    let adapter = instance
        .request_adapter(&request_adapter_options)
        .await
        .expect("Unable to find a GPU! Make sure you have installed required drivers!");

    let adapter_info = adapter.get_info();
    info!("init_render: adapter_info = {:?}", adapter_info);

    let trace_path = None;

    if matches!(options.priority, RenderPriority::Functionality) {
        let mut features =
            adapter.features() | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
        if adapter_info.device_type == wgpu::DeviceType::DiscreteGpu {
            // `MAPPABLE_PRIMARY_BUFFERS` can have a significant, negative performance impact for
            // discrete GPUs due to having to transfer data across the PCI-E bus and so it
            // should not be automatically enabled in this case. It is however beneficial for
            // integrated GPUs.
            features -= wgpu::Features::MAPPABLE_PRIMARY_BUFFERS;
        }
        options.features = features;
        options.limits = adapter.limits();
    }

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: options.device_label.as_ref().map(|a| a.as_ref()),
                features: options.features,
                limits: options.limits.clone(),
            },
            trace_path,
        )
        .await
        .unwrap();

    let device = Arc::new(device);

    let queue = Arc::new(queue);

    debug!("init_render: wgpu limits: {:#?}", &options.limits);

    debug!("init_render: wgpu features: {:#?}", &options.features);

    (RenderDevice::from(device), queue, options)
}
