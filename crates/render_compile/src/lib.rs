use std::{
    borrow::Cow,
    collections::{hash_map::Entry, HashMap},
    intrinsics::transmute, path::Path, process::Command,
};

use inflector::Inflector;
use naga::{ScalarKind, StorageAccess, ShaderStage, Binding, AddressSpace};
use pi_ordmap::{sbtree::Tree, ordmap::{ImOrdMap, Iter, OrdMap}};
use thiserror::Error;

#[derive(PartialEq, Eq, Debug)]
pub enum ProcessedShader<'a> {
    Wgsl(Cow<'a, str>),
    Glsl(Cow<'a, str>, naga::ShaderStage),
    SpirV(Cow<'a, [u8]>),
}

pub fn compile_and_out(
	share_name: &str,
    vs_shader: ProcessedShader,
    fs_shader: ProcessedShader,
	out_dir: &Path) {
	let root_dir = std::env::current_dir().unwrap();

	let r = compile(
		share_name,
		vs_shader,
		fs_shader,
	);

	println!("r==========={:?}", r);

	let file_name = format!("{}.rs", share_name);
	match r {
		Ok(r) => std::fs::write(root_dir.join(out_dir).join(file_name.as_str()), r).unwrap(),
		Err(r) => log::error!("{:?}", r),
	};

	Command::new("rustfmt")
		.arg(format!("{}", file_name))
		.arg("--config")
		.arg("hard_tabs=true")
		.current_dir(root_dir.join(out_dir))
		.status()
		.unwrap();
}

pub fn compile(
    shader_name: &str,
    vs_shader: ProcessedShader,
    fs_shader: ProcessedShader,
) -> Result<String, ShaderReflectError> {
    let vs_module = match &vs_shader {
        ProcessedShader::Wgsl(source) => naga::front::wgsl::parse_str(source)?,
        ProcessedShader::Glsl(source, shader_stage) => {
            let mut parser = naga::front::glsl::Parser::default();
            parser
                .parse(&naga::front::glsl::Options::from(*shader_stage), source)
                .map_err(ShaderReflectError::GlslParse)?
        }
        ProcessedShader::SpirV(source) => naga::front::spv::parse_u8_slice(
            source,
            &naga::front::spv::Options {
                adjust_coordinate_space: false,
                ..naga::front::spv::Options::default()
            },
        )?,
    };
    let fs_module = match &fs_shader {
        ProcessedShader::Wgsl(source) => naga::front::wgsl::parse_str(source)?,
        ProcessedShader::Glsl(source, shader_stage) => {
            let mut parser = naga::front::glsl::Parser::default();
            parser
                .parse(&naga::front::glsl::Options::from(*shader_stage), source)
                .map_err(ShaderReflectError::GlslParse)?
        }
        ProcessedShader::SpirV(source) => naga::front::spv::parse_u8_slice(
            source,
            &naga::front::spv::Options {
                adjust_coordinate_space: false,
                ..naga::front::spv::Options::default()
            },
        )?,
    };
    println!("fs: {:?} {:?}", vs_module, fs_module);

	let mut input_strs = Vec::new();

	parse_input(&vs_module, &mut input_strs);

    let vs_uniforms = get_uniforms(&vs_module);
    let fs_uniforms = get_uniforms(&fs_module);

    let uniforms = merge_uniforms(
        &vs_uniforms,
        &fs_uniforms,
        &vs_module.types,
        &fs_module.types,
    );

    let mut functions = Vec::new();
	let mut buffer_uniforms = Vec::new();
	let mut buffer_bindings = Vec::new();
    let mut groups = Vec::new();
    for pi_ordmap::ordmap::Entry(group, bindings) in uniforms.iter(None, false) {
        let mut bindings_layout_info = Vec::new();
        let mut bindings_info = Vec::new();
        let mut group_params = Vec::new();
		let mut group_layout_params = Vec::new();

        let mut name1 = "".to_string();
		let mut i = 0;
		let mut is_all_buffer = true;
        for pi_ordmap::ordmap::Entry((bind, visibility), (item, var_name, from)) in bindings.iter(None, false) {
            let name = if let Some(r) = &item.name {
                r
            } else {
                match var_name {
                    Some(r) => r,
                    None => continue,
                }
            };
            name1 = name1 + "_" + name.to_snake_case().as_str();

            let mut buffer_info = Vec::new();
            let mut texture_sampler_info = Vec::new();
            let mut layout_info = LayoutInfo::default();
            get_layout_info(
                &item,
                &mut buffer_info,
                &mut texture_sampler_info,
                &mut layout_info,
                match from {
                    UniformFrom::Vert => &vs_module.types,
                    UniformFrom::Frag => &fs_module.types,
                },
            );

            let mut visibility_str = Vec::new();
            if visibility & (UniformFrom::Vert as usize) > 0 {
                visibility_str.push("wgpu::ShaderStages::VERTEX");
            }
            if visibility & (UniformFrom::Frag as usize) > 0 {
                visibility_str.push("wgpu::ShaderStages::FRAGMENT");
            }
			let mut tail = layout_info.size;

            for item in buffer_info.iter() {
                let ty = get_type(item.kind);

				let name = item.name.to_class_case();

				buffer_uniforms.push(format!("
				pub struct {}Uniform<'a>(pub &'a[{}]);
				impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for {}Uniform<'a> {{
					fn write_into(&self, index: u32, buffer: &mut [u8]) {{
						unsafe {{ std::ptr::copy_nonoverlapping(
							self.0.as_ptr() as usize as *const u8,
							buffer.as_mut_ptr().add(index as usize + {}),
							{},
						) }};
					}}
				}}
				",
                    name, ty, name, item.alignment, item.size * item.width
                ));

                // let name = item.name.to_snake_case();
                // // set_uniform
                // functions.push(format!(
                //     "
				// 	pub fn set_{}(&mut self, index: &pi_render::rhi::block_alloc::BlockIndex, {}: &[{}]) -> Result<(), String> {{
				// 		self.context.lock().unwrap().set_uniform(index, {}, bytemuck::cast_slice({}))
				// 	}}
				// ",
                //     name, name, ty, item.alignment, name
                // ));;
            }
			if buffer_info.len() > 0 {
				group_params.push("buffer: &wgpu::Buffer".to_string());
				group_layout_params.push("has_dynamic_offset: bool".to_string());
				let name = name.to_class_case();
				buffer_bindings.push(format!("
					pub struct {}Bind;
					impl pi_render::rhi::dyn_uniform_buffer::Bind for {}Bind {{
						#[inline]
						fn min_size() -> usize {{
							{}
						}}

						fn index() -> pi_render::rhi::dyn_uniform_buffer::BindIndex {{
							pi_render::rhi::dyn_uniform_buffer::BindIndex::new({})
						}}
					}}
				", name, name, tail, i));
				bindings_layout_info.push(format!(
                    "
					wgpu::BindGroupLayoutEntry {{
						binding: {},
						visibility: {},
						ty: wgpu::BindingType::Buffer {{
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset,
							min_binding_size: wgpu::BufferSize::new({}),
						}},
						count: None, // TODO
					}}
				",
                    bind,
                    visibility_str.join(" | "),
                    tail
                )); 
                bindings_info.push(format!(
                    "
					wgpu::BindGroupEntry {{
						binding: {},
						resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {{
							buffer,
							offset: 0,
							size: Some(std::num::NonZeroU64::new({}).unwrap())
						}}),
					}}
				",
                    bind,
                    tail
                ));
			}

            for item in texture_sampler_info.iter() {
                let name = name.to_snake_case();
				is_all_buffer = false;

                match &item.inner {
                    naga::TypeInner::Image {
                        dim,
                        class,
						..
                    } => {
                        let (sample_type, multi) = match class {
                            naga::ImageClass::Sampled { kind, multi } => (
                                format!(
                                    "wgpu::TextureSampleType::{}",
                                    match kind {
                                        ScalarKind::Sint => "Sint",
                                        ScalarKind::Uint => "Uint",
                                        ScalarKind::Float => "Float{filterable:true}",
                                        ScalarKind::Bool => continue,
                                    }
                                ),
                                multi,
                            ),
                            naga::ImageClass::Depth { multi } => {
                                ("wgpu::TextureSampleType::Depth".to_string(), multi)
                            }
                            naga::ImageClass::Storage { .. } => todo!(),
                        };
                        let dim = match dim {
                            naga::ImageDimension::D1 => "wgpu::TextureViewDimension::D1",
                            naga::ImageDimension::D2 => "wgpu::TextureViewDimension::D2",
                            naga::ImageDimension::D3 => "wgpu::TextureViewDimension::D2",
                            naga::ImageDimension::Cube => "wgpu::TextureViewDimension::Cube",
                        };

                        // wgpu::TextureSampleType

                        bindings_layout_info.push(format!(
                            "
							wgpu::BindGroupLayoutEntry {{
								binding: {},
								visibility:{},
								ty: wgpu::BindingType::Texture {{
									multisampled: {},
									sample_type: {},
									view_dimension: {},
								}},
								count: None, // TODO
							}}
						",
                            bind,
                            visibility_str.join(" | "),
                            multi,
                            sample_type,
                            dim
                        ));
                        bindings_info.push(format!(
                            "
							wgpu::BindGroupEntry {{
								binding: {},
								resource: wgpu::BindingResource::TextureView({}),
							}}
						",
                            bind, name
                        ));
                        group_params.push(format!("{}: &wgpu::TextureView", name));
                    }
                    naga::TypeInner::Sampler { comparison } => {
                        let filtering = if *comparison {
                            "wgpu::SamplerBindingType::Comparison"
                        } else {
                            "wgpu::SamplerBindingType::Filtering"
                        };
                        bindings_layout_info.push(format!(
                            "
							wgpu::BindGroupLayoutEntry {{
								binding: {},
								visibility: {},
								ty: wgpu::BindingType::Sampler({}),
								count: None,
							}}
						",
                            bind,
                            visibility_str.join(" | "),
                            filtering
                        ));
                        bindings_info.push(format!(
                            "
							wgpu::BindGroupEntry {{
								binding: {},
								resource: wgpu::BindingResource::Sampler({}),
							}}
						",
                            bind, name
                        ));
                        group_params.push(format!("{}: &wgpu::Sampler", name));
                    }
                    _ => continue,
                }
            }
			i += 1;
        }

        let name = name1.to_snake_case();
		let class_name = name.to_class_case();

        // groups
        groups.push(format!(
            "
			pub struct {}Group;
			impl pi_render::rhi::dyn_uniform_buffer::Group for {}Group {{
				fn id() -> u32 {{
					{}
				}}

				fn create_layout(device: &pi_render::rhi::device::RenderDevice, has_dynamic_offset: bool) -> pi_render::rhi::bind_group_layout::BindGroupLayout {{
					device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {{
						label: Some(\"{} bindgroup layout\"),
						entries: &[
							{}
						],
					}})
				}}
			}}
		",
			class_name,
			class_name,
            *group,

            name,
            bindings_layout_info.join(",")
        ));

		if is_all_buffer {
			groups.push(format!(
				"
				impl pi_render::rhi::dyn_uniform_buffer::BufferGroup for {}Group {{
					fn create_bind_group(device: &pi_render::rhi::device::RenderDevice, layout: &pi_render::rhi::bind_group_layout::BindGroupLayout, buffer: &pi_render::rhi::buffer::Buffer) -> pi_render::rhi::bind_group::BindGroup {{
						device.create_bind_group(&wgpu::BindGroupDescriptor {{
							layout,
							entries: &[
								{}
							],
							label: Some(\"{} bindgroup\"),
						}})
					}}
				}}
			",
				class_name,
				bindings_info.join(","),
				name,
			));
		} else {
			functions.push(format!("
				pub fn create_bind_group_{}( device: &pi_render::rhi::device::RenderDevice, layout: &pi_render::rhi::bind_group_layout::BindGroupLayout, {}) -> pi_render::rhi::bind_group::BindGroup {{
					device.create_bind_group(&wgpu::BindGroupDescriptor {{
						layout,
						entries: &[
							{}
						],
						label: Some(\"{} bindgroup\"),
					}})
				}}
			", name, group_params.join(","), bindings_info.join(","), name));
		}
    }

    let share_name = shader_name.to_class_case();
    let out_put = format!(
        "
		{}
		{}
		{}
		{}
		pub struct {}Shader;

		impl {}Shader {{
			{}
		}}
	",
		input_strs.join("\n"),
		groups.join("\n"),
		buffer_bindings.join("\n"),
		buffer_uniforms.join("\n"),
        share_name,
        share_name,
        functions.join("\n")
    );

    println!("out_put: {:?}", out_put);

    return Ok(out_put);
}

pub fn get_type(kind: naga::ScalarKind) -> String {
    match kind {
        naga::ScalarKind::Sint => "i32".to_string(),
        naga::ScalarKind::Uint => "u32".to_string(),
        naga::ScalarKind::Float => "f32".to_string(),
        naga::ScalarKind::Bool => "bool".to_string(),
    }
}

pub fn parse_input(vs_module: &naga::Module, input_strs: &mut Vec<String>) {
	for entry in vs_module.entry_points.iter() {
		if entry.stage == ShaderStage::Vertex {
			for arg in entry.function.arguments.iter() {
				if let (Some(name), Some(binding)) = (&arg.name, &arg.binding) {
					if let Binding::Location {location, interpolation, sampling} =  binding {
						let name_class = name.to_class_case();
						input_strs.push(format!("
						pub struct {}VertexBuffer;
						impl {}VertexBuffer {{
							pub fn id() -> u32 {{
								{}
							}}
						}}
						", name_class, name_class, location));
					}
					
				}
			}
		}
	}
	// array_stride: 8 as wgpu::BufferAddress,
	// 	step_mode: wgpu::VertexStepMode::Vertex,
	// 	attributes: vec![
	// 		wgpu::VertexAttribute {
	// 			format: wgpu::VertexFormat::Float32x2,
	// 			offset: 0,
	// 			shader_location: 0,
	// 		},
	// 	],

	// [EntryPoint {
	// 	name: "main",
	// 	stage: Vertex,
	// 	early_depth_test: None,
	// 	workgroup_size: [0, 0, 0],
	// 	function: Function {
	// 		name: None,
	// 		arguments: [FunctionArgument {
	// 			name: Some("position"),
	// 			ty: [1],
	// 			binding: Some(Location {
	// 				location: 0,
	// 				interpolation: Some(Perspective),
	// 				sampling: None
	// 			})
	// 		}],
}

struct LayoutInfo {
    name: String,
    alignment: u32,
    size: usize,
    width: usize,
    kind: naga::ScalarKind,
}
// 	Buffer{
// 		name: String,
// 		alignment: u32,
// 		size: usize,
// 		width: usize,
// 		kind: naga::ScalarKind,
// 	},
// 	Sampler(comparison),
// 	Texture {
// 		dim: naga::ImageDimension,
// 		arrayed: bool,
// 		class: naga::Sampled
// 	},
// }

impl Default for LayoutInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            alignment: Default::default(),
            size: Default::default(),
            width: Default::default(),
            kind: ScalarKind::Float,
        }
    }
}

fn compare_type(
    ty1: &naga::Type,
    ty2: &naga::Type,
    types1: &naga::UniqueArena<naga::Type>,
    types2: &naga::UniqueArena<naga::Type>,
) -> bool {
    if &ty1.name != &ty2.name {
        return false;
    }

    match (&ty1.inner, &ty2.inner) {
        (
            naga::TypeInner::Struct {
                members: members1,
                span: span1,
            },
            naga::TypeInner::Struct {
                members: members2,
                span: span2,
            },
        ) => {
            if span1 != span2 || members1.len() != members2.len() {
                return false;
            }
            for i in 0..members1.len() {
                let (m1, m2) = (&members1[i], &members2[i]);
                if m1.offset != m2.offset || &m1.name != &m2.name {
                    return false;
                }

                if !compare_type(&types1[m1.ty], &types2[m2.ty], types1, types1) {
                    return false;
                }
            }
            return true;
        }
        _ => &ty1.inner == &ty2.inner,
    }
}

fn get_layout_info<'a: 'b, 'b>(
    ty: &'a naga::Type,
    infos: &mut Vec<LayoutInfo>,
    other_info: &'b mut Vec<&'a naga::Type>,
    cur_info: &mut LayoutInfo,
    types: &'a naga::UniqueArena<naga::Type>,
) {
    match &ty.inner {
        naga::TypeInner::Scalar { width, kind } => {
            cur_info.width = *width as usize;
            cur_info.size = 1;
            cur_info.kind = kind.clone();
        }
        naga::TypeInner::Vector { size, kind, width } => {
            cur_info.width = *width as usize;
            cur_info.size = unsafe { transmute::<_, u8>(*size) } as usize;
            cur_info.kind = kind.clone();
        }
        naga::TypeInner::Matrix {
            columns,
            rows,
            width,
        } => {
            cur_info.width = *width as usize;
            cur_info.size =
                unsafe { transmute::<_, u8>(*columns) * transmute::<_, u8>(*rows) } as usize;
            cur_info.kind = ScalarKind::Float;
        }
        naga::TypeInner::Atomic { kind, width } => {
            cur_info.size = 1;
            cur_info.width = *width as usize;
            cur_info.kind = kind.clone();
        }
        naga::TypeInner::Struct { members, span } => {
            cur_info.size = *span as usize;
            for item in members.iter() {
                let mut layout_info = LayoutInfo::default();
                get_layout_info(&types[item.ty], infos, other_info, &mut layout_info, types);
                if let Some(r) = &item.name {
                    layout_info.alignment = item.offset;
                    layout_info.name = r.clone();
                    infos.push(layout_info);
                }
            }
        }
        naga::TypeInner::Image { .. } | naga::TypeInner::Sampler { .. } => {
            other_info.push(ty);
        }
        // naga::TypeInner::Array { base, size, stride } => todo!(),
        _ => (),
    };
}

fn merge_uniforms<'a: 'b, 'b>(
    vs_uniforms: &'b HashMap<(u32, u32), (&'a naga::Type, &'a Option<String>)>,
    fs_uniforms: &'b HashMap<(u32, u32), (&'a naga::Type, &'a Option<String>)>,
    types1: &naga::UniqueArena<naga::Type>,
    types2: &naga::UniqueArena<naga::Type>,
) -> OrdMap<Tree<u32, OrdMap<Tree<(u32, usize), (&'b naga::Type, &'a Option<String>, UniformFrom)>>>> {
    let mut r = HashMap::new();
    for (k, item) in vs_uniforms.iter() {
        match fs_uniforms.get(k) {
            Some(ty) => {
                if !compare_type(item.0, ty.0, types1, types2) {
                    panic!("The same binding exists in VS and FS, but the values are different, set: {}, bind: {}, vs uniform: {:?}, fs uniform: {:?}", k.0, k.1, item.0.name, ty.0.name);
                } else {
                    r.insert(
                        (
                            k.0,
                            k.1,
                            (UniformFrom::Vert as usize) | (UniformFrom::Frag as usize),
                        ),
                        (item.0, item.1, UniformFrom::Vert),
                    );
                }
            }
            None => {
                r.insert(
                    (k.0, k.1, UniformFrom::Vert as usize),
                    (item.0, item.1, UniformFrom::Vert),
                );
            }
        };
    }

    for (k, item) in fs_uniforms.iter() {
        if let None = vs_uniforms.get(k) {
            r.insert(
                (k.0, k.1, (UniformFrom::Frag as usize)),
                (item.0, item.1, UniformFrom::Frag),
            );
        }
    }

    let mut ret = OrdMap::new(Tree::new());

    for ((group, bind, share), item) in r.iter() {
        match ret.get(group) {
            None => {
                let mut map = OrdMap::new(Tree::new());
                map.insert((*bind, *share), (item.0.clone(), item.1, item.2.clone()));
                ret.insert(*group, map);
            }
            Some(_r) => {
				let mut r = ret.delete(group, true).unwrap().unwrap();
                r.insert((*bind, *share), (item.0.clone(), item.1, item.2.clone()));
				ret.insert(*group, r);
            }
        };
    }
    ret
}

#[derive(Clone, Copy)]
enum UniformFrom {
    Vert = 1,
    Frag = 2,
}

fn get_uniforms(module: &naga::Module) -> HashMap<(u32, u32), (&naga::Type, &Option<String>)> {
    let mut uniforms = HashMap::new();
    for item in module.global_variables.iter() {
        if item.1.space == AddressSpace::Uniform || item.1.space == AddressSpace::Handle {
            let ty = &module.types[item.1.ty];
            let (group, binding) = match &item.1.binding {
                Some(r) => (r.group, r.binding),
                None => panic!("Uniform does not set binding: {:?}", ty.name),
            };
            match uniforms.entry((group, binding)) {
                Entry::Vacant(r) => {
                    r.insert((ty, &item.1.name));
                }
                Entry::Occupied(_r) => panic!("Uniform setting binding repeated: {:?}", ty.name),
            };
        }
    }
    uniforms
}

#[derive(Error, Debug)]
pub enum ShaderReflectError {
    #[error(transparent)]
    WgslParse(#[from] naga::front::wgsl::ParseError),

    #[error("GLSL Parse Error: {0:?}")]
    GlslParse(Vec<naga::front::glsl::Error>),

    #[error(transparent)]
    SpirVParse(#[from] naga::front::spv::Error),

    #[error(transparent)]
    Validation(#[from] naga::WithSpan<naga::valid::ValidationError>),
}

#[test]
fn test() {
    use std::process::Command;

	let vs_code = include_str!("../source/text.vert");
    let fs_code = include_str!("../source/text.frag");
	compile_and_out(
		"text", 
		ProcessedShader::Glsl(Cow::Borrowed(vs_code), naga::ShaderStage::Vertex),
		ProcessedShader::Glsl(Cow::Borrowed(fs_code), naga::ShaderStage::Fragment), 
		Path::new("out/"));

   

    // let root_dir = std::env::current_dir().unwrap();
    // println!("root_dir: {:?}", root_dir);

    // let r = compile(
    //     "Color",
    //     ProcessedShader::Glsl(Cow::Borrowed(vs_code), naga::ShaderStage::Vertex),
    //     ProcessedShader::Glsl(Cow::Borrowed(fs_code), naga::ShaderStage::Fragment),
    // );

	// let file_name = "color.rs";
    // match r {
    //     Ok(r) => std::fs::write(root_dir.join("output").join(file_name), r),
    //     Err(r) => panic!("{:?}", r),
    // };

    // Command::new("rustfmt")
    //     .arg(format!("{}", file_name))
	// 	.arg("--config")
	// 	.arg("hard_tabs=true")
    //     .current_dir(root_dir.join("output"))
    //     .status()
    //     .unwrap();
}

#[test]
fn testx() {
	let name = "ColorMatr";
	println!("{}, {}", name.to_snake_case(), name.to_snake_case().to_class_case());
}
