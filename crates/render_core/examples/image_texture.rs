
use std::{sync::Arc, thread::sleep, time::Duration};

use pi_assets::{mgr::{AssetMgr, LoadResult}, asset::{GarbageEmpty, Handle}};
use pi_async_rt::rt::{ AsyncRuntime};
use pi_atom::Atom;

use pi_hal::{loader::AsyncLoader, runtime::MULTI_MEDIA_RUNTIME, init_load_cb, on_load};
use pi_share::{Share, ShareRwLock};
use render_core::{
    rhi::{device::RenderDevice, options::{RenderOptions, RenderPriority}, RenderQueue, asset::TextureRes},
    renderer::{
        bind_buffer::BindBufferAllocator,
        sampler::SamplerRes,
        buildin_data::{DefaultTexture, EDefaultTexture},
        bind_group::{BindGroupLayout, BindGroupUsage, BindGroup, BindsRecorder},
        attributes::{ShaderAttribute, EVertexDataKind, KeyShaderFromAttributes},
        shader_stage::EShaderStage, texture::{ImageTexture, ImageTexture2DDesc, KeyImageTexture}
    },
    render_3d::{
        shader::*,
        bind_groups::{
            texture_sampler::{BindGroupTextureSamplers},
            model::{BindGroupModel, KeyBindGroupModel},
            scene::{BindGroupScene, KeyBindGroupScene}
        },
        binds::{
            model::*,
            effect_value::ShaderBindEffectValue,
            scene::*,
        }
    }, asset::TAssetKeyU64
};
use wgpu::{Device, Instance};

/// Initializes the renderer by retrieving and preparing the GPU instance, device and queue
/// for the specified backend.
pub async fn initialize_renderer(
    instance: &wgpu::Instance,
    options: &RenderOptions,
    request_adapter_options: &wgpu::RequestAdapterOptions<'_>,
) -> (RenderDevice, RenderQueue, wgpu::AdapterInfo) {
    let adapter = instance
        .request_adapter(request_adapter_options)
        .await
        .expect("Unable to find a GPU! Make sure you have installed required drivers!");

    let adapter_info = adapter.get_info();
    log::warn!("initialize_renderer {:?}", adapter_info);

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
			max_buffer_size: limits
				.max_buffer_size
				.min(constrained_limits.max_buffer_size),
			max_bindings_per_bind_group: limits
				.max_bindings_per_bind_group
				.min(constrained_limits.max_bindings_per_bind_group),
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
    let device = Share::new(device);
    let queue = Share::new(queue);

    (RenderDevice::from(device), queue, adapter_info)
}

pub async fn  setup_render_context(
    options: RenderOptions,
    window: Arc<winit::window::Window>,
) -> (RenderDevice, RenderQueue, wgpu::AdapterInfo) {
    let backends = options.backends;

    
    // let runtime = pi_async_rt::rt::serial::AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    // let mut result: Share<ShareRwLock<Option<(RenderDevice, RenderQueue, wgpu::AdapterInfo)>>> = Share::new(ShareRwLock::new(None));
    
    // let result1 = result.clone();
    // let rt = runtime.clone();

    // let _ = runtime.spawn(runtime.alloc(), async move {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
			/// Which `Backends` to enable.
			backends: options.backends,
			/// Which DX12 shader compiler to use.
			dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
		});
        let surface = unsafe { instance.create_surface(window.as_ref()).unwrap() };
        let request_adapter_options = wgpu::RequestAdapterOptions {
            power_preference: options.power_preference,
            compatible_surface: Some(&surface),
            ..Default::default()
        };
        
        log::debug!(">>>> render_graphic");
        let (device, queue, adapter_info) =
            initialize_renderer(&instance, &options, &request_adapter_options).await;
            

        log::debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
        log::debug!("Configured wgpu adapter Features: {:#?}", device.features());

        (device, queue, adapter_info)
}

pub(crate) fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    

    let event_loop = winit::event_loop::EventLoop::new();
    let window = Arc::new(winit::window::Window::new(&event_loop).unwrap());
    
    let options = RenderOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    };
    let backends = options.backends;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
		/// Which `Backends` to enable.
		backends: options.backends,
		/// Which DX12 shader compiler to use.
		dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
	});
    let surface = unsafe { instance.create_surface(window.as_ref()) };

    init_load_cb(Arc::new(|path: String| {
        MULTI_MEDIA_RUNTIME
            .spawn(MULTI_MEDIA_RUNTIME.alloc(), async move {
                log::debug!("Load {}", path);
                let r = std::fs::read(path.clone()).unwrap();
                on_load(&path, r);
            })
            .unwrap();
    }));

    MULTI_MEDIA_RUNTIME
    .spawn(MULTI_MEDIA_RUNTIME.alloc(), async move {
        let key = KeyImageTexture::File(String::from("E:/Rust/PI/pi_3d/assets/images/eff_ui_ll_085.png"), true);
        let (device, queue, adapter_info) = setup_render_context(
            options,
            window
        ).await;

        let mgr = AssetMgr::<ImageTexture>::new(GarbageEmpty(), false, 1024, 1000);

        let desc = ImageTexture2DDesc {
            url: key.clone(),
            device: device.clone(),
            queue: queue.clone(),
        };

        let result = AssetMgr::load(&mgr, &key);

        let r = ImageTexture::async_load(desc, result).await;
        match r {
            Ok(r) => {
                log::info!("load image success, {:?}", r.key());
            }
            Err(e) => {
                log::error!("load image fail, {:?}", e);
            }
        };
    })
    .unwrap();

    loop {
        sleep(Duration::new(1, 0));
        println!("Frame.");
    }
}