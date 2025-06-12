//! RHI: 渲染硬件接口 [Render Hardward Interface]
//! 对 wgpu 的 封装
//! 封装成 可 Clone 的 API 对象

pub mod asset;
pub mod bind_group;
pub mod bind_group_layout;
pub mod block_alloc;
pub mod buffer;
pub mod device;
pub mod dyn_uniform_buffer;
pub mod options;
pub mod pipeline;
pub mod shader;
pub mod texture;
pub mod uniform_vec;
pub mod draw_obj;
pub mod sampler;
pub mod small_struct_allocator;
pub mod buffer_alloc;
pub mod id_alloter;

use self::device::RenderDevice;
use pi_share::Share;

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
    // ComputePassDescriptor,
    // Pipeline
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
    // ImageCopyBufferBase,
    ImageCopyTexture,
    // ImageCopyTextureBase,
    ImageDataLayout,
    ImageSubresourceRange,

    // Vertex
    IndexFormat,
    Limits,

    // Command
    LoadOp,
    // Buffer
    // MapMode,
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

pub type RenderQueue = Share<wgpu::Queue>;
pub type RenderInstance = wgpu::Instance;
pub type AdapterInfo = wgpu::AdapterInfo;
