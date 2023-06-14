
use std::sync::Arc;

use pi_assets::{mgr::AssetMgr, asset::{GarbageEmpty, Handle}};
use pi_async::rt::serial::AsyncRuntime;
use pi_atom::Atom;

use pi_share::{Share, ShareRwLock};
use render_core::{
    rhi::{device::RenderDevice, options::{RenderOptions, RenderPriority}, RenderQueue, asset::TextureRes},
    renderer::{
        bind_buffer::BindBufferAllocator,
        sampler::SamplerRes,
        buildin_data::{DefaultTexture, EDefaultTexture},
        bind_group::{BindGroupLayout, BindGroupUsage, BindGroup, BindsRecorder},
        attributes::{ShaderAttribute, EVertexDataKind, KeyShaderFromAttributes},
        shader_stage::EShaderStage, shader::TShaderSetBlock
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

pub fn setup_render_context(
    options: RenderOptions,
    window: Arc<winit::window::Window>,
) -> (RenderDevice, RenderQueue, wgpu::AdapterInfo) {
    let backends = options.backends;

    
    let runtime = pi_async::rt::serial::AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut result: Share<ShareRwLock<Option<(RenderDevice, RenderQueue, wgpu::AdapterInfo)>>> = Share::new(ShareRwLock::new(None));
    
    let result1 = result.clone();
    let rt = runtime.clone();

    let _ = runtime.spawn(runtime.alloc(), async move {
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

        *result1.write() = Some(
            (device, queue, adapter_info)
        );
    });

    loop {
        if result.read().is_some() {
            match Share::try_unwrap(result) {
                Ok(r) => {
                    let (device, queue, adapter_info) = r.into_inner().unwrap();

                    return {
                        let boxed = Box::new(
                            (device, queue, adapter_info)
                        );
                        *boxed
                    };
                }
                Err(r) => result = r,
            }
        }
    }
}

// pub struct BufferLimit;
// impl TMemoryAllocatorLimit for BufferLimit {
//     fn max_size(&self) -> u64 {
//         1 * 1024 * 1024 * 1024
//     }
// }


pub(crate) fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    
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

    let (device, queue, adapter_info) = setup_render_context(
        options,
        window
    );
    

    // let mut bind_buffer_allocator = DynMergyBufferAllocator::new(&BufferLimit, 1 * 1024 * 1024);
    let mut bind_buffer_allocator = BindBufferAllocator::new(&device);
    let mut asset_shader_meta: Share<AssetMgr<ShaderEffectMeta>> = AssetMgr::<ShaderEffectMeta>::new(GarbageEmpty(), false, 60 * 1024 * 1024, 1000);
    let mut asset_tex: Share<AssetMgr<TextureRes>> = AssetMgr::<TextureRes>::new(GarbageEmpty(), false, 60 * 1024 * 1024, 1000);
    let mut asset_sampler: Share<AssetMgr<SamplerRes>> = AssetMgr::<SamplerRes>::new(GarbageEmpty(), false, 60 * 1024 * 1024, 1000);
    let mut asset_bind_group: Share<AssetMgr<BindGroup>> = AssetMgr::<BindGroup>::new(GarbageEmpty(), false, 60 * 1024 * 1024, 1000);
    let mut asset_bind_group_layout: Share<AssetMgr<BindGroupLayout>> = AssetMgr::<BindGroupLayout>::new(GarbageEmpty(), false, 60 * 1024 * 1024, 1000);
    let mut bindsrecorder = BindsRecorder::new();

    let white = DefaultTexture::create(&device, &queue, EDefaultTexture::White, wgpu::TextureDimension::D2);
    let texture = white.create_view(&wgpu::TextureViewDescriptor {
        label: None,
        format: Some(wgpu::TextureFormat::Rgba8Unorm),
        dimension: Some(wgpu::TextureViewDimension::D2),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });
    asset_tex.insert(Atom::from(DefaultTexture::WHITE_2D).get_hash() as u64, TextureRes::new(1, 1, 4, texture, true, wgpu::TextureFormat::Rgba8Unorm));

    let textures = vec![
        UniformTexture2DDesc::new(
            Atom::from("_EmissiveTex"), 
            wgpu::TextureSampleType::Float { filterable: true}, 
            false, 
            EShaderStage::FRAGMENT,
            EDefaultTexture::White,
        ),
        UniformTexture2DDesc::new(
            Atom::from("_MainTex"), 
            wgpu::TextureSampleType::Float { filterable: true}, 
            false, 
            EShaderStage::FRAGMENT,
            EDefaultTexture::White,
        ),
        UniformTexture2DDesc::new(
            Atom::from("_BoneTex"), 
            wgpu::TextureSampleType::Float { filterable: true}, 
            false, 
            EShaderStage::VERTEXFRAGMENT,
            EDefaultTexture::White,
        ),
    ];
    let valuedesc = MaterialValueBindDesc {
        stage: wgpu::ShaderStages::VERTEX_FRAGMENT,
        mat4_list: vec![UniformPropertyMat4(Atom::from("emissiveMatrics"), [0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,])],
        mat2_list: vec![],
        vec4_list: vec![UniformPropertyVec4(Atom::from("emissiveColor"), [1., 1., 1., 1.])],
        vec2_list: vec![],
        float_list: vec![],
        int_list: vec![],
        uint_list: vec![],
    };

    let shader_meta = ShaderEffectMeta::new(
        valuedesc,
        textures,
        Varyings(
            vec![
                Varying { 
                    format: Atom::from("vec3"),
                    name: Atom::from("v_normal"),
                },
                Varying { 
                    format: Atom::from("vec3"),
                    name: Atom::from("v_pos"),
                },
            ]
        ),
        BlockCodeAtom { 
            define: Atom::from(""), 
            running: Atom::from("
vec3 position = A_POSITION;
vec3 normal = A_NORMAL;
mat4 finalWorld = PI_ObjectToWorld;

vec4 positionUpdate =  vec4(position, 1.);
vec4 worldPos =  finalWorld * positionUpdate;
// vec4 worldPos =  positionUpdate;

gl_Position = PI_MATRIX_VP * worldPos;
// gl_Position = positionUpdate;

v_pos = worldPos.xyz;

mat3 normalWorld = mat3(finalWorld);
v_normal = normal; // normalize(vec3(finalWorld * vec4(normal, 1.0)));
")
        },
        BlockCodeAtom { 
            define: Atom::from("layout(location = 0) out vec4 gl_FragColor;\r\n"), 
            running: Atom::from("
vec4 baseColor = vec4(1., 1., 1., 1.);

baseColor.rgb *= emissiveColor.rgb * emissiveColor.a;

float alpha = 1.0;

// float level = dot(v_normal, vec3(0., 0., -1.));
baseColor.rgb = mix(baseColor.rgb, v_normal, 0.5);
// baseColor.rgb = (v_pos + vec3(1., 1., 1.)) / 2.;

gl_FragColor = vec4(baseColor.rgb, alpha);
")
        },
        ShaderDefinesSet::default()
    );

    let key_meta = Atom::from("TTest");
    let shader_meta = asset_shader_meta.insert(key_meta.clone(), shader_meta).unwrap();
    let varyings = Varyings::default();

    let useinfo = shader_meta.textures.use_info(vec![]);
    println!("useinfo: {:?}", useinfo);
    println!("_BoneTex Slot: {:?}", shader_meta.textures.binary_search_by(|a| { a.slotname.cmp(&Atom::from("_BoneTex")) } ));
    println!("_BoneTex Slot: {:?}", shader_meta.textures.binary_search_by(|a| { a.slotname.cmp(&Atom::from("_MainTex")) } ));

    // let effect_bind_textures = EffectTextureSamples::new(&useinfo);
    // let bindgroup_texure_samplers = BindGroupTextureSamplers::new(
    //     &shader_meta, 
    //     (
    //         Some(())
    //     ), 
    //     (
            
    //     ),
    //     &device, 
    //     &asset_bind_group_layout,
    //     &asset_bind_group
    // ).unwrap();

    // println!("texdesc.vs_code ");
    // println!("{}", bindgroup_texure_samplers.vs_define_code());
    // println!("texdesc.fs_code ");
    // println!("{}", bindgroup_texure_samplers.fs_define_code());

    let valuedesc = &shader_meta.uniforms;
    let bind_model: Arc<ShaderBindModelAboutMatrix> = Arc::new(ShaderBindModelAboutMatrix::new(&mut bind_buffer_allocator).unwrap());
    let bind_effect_value: Arc<ShaderBindEffectValue> = Arc::new(ShaderBindEffectValue::new(&device, key_meta.clone(), shader_meta.clone(), &mut bind_buffer_allocator).unwrap());
    bind_model.data().write_data(0, &[0, 0, 0, 0]);
    bind_effect_value.data().write_data(0, &[0, 0, 0, 0]);


    // let key_scene = KeyShaderSceneAbout { fog: true, brdf: false, env: false };
    let bind_camera = Arc::new(ShaderBindSceneAboutBase::new(&mut bind_buffer_allocator).unwrap());
    let bind_effect = Arc::new(ShaderBindSceneAboutEffect::new(&mut bind_buffer_allocator).unwrap());
    bind_camera.data().write_data(0, &[0, 0, 0, 0]);
    bind_effect.data().write_data(0, &[0, 0, 0, 0]);
    
    bind_buffer_allocator.write_buffer(&device, &queue);
    // write buffer, before bindgroup

    let key = KeyBindGroupModel::new(
        bind_model,
        None,
        Some(bind_effect_value),
        &mut bindsrecorder
    );
    let key_bind_group = key.key_bind_group();
    let key_bind_group_layout = key.key_bind_group_layout();
    let bind_group_layout = BindGroupLayout::new(&device, &key_bind_group_layout);
    let bind_group_layout = asset_bind_group_layout.insert(key_bind_group_layout.asset_u64(), bind_group_layout).unwrap();
    let bind_group = BindGroup::new(&device, &key_bind_group, bind_group_layout);
    let bind_group = asset_bind_group.insert(key_bind_group.asset_u64(), bind_group).unwrap();
    let bindgroup_model: BindGroupModel = BindGroupModel::new(BindGroupUsage::new(1, key_bind_group, bind_group), key);

    let key = KeyBindGroupScene::new(
        bind_camera, 
        Some(bind_effect),
        &mut bindsrecorder
    );
    let key_bind_group = key.key_bind_group();
    let key_bind_group_layout = key.key_bind_group_layout();
    let bind_group_layout = BindGroupLayout::new(&device, &key_bind_group_layout);
    let bind_group_layout = asset_bind_group_layout.insert(key_bind_group_layout.asset_u64(), bind_group_layout).unwrap();
    let bind_group = BindGroup::new(&device, &key_bind_group, bind_group_layout);
    let bind_group = asset_bind_group.insert(key_bind_group.asset_u64(), bind_group).unwrap();
    let bindgroup_scene = BindGroupScene::new(BindGroupUsage::new(0, key_bind_group, bind_group), key);


    let meshdes = vec![
        ShaderAttribute { kind: EVertexDataKind::Position, location: 0 },
        ShaderAttribute { kind: EVertexDataKind::Normal, location: 1 },
    ];
    // let shader = shader_meta.build::<BindGroupScene, BindGroupModel, BindGroupTextureSamplers>(
    //     &device,
    //     &key_meta,
    //     &KeyShaderFromAttributes(meshdes),
    //     &EInstanceCode(EInstanceCode::NONE),
    //     &ESkinCode::None,
    //     &bindgroup_scene,
    //     &bindgroup_model,
    //     None,
    //     None,
    // );
    let mut vs_defines = vec![];
    vs_defines.push(bindgroup_scene.vs_define_code());
    vs_defines.push(bindgroup_model.vs_define_code());
    let mut fs_defines = vec![];
    fs_defines.push(bindgroup_scene.fs_define_code());
    fs_defines.push(bindgroup_model.fs_define_code());

    let shader = shader_meta.build_2(
        &device,
        &key_meta,
        &KeyShaderFromAttributes(meshdes),
        &EInstanceCode(EInstanceCode::NONE),
        &ERenderAlignment::Facing,
        &ESkinCode::None,
        &vs_defines,
        &vec![],
        &vec![],
        &fs_defines,
        &vec![],
        &vec![],
    );


    println!("Shader End.")
}