
use std::sync::Arc;

use pi_assets::{mgr::AssetMgr, asset::{GarbageEmpty, Handle}};
use pi_async::rt::serial::AsyncRuntime;
use pi_atom::Atom;

use pi_share::{Share, ShareRwLock};
use render_core::{
    rhi::{device::RenderDevice, initialize_renderer, options::RenderOptions, RenderQueue, asset::TextureRes, shader::ShaderMeta, bind_group::BindGroup},
    renderer::{
        bind_buffer::BindBufferAllocator,
        sampler::SamplerRes,
        buildin_data::{DefaultTexture, EDefaultTexture},
        bind_group::{BindGroupLayout, BindGroupUsage},
        attributes::{ShaderAttribute, EVertexDataKind, KeyShaderFromAttributes},
        shader_stage::EShaderStage
    },
    render_3d::{
        shader::{
            shader_effect_meta::ShaderEffectMeta,
            uniform_texture::UniformTexture2DDesc,
            uniform_value::{UniformPropertyMat4, UniformPropertyVec4, MaterialValueBindDesc},
            varying_code::{Varyings, Varying},
            block_code::BlockCodeAtom,
            shader_defines::ShaderDefinesSet,
            shader::Shader3D,
            instance_code::EInstanceCode
        },
        bind_groups::{
            texture_sampler::{BindGroupTextureSamplers},
            model::{BindGroupModel},
            scene::{BindGroupScene}
        },
        binds::{
            model::base::ShaderBindModelAboutMatrix,
            effect_value::ShaderBindEffectValue,
            scene::{base::ShaderBindSceneAboutBase, effect::ShaderBindSceneAboutEffect},
            effect_texture2d::EffectBindTexture2D01
        }
    }
};
use wgpu::{Device, Instance};

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
        let instance = wgpu::Instance::new(backends);
        let surface = unsafe { instance.create_surface(window.as_ref()) };
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
    let instance = wgpu::Instance::new(backends);
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
    asset_tex.insert(Atom::from(DefaultTexture::WHITE_2D).get_hash() as u64, TextureRes::new(1, 1, 4, texture, true));

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

    let bindgroup_model: BindGroupModel = BindGroupModel::new(
        bind_model,
        None,
        Some(bind_effect_value),
        &device,
        &asset_bind_group_layout,
        &asset_bind_group,
    ).unwrap();
    let bindgroup_scene = BindGroupScene::new(
        bind_camera, 
        Some(bind_effect),
        &device, 
        &asset_bind_group_layout, 
        &asset_bind_group,
    ).unwrap();

    let meshdes = vec![
        ShaderAttribute { kind: EVertexDataKind::Position, location: 0 },
        ShaderAttribute { kind: EVertexDataKind::Normal, location: 1 },
    ];
    let shader = shader_meta.build::<BindGroupScene, BindGroupModel, BindGroupTextureSamplers>(
        &device,
        &key_meta,
        &KeyShaderFromAttributes(meshdes),
        &EInstanceCode(EInstanceCode::NONE),
        &bindgroup_scene,
        &bindgroup_model,
        None,
        None,
    );
}