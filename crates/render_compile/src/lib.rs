mod file;

use std::{
    collections::hash_map::Entry,
    path::{Path, PathBuf}, fs::{DirEntry, read},
	str::FromStr, borrow::Cow, mem::replace, cmp::Ordering, sync::Arc,
};

use inflector::{Inflector};
use once_cell::sync::Lazy;
use pi_naga::{ScalarKind, ShaderStage, Binding, AddressSpace, ImageDimension, ImageClass, ArraySize, Interpolation, Sampling, Module, Type, Constant, UniqueArena, TypeInner, ConstantInner, ScalarValue, Arena, front};
use pi_hash::{XHashMap, XHashSet};
use thiserror::Error;
use render_core::rhi::shader::{ShaderImport, CodeLoader, ProcessShaderError};
use regex::Regex;
use pi_atom::Atom;

pub type ShaderId = Atom;

pub enum ShaderPath {
    Wgsl(String),
    Glsl{
		vs: String,
		fs: String
	},
    SpirV(String),
}

/// 解析器
/// # example
/// ```
/// // 创建解析器
/// let mut parser = Parser::default();
/// std::fs::write("build_error.text", "zzzzzzz").unwrap();

/// let r = parser
///     .push_gen_path(&["src/shaders/"]) // 设置shader所在目录
///     .push_program(vec![ProgramDesc::new("src/shaders/test.vert", "src/shaders/test.frag", "src/shaders/aa")]) // 添加额外的program
///     .parse(); // 解析
/// 
/// match r {
///     Ok(r) => {
/// 		// 将解析结果输出到文件
///         for shader in r.shader_result.iter() {
///             std::fs::write(&shader.0, &shader.1).unwrap();
///         }
///			// 输出的rs文件设置在mod.rs中（因此要求，输出的rs文件不要和手动编辑的rs文件放在同一个目录下）
///         let mods = r.to_mod();
///         for (dir, mods) in mods.iter() {
///             std::fs::write(
///                 Path::new(dir).join("mod.rs"),
///                 mods.iter()
///                     .map(|r| "pub mod ".to_string() + r.as_str() + ";")
///                     .collect::<Vec<String>>()
///                     .join("\n"),
///             )
///             .unwrap()
///         }
///     }
///     Err(e) => {
///         panic!("e============={:?}", e);
///     }
/// }
/// ```
#[derive(Default)]
pub struct Parser {
	// 需要生成rs文件的着色器目录或文件
	gen_paths: Vec<String>,
	// 顶点、像素着色器组，每组为一个shaderprogram
	vert_frag_groups: Vec<ProgramDesc>,
}

pub struct ProgramDesc {
	vs: Atom,
	fs: Atom,
	out_put: String,
}

impl ProgramDesc {
	/// 创建ProgramDesc
	/// * vs - vs代码路径， 例如： xxx/yyy.vert
	/// * fs - fs代码路径， 例如： xxx/bbb.frag
	/// * out_put - 输出路径， 例如： src/zzz/xxx.rs, 或src/zzz/xxx， 为指定".rs"后缀， 会自动补齐
	pub fn new(vs: &str, fs: &str, out_put: &str) -> Self {
		Self {
			vs: Atom::from(vs),
			fs: Atom::from(fs),
			out_put: if out_put.ends_with(".rs"){ out_put[0..out_put.len() - 3].to_string() } else{ out_put.to_string()},
		}
	}
}

impl Parser {
	/// push多个生成路径
	/// Parser.parse方法会扫描这些路径下的所有后缀为".vert", ".frag", ".glsl"的文件
	/// ".glsl"文件只能定义普通函数（非入口函数）、binding两种类型的数据。它被构建后，会在同目录下生成同名的“.rs文件”
	/// 同目录下，同名的".vert"和".frag"会为当成一个Program构建，在该目录生成一个同名的“.rs”文件（a.vert + a.frag生成a.rs）
	/// 注意， 只有".glsl"能够被其他文件通过use指令导入
	pub fn push_gen_path(&mut self, slice: &[&str]) -> &mut Self {
		for s in slice.iter() {
			self.gen_paths.push(s.to_string());
		}
		self
	}

	/// push多个program
	/// 如果希望将gen_path路径中的其他非同名的“.vert”和“.frag”文件组合为一个program，则可以通过该接口指定
	pub fn push_program(&mut self, slice: Vec<ProgramDesc>) -> &mut Self {
		for s in slice.into_iter() {
			self.vert_frag_groups.push(s);
		}
		self
	}

	/// 解析shader
	pub fn parse(&mut self) -> Result<PasreResult, CompileShaderError> {
		let mut built_temp = BuildTemp::default();
		// 加载所有需要使用的shader，存储在shader_mar中
		let mut visited_path = XHashMap::default();
		let mut cb = |shader_path: &DirEntry| -> Option<String> {
			// 去重
			match visited_path.get(&shader_path.path()) {
				Some(_r) => return None,
				None => visited_path.insert(shader_path.path(), true),
			};

			let (path, stage) = if let Some(r)  = shader_path.path().extension() {
				match &*r.to_string_lossy() {
					"vert" => (shader_path.path().to_string_lossy().to_string(), ShaderStage::Vertex),
					"frag" => (shader_path.path().to_string_lossy().to_string(), ShaderStage::Fragment),
					"glsl" => (shader_path.path().to_string_lossy().to_string(), ShaderStage::Compute),
					_ => return None,
				}
			} else {
				return None;
			};
            let file = read(shader_path.path());
            if let Ok(bytes) = file {
				let shader_code = String::from_utf8(bytes.to_vec()).unwrap();
				println!("path=================={:?}", shader_path);
				let _ = self.parse_shader_slice_code(shader_code, &shader_path.path(), stage,  &mut built_temp);
            }
			Some(path)
        };

		let mut gen_paths = XHashSet::default();
		for path in self.gen_paths.iter() {
			file::visit_dirs(path, &mut |shader_path: &DirEntry| {
				if let Some(_) = cb(shader_path) {
					gen_paths.insert(Atom::from(shader_path.path().to_string_lossy().to_string()));
				}
			}).unwrap();
		}

		// 
		for i in self.vert_frag_groups.iter() {
			// CompileShaderError
			let vs = match built_temp.built_slice.get(&i.vs) {
				Some(r) => r.clone(),
				None => return Err(CompileShaderError::ShaderNotExist(i.vs.clone())),
			};
			let fs = match built_temp.built_slice.get(&i.fs) {
				Some(r) => r.clone(),
				None => return Err(CompileShaderError::ShaderNotExist(i.fs.clone())),
			};
			let vs_path = Atom::from(i.out_put.clone() + ".vert");
			let fs_path = Atom::from(i.out_put.clone() + ".frag");

			built_temp.built_slice.insert(vs_path.clone(), vs);
			built_temp.built_slice.insert(fs_path.clone(), fs);
			built_temp.built_map.insert(vs_path.clone(), built_temp.built_map.get(&i.vs).unwrap().clone());
			built_temp.built_map.insert(fs_path.clone(), built_temp.built_map.get(&i.fs).unwrap().clone());
			
			gen_paths.insert(vs_path);
			gen_paths.insert(fs_path);
		}

		self.parse1(&gen_paths, &mut built_temp)
		
	}

	fn parse1(&mut self, gen_paths: &XHashSet<Atom>, built_temp: &mut BuildTemp) -> Result<PasreResult, CompileShaderError> {
		// 内建变量字符串
		// let mut build_var = ShaderVarIndice::default();
		let mut result = PasreResult::default();
		// build_var.binding.insert(UniformSlot { group: ShaderVarTypes::SET, binding: ShaderVarTypes::BINDIND }, (Atom::from(""), BindingType::Sampler(false, ArrayLen::None), "".to_string()));

		// 按program构建
		for i in gen_paths.iter() {
			if i.ends_with(".vert") {
				let frag = Atom::from(i[0..i.len() - 5].to_string() + ".frag");
				if gen_paths.contains(&frag) {
					self.build_program(

						i, 
						&frag, 
						built_temp,
						&mut result,
					)?;
					continue;
				}
			}
			// println!("pppp============={:?}", &Path::new(i).to_path_buf());
			// let shader_id = built_temp.path_map_id.get(&Path::new(i).to_path_buf()).unwrap().clone();
			self.build_shader(
				i.clone(),
				built_temp,
				&mut result,
				&mut XHashSet::default(),
			)?;
		}
		Ok(result)
	}

	fn build_program(
		&mut self,
		vert_path: &Atom,
		frag_path: &Atom,
		built_temp: &mut BuildTemp,
		result: &mut PasreResult,
	) -> Result<(), CompileShaderError>  {
		let vert_path_str = vert_path;
		// let name = Path::new(frag_path.as_str()).file_name().unwrap().to_str().unwrap();
		// let name = &name[0..name.len() - 5];
		// let class_name = to_class_case(name);

		let vs_shader_id = vert_path.clone();
		let fs_shader_id = frag_path.clone();
		self.build_shader(
			vs_shader_id.clone(),
			built_temp,
			result,
			&mut XHashSet::default(),
		)?;
		self.build_shader(
			fs_shader_id.clone(),
			built_temp,
			result,
			&mut XHashSet::default(),
		)?;

		let vs_module = built_temp.built_map.get(&vs_shader_id).unwrap();
		let fs_module = built_temp.built_map.get(&fs_shader_id).unwrap();
		let vs_bindings_list = built_temp.bindings.get(&vs_shader_id).unwrap();
		let fs_bindings_list = built_temp.bindings.get(&fs_shader_id).unwrap();
		let vs_bindings = vs_bindings_list
			.iter()
			.filter(|r| {fs_bindings_list.get(r.0).is_none()})
			.map(|r| {(r.0.clone(), r.1.clone())})
			.collect::<XHashMap<UniformSlot, (BindingType, String)>>();
		let share_bindings = vs_bindings_list
			.iter()
			.filter(|r| {fs_bindings_list.get(r.0).is_some()})
			.map(|r| {(r.0.clone(), r.1.clone())})
			.collect::<XHashMap<UniformSlot, (BindingType, String)>>();
		let fs_bindings = fs_bindings_list
			.iter()
			.filter(|r| {vs_bindings_list.get(r.0).is_none()})
			.map(|r| {(r.0.clone(), r.1.clone())})
			.collect::<XHashMap<UniformSlot, (BindingType, String)>>();
		let inputs = built_temp.inputs.get(&vs_shader_id).unwrap();
		
		let vs_slice = built_temp.built_slice.get(&vs_shader_id).unwrap();
		let fs_slice = built_temp.built_slice.get(&fs_shader_id).unwrap();

		let vs_compile_result = compile_uniform(
			vs_module, 
			&vs_bindings,
			&inputs,
			vs_slice,
		)?;
		let share_compile_result = compile_uniform(
			vs_module, 
			&share_bindings,
			&inputs,
			vs_slice,
		)?;
		let fs_compile_result = compile_uniform(
			fs_module, 
			&fs_bindings,
			&Vec::default(),
			fs_slice,
		)?;
		// format!("Define::new({}, {}[{}])", r.0, define_name, r.1 ) 
		// push_meta.push(format!("{}::push_meta(meta, visibility, defines.clone().extend_from_slice(&[{}])));", path, defines));
		let mut vs_imports = XHashSet::default();
		vs_imports.extend(vs_slice.imports.clone().into_iter());
		let mut fs_imports = XHashSet::default();
		fs_imports.extend(fs_slice.imports.clone().into_iter());

		let vs_import = vs_imports
			.iter()
			.filter(|r| {!fs_imports.contains(*r)})
			.map(|r| {
				let defines = r.1.iter().map(|r| {format!("Define::new({}, VS_DEFINE[{}].clone())", r.0, r.1 )}).collect::<Vec<String>>().join(",");
				if let ShaderImport::Path(path) = &r.0 {
					format!("{}::push_meta(meta, visibility, &[{}]);", path, defines)
				} else {
					"".to_string()
				}
			})
			.collect::<Vec<String>>();
		let fs_import = fs_imports
			.iter()
			.filter(|r| {!vs_imports.contains(*r)})
			.map(|r| {
				let defines = r.1.iter().map(|r| {format!("Define::new({}, FS_DEFINE[{}].clone())", r.0, r.1 )}).collect::<Vec<String>>().join(",");
				if let ShaderImport::Path(path) = &r.0 {
					format!("{}::push_meta(meta, visibility, &[{}]);", path, defines)
				} else {
					"".to_string()
				}
			})
			.collect::<Vec<String>>();
		let share_import = vs_imports
			.iter()
			.filter(|r| {fs_imports.contains(*r)})
			.map(|r| {
				let defines = r.1.iter().map(|r| {format!("Define::new({}, VS_DEFINE[{}].clone())", r.0, r.1 )}).collect::<Vec<String>>().join(",");
				if let ShaderImport::Path(path) = &r.0 {
					format!("{}::push_meta(meta, visibility, &[{}]);", path, defines)
				} else {
					"".to_string()
				}
			})
			.collect::<Vec<String>>();

		let vs_visibility = if vs_compile_result.entrys.len() > 0 || vs_import.len() > 0 {
			"let visibility = wgpu::ShaderStages::VERTEX;"
		} else {
			""
		};
		let fs_visibility = if fs_compile_result.entrys.len() > 0  || fs_import.len() > 0{
			"let visibility = wgpu::ShaderStages::FRAGMENT;\n"
		} else {
			""
		};
		let share_visibility = if share_compile_result.entrys.len() > 0 || share_import.len() > 0 {
			"let visibility = wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT;\n"
		} else {
			""
		};

		let mut varing_list = Vec::new();
		let mut in_list = Vec::new();
		let mut out_list = Vec::new();

		for InOut {name,ty, location, defines } in vs_slice.out_list.iter() {
			varing_list.push(format!(r#"
				InOut::new("{}", "{}", {}, vec![{}])
			"#,
				name, ty, location, defines.iter().map(|r| {format!(r#"Define::new({}, VS_DEFINE[{}].clone()))"#, r.0, r.1)}).collect::<Vec<String>>().join(",")
			));
		}

		for InOut {name,ty, location, defines } in vs_slice.in_list.iter() {
			in_list.push(format!(r#"
				InOut::new("{}", "{}", {}, vec![{}])
			"#,
				name, ty, location, defines.iter().map(|r| {format!(r#"Define::new({}, VS_DEFINE[{}].clone())"#, r.0, r.1)}).collect::<Vec<String>>().join(",")
			));
		}

		for InOut { name,ty, location, defines} in fs_slice.out_list.iter() {
			out_list.push(format!(r#"
				InOut::new("{}", "{}", {}, vec![{}])
			"#,
				name, ty, location,  defines.iter().map(|r| {format!(r#"Define::new({}, FS_DEFINE[{}].clone())"#, r.0, r.1)}).collect::<Vec<String>>().join(",")
			));
		}
		
		let out_str = format!(r#"
			use pi_render::rhi::shader::{{BindingExpandDesc, TypeKind, TypeSize, ArrayLen, ShaderMeta, CodeSlice, BlockCodeAtom, InOut, AsLayoutEntry, Define, merge_defines, BindingExpandDescList,  ShaderVarying, ShaderInput, ShaderOutput}};
			use render_derive::{{BindLayout, BufferSize, Uniform, BindingType}};
			use pi_atom::Atom;
			use pi_map::vecmap::VecMap;
			{}
			{}
			{}

			pub struct ProgramMeta;
			impl ProgramMeta {{
				pub fn create_meta() -> pi_render::rhi::shader::ShaderMeta {{
					let mut meta_ = ShaderMeta::default();
					let defines: &[Define] = &[];
					let meta = &mut meta_;
					{}
					{}
					{}
					{}
					{}
					{}
					{}
					{}
					{}
					push_vs_code(&mut meta.vs);
					push_fs_code(&mut meta.fs);
					meta.varyings = ShaderVarying(vec![
						{}
					]);
					meta.ins = ShaderInput(vec![
						{}
					]);
					meta.outs = ShaderOutput(vec![
						{}
					]);
					meta_
				}}
			}}
			fn push_vs_code(codes: &mut BlockCodeAtom) {{
				let defines: &[Define] = &[];
				{}
			}}

			fn push_fs_code(codes: &mut BlockCodeAtom) {{
				let defines: &[Define] = &[];
				{}
			}}

			lazy_static! {{
				static ref VS_CODE: Vec<CodeSlice> = vec![{}];
				static ref FS_CODE: Vec<CodeSlice> = vec![{}];
				static ref VS_DEFINE: Vec<Atom> = vec![{}];
				static ref FS_DEFINE: Vec<Atom> = vec![{}];
			}}
		"#, 
			vs_compile_result.out_code,
			fs_compile_result.out_code,
			share_compile_result.out_code,

			vs_visibility,
			vs_compile_result.entrys.join("\n"),
			vs_import.join("\n"),
			fs_visibility,
			fs_compile_result.entrys.join("\n"),
			fs_import.join("\n"),
			share_visibility,
			share_compile_result.entrys.join("\n"),
			share_import.join("\n"),
			
			varing_list.join(","), in_list.join(","), out_list.join(","),
			vs_compile_result.push_code.join("\n"),
			fs_compile_result.push_code.join("\n"),
			vs_compile_result.codes.join(",\n"),
			fs_compile_result.codes.join(",\n"),
			vs_slice.defines.iter().map(|r| {format!(r#"Atom::from("{}")"#, r)}).collect::<Vec<String>>().join(","),
			fs_slice.defines.iter().map(|r| {format!(r#"Atom::from("{}")"#, r)}).collect::<Vec<String>>().join(",")
		);
		result.shader_result.push((
			vert_path_str[0..vert_path_str.len() - 5].to_string() + ".rs", 
			out_str
		));
		Ok(())
	}

	fn build_shader(
		&mut self,
		shader_id: ShaderId,
		built_temp: &mut BuildTemp,
		result: &mut PasreResult,
		import_set: &mut XHashSet<ShaderId>, // 用于检查是否循环引用
	) -> Result<(), CompileShaderError> {
		// 已经构建，则忽略
		if built_temp.built_set.contains(&shader_id) {
			return Ok(());
		}
		// 检查是否循环引用
		if import_set.contains(&shader_id) {
			return Ok(Default::default()); // 返回循环引用错误， TODO
		} else {
			import_set.insert(shader_id.clone());
		}
		
		let slice = built_temp.built_slice.get(&shader_id).unwrap();
		let module = built_temp.built_map.get(&shader_id).unwrap();
		let bindings = get_bindings(module, &XHashMap::default());
		let inputs = get_inputs(module, &XHashMap::default());
		let shader_slice = built_temp.built_slice.get(&shader_id).unwrap();
		let extension = slice.path.extension().unwrap();

		println!("p=================={:?}, {:?}", extension, slice.path);
		if extension != "vert" && extension != "frag" {
			let uniform_code = compile_uniform(
				module, 
				&bindings,
				&inputs,
				shader_slice,
			)?;
			let compile_result =format!(r#"
				use pi_render::rhi::shader::{{ ShaderMeta, CodeSlice, BlockCodeAtom, BindingExpandDesc, TypeKind, ArrayLen, TypeSize, AsLayoutEntry, Define, merge_defines, BindingExpandDescList}};
				use render_derive::{{BindLayout, BufferSize, Uniform, BindingType}};
				use pi_atom::Atom;
				{}
				pub fn push_meta(meta: &mut ShaderMeta, visibility: wgpu::ShaderStages, defines: &[Define]) {{
					{}
					{}
				}}

				pub fn push_code(codes: &mut BlockCodeAtom, defines: &[Define]) {{
					{}
				}}

				lazy_static! {{
					static ref CODE: Vec<CodeSlice> = vec![{}];
					static ref DEFINE: Vec<Atom> = vec![{}];
				}}
			"#, 
				uniform_code.out_code,
				uniform_code.entrys.join("\n"),
				uniform_code.push_meta.join("\n"),
				uniform_code.push_code.join("\n"),
				uniform_code.codes.join(",\n"),
				slice.defines.iter().map(|r| {format!(r#"Atom::from("{}")"#, r)}).collect::<Vec<String>>().join(",")
			);
			println!("p=================={:?}, {:?}, {:?}", extension, slice.path, compile_result);
			let path = slice.path.to_string_lossy().to_string();
			result.shader_result.push((path[0..path.len() - 5].to_string() + ".rs", compile_result));
		}

		built_temp.bindings.insert(shader_id.clone(), bindings.clone());
		built_temp.inputs.insert(shader_id.clone(), inputs.clone());

		// bindings.into_iter().map(|r| {(r.0, (shader_id.clone(), r.1.0, r.1.1))});

		// exclude.binding.extend(bindings.into_iter().map(|r| {(r.0, (shader_id, r.1.0, r.1.1))}));
		// exclude.input.extend(inputs.into_iter().map(|r| {(r.location, (shader_id, r.name))}));

		// 设置构建状态为已构建
		built_temp.built_set.insert(shader_id);
		// 缓存exclude
		// built_temp.built_vars.insert(shader_id, exclude.clone());

		Ok(())
	}


	// 提取功能代码片段
	fn parse_shader_slice_code(&self, shader_str: impl Into<Cow<'static, str>>, path: &PathBuf, shader_stage: ShaderStage, build_temp: &mut BuildTemp) -> Result<(), CompileShaderError> {
		let id = Atom::from(path.to_string_lossy().to_string());
		let shader_str = shader_str.into();
		
		let mut run_code_list = Vec::new();

        let mut uniform_scopes = vec![];
		let mut runscopes = vec![];
		let mut scopes = vec![true];
		let mut defines = Vec::new();

		let mut default_value = "".to_string();
        let mut run_string = String::new();
		let mut in_list = Vec::default();
		let mut out_list = Vec::default();
		let mut cmd_list = Vec::default();
		let mut imports = Vec::default();

		let mut last_code = String::new();

		let mut default_map = XHashMap::default();
		let mut binding_defines:  XHashMap<UniformSlot, Vec<(bool, usize)>> = XHashMap::default();
		let mut define_list: Vec<String> = Vec::new();
		let mut define_set: XHashSet<String> = XHashSet::default();


		// let mut slice_map = XHashMap::default();
		let push_code = |run_code_list: &mut Vec<Code>, run_string: &mut String, cmd_list: &mut Vec<String>, defines: &Vec<(bool, usize)>| {
			let run_string = replace(run_string, "".to_string());
			run_code_list.push(Code {content: CodeItem::Code {run_code: run_string, other_code: cmd_list.join("\n") }, defines: defines.clone()});
			cmd_list.clear();
		};

		for line in shader_str.lines() {
			if SHADER_PROCESSOR.note_regex.captures(line).is_some(){
				// ignore note
			}else if let Some(cap) = SHADER_PROCESSOR.ifdef_regex.captures(line) {
				// 遇到 #ifdef
				push_code(&mut run_code_list, &mut run_string, &mut cmd_list, &defines);
                // 取对应的 def
                let def = cap.get(1).unwrap().as_str().to_string();
                // 将 shader_defs 是否 含 该 def 的结果 加到 scopes 中
				if !define_set.contains(&def) {
					define_set.insert(def.clone());
					define_list.push(def.clone());
				}
                scopes.push(*scopes.last().unwrap());
				defines.push((true, define_list.len() - 1));
			} else if let Some(cap) = SHADER_PROCESSOR.ifndef_regex.captures(line) {
				push_code(&mut run_code_list, &mut run_string, &mut cmd_list, &defines);
                // #ifndef 就将结果 取反，然后加到 scopes中
                let def = cap.get(1).unwrap().as_str().to_string();
				if !define_set.contains(&def) {
					define_set.insert(def.clone());
					define_list.push(def.clone());
				}
				defines.push((false, define_list.len() - 1));
                scopes.push(*scopes.last().unwrap());
            } else if SHADER_PROCESSOR.else_regex.is_match(line) {
				push_code(&mut run_code_list, &mut run_string, &mut cmd_list, &defines);
                // 遇到 #else
                let mut is_parent_scope_truthy = true;
                if scopes.len() > 1 {
                    is_parent_scope_truthy = scopes[scopes.len() - 2];
                }
                if let Some(last) = scopes.last_mut() {
                    *last = is_parent_scope_truthy && !*last;
                }
            } else if SHADER_PROCESSOR.endif_regex.is_match(line) {
				push_code(&mut run_code_list, &mut run_string, &mut cmd_list, &defines);
				// slice_map.insert(defines.clone(), slice);
                // 遇到 #endif，scopes 结束
                scopes.pop();
                if scopes.is_empty() {
                    return Err(CompileShaderError::TypeNotSupport("".to_string())); // TODO
                }
				defines.pop();
            } else if let Some(cap) = SHADER_PROCESSOR.layout_struct_regex.captures(line) {
				println!("!!!!!!!!!!!!!!================={:?}, {:?}", line, path);
				default_value = "".to_string();
				uniform_scopes.push(true);
				last_code.push_str(line);
				last_code.push_str("\n");
				if defines.len() > 0 {
					let group = u32::from_str(cap.get(1).unwrap().as_str().to_string().as_str()).unwrap() ;
					let binding = u32::from_str(cap.get(2).unwrap().as_str().to_string().as_str()).unwrap() ;
					binding_defines.insert(UniformSlot { group, binding }, defines.clone());
				}
			} else if let Some(cap) = SHADER_PROCESSOR.layout_simple_regex.captures(line) {
				let name = cap.get(3).unwrap().as_str().to_string();
				if default_value.as_str() != "" {
					default_map.insert(name, default_value);
				}
				default_value = "".to_string();
				last_code.push_str(line);
				last_code.push_str("\n");
				if defines.len() > 0 {
					let group = u32::from_str(cap.get(1).unwrap().as_str().to_string().as_str()).unwrap() ;
					let binding = u32::from_str(cap.get(2).unwrap().as_str().to_string().as_str()).unwrap() ;
					binding_defines.insert(UniformSlot { group, binding }, defines.clone());
				}
			} else if let Some(cap) = SHADER_PROCESSOR.in_regex.captures(line){
				let location = u32::from_str(cap.get(1).unwrap().as_str()).unwrap();
				let ty = cap.get(2).unwrap().as_str().to_string();
				let name = cap.get(3).unwrap().as_str().to_string();
				in_list.push(InOut{name, ty, location, defines: defines.clone()});
				last_code.push_str(line);
				last_code.push_str("\n");
			} else if let Some(cap) = SHADER_PROCESSOR.out_regex.captures(line){
				let location = u32::from_str(cap.get(1).unwrap().as_str()).unwrap();
				let ty = cap.get(2).unwrap().as_str().to_string();
				let name = cap.get(3).unwrap().as_str().to_string();
				out_list.push(InOut{name, ty, location, defines: defines.clone()});
				last_code.push_str(line);
				last_code.push_str("\n");
			} else if uniform_scopes.len() > 0 {
				println!("lin================={:?}, {:?}", line, path);
				if let Some(cap) = SHADER_PROCESSOR.default_value_regex.captures(line){
					println!("default===================!!!");
					// 默认值
					default_value = cap.get(1).unwrap().as_str().to_string();
				} else {
					if line.find("}").is_some() {
						uniform_scopes.pop();
					}
					if default_value.as_str() != "" {
						if let Some(cap) = SHADER_PROCESSOR.struct_feild.captures(line) {
							let name = cap.get(1).unwrap().as_str().to_string();
							default_map.insert(name, default_value);
							default_value = "".to_string();
						}
					}
					last_code.push_str(line);
					last_code.push_str("\n");
				}
            } else if let Some(cap) = SHADER_PROCESSOR.default_value_regex.captures(line){
				// 默认值
				default_value = cap.get(1).unwrap().as_str().to_string();
			} else if let Some(cap) = SHADER_PROCESSOR.import_asset_path_regex.captures(line){
				let import = cap.get(1).unwrap().as_str().to_string();
				imports.push((ShaderImport::Path(import.clone()), defines.clone()));

				// 根据imports分割代码
				run_code_list.push(Code {content: CodeItem::Import(ShaderImport::Path(import)), defines: defines.clone()});
				run_code_list.push(Code {content: CodeItem::Code {run_code: run_string, other_code: cmd_list.join("\n") }, defines: defines.clone()});

				run_string = "".to_string();
				cmd_list.clear();
			} else if let Some(cap) =  SHADER_PROCESSOR.import_custom_path_regex.captures(line){
				let import = cap.get(1).unwrap().as_str().to_string();
				imports.push((ShaderImport::Path(import.clone()), defines.clone()));

				// 根据imports分割代码
				run_code_list.push(Code {content: CodeItem::Code {run_code: run_string, other_code: cmd_list.join("\n") }, defines: defines.clone()});
				run_code_list.push(Code {content: CodeItem::Import(ShaderImport::Path(import)), defines: defines.clone()});
				run_string = "".to_string();
				cmd_list.clear();
			} else if SHADER_PROCESSOR.define_import_path_regex.captures(line).is_some() {
				// ignore line
			}else if SHADER_PROCESSOR.entry_regex.captures(line).is_some() {
				runscopes.push(true);
				last_code.push_str(line);
				last_code.push_str("\n");
			} else if runscopes.len() > 0 {
				if line.find("}").is_some() {
					runscopes.pop();
				} else if !is_whitespace(line) {
					run_string.push_str(line);
                	run_string.push('\n');
				}
				last_code.push_str(line);
				last_code.push_str("\n");
			} else if !is_whitespace(line){
				cmd_list.push(line.to_string() + "\n");
				last_code.push_str(line);
				last_code.push_str("\n");
			}
        }
		run_code_list.push(Code {content: CodeItem::Code {run_code: run_string, other_code: cmd_list.join("\n") }, defines: defines.clone()});

		let mut parser = front::glsl::Parser::default();
		let module = parser
			.parse(&front::glsl::Options::from(ShaderStage::Vertex), &*last_code)
			.map_err(CompileShaderError::GlslParse1)?;
		build_temp.built_map.insert(id.clone(), Arc::new(module));

		let slice = ShaderSlice { 
			code: run_code_list, 
			in_list, 
			out_list, 
			default_value: default_map, 
			// source: last_code.into(),
			path: path.clone(),
			imports,
			stage: match shader_stage {
				ShaderStage::Vertex => ShaderStageTy::Vert,
				ShaderStage::Fragment => ShaderStageTy::Frag,
				ShaderStage::Compute => ShaderStageTy::Compute,
			},
			binding_defines,
			defines: define_list,
		};
		println!("built_slice!!!==========={:?}, {:?}", path, slice);
		build_temp.built_slice.insert(id.clone(), slice);
		Ok(())
	}

	// fn join_slice_code(&self, shader_id: &ShaderId, code: &mut String, varing: &mut Vec<(String, String, u32)>, default_value: &mut XHashMap<String, String>, temp: &BuildTemp, is_vs: bool) {
	// 	let (shader, _path) = self.shader_mgr.shaders.get(shader_id).unwrap().clone();
	// 	let slice = temp.built_slice.get(shader_id).unwrap();
	// 	code.push_str(slice.code.as_str());
	// 	if is_vs {
	// 		varing.extend_from_slice(&slice.out_list);
	// 	}
	// 	default_value.extend(slice.default_value.clone().into_iter());
	// 	code.push_str("\n");

	// 	for i in shader.imports() {
	// 		let id = self.shader_mgr.import_shaders.get(i).unwrap();
	// 		self.join_slice_code(id, code, varing, default_value, temp, is_vs);
	// 	}
	// }
}

pub fn is_whitespace(s: &str) -> bool{
	for i in s.chars() {
		if i != ' ' && i != '\n' && i != '\t' && i != '\r'{
			return  false;
		}
	}
	true
}

#[derive(Debug, Clone)]
pub struct ShaderSlice {
	code: Vec<Code>,

	in_list: Vec<InOut>,
	out_list: Vec<InOut>,
	default_value: XHashMap<String, String>,
	

	// source: Cow<'static, str>,
	stage: ShaderStageTy,
	path: PathBuf,
	imports: Vec<(ShaderImport,Vec<(bool, usize)>)>,
	binding_defines: XHashMap<UniformSlot, Vec<(bool, usize)>>,

	// 宏
	defines: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Code {
	content: CodeItem,
	defines: Vec<(bool, usize)>
}

#[derive(Debug, Clone)]
pub enum CodeItem {
	Code{run_code: String, other_code: String}, // 代码字符串
	Import(ShaderImport), // 导入路径
}

#[derive(Default, Debug, Clone)]
pub struct InOut {
	name: String,
	ty: String,
	location: u32,
	defines: Vec<(bool, usize)>,
}

#[derive(Debug, Default, Clone)]
pub enum ShaderStageTy {
	#[default]
	Vert,
	Frag,
	Compute
}

pub static SHADER_PROCESSOR: Lazy<ShaderProcessor1> =
    Lazy::new(ShaderProcessor1::default);

// pub static BILD_IN_VAR_PROCESSOR: Lazy<BuildInVarProcessor> =
// Lazy::new(BuildInVarProcessor::default);
	
pub struct ShaderProcessor1 {
	import_asset_path_regex: Regex,
	// #import ...
	import_custom_path_regex: Regex,
	// #define_import_path ...
	define_import_path_regex: Regex,
	layout_simple_regex: Regex,
	layout_struct_regex: Regex,
	default_value_regex: Regex,
	struct_feild: Regex,
	in_regex: Regex,
	out_regex: Regex,
	// other_cmd_regex: Regex,

	note_regex: Regex,

	ifdef_regex: Regex,
	ifndef_regex: Regex,
	else_regex: Regex,
	endif_regex: Regex,
	entry_regex: Regex,
}

impl Default for ShaderProcessor1 {
	fn default() -> Self {
		Self { 
			note_regex: Regex::new(r#"^\s*//"#).unwrap(),
			// layout(set=..,binding=..) uniform .. ..;
			layout_simple_regex: Regex::new(r#"^\s*layout\s*\(\s*set\s*=\s*([0-9]+)\s*,\s*binding\s*=\s*([0-9]+)\s*\)\s*uniform\s*[a-zA-Z0-9]+\s*([a-zA-Z_0-9]+)"#).unwrap(),
			// layout(set=..,binding=..) uniform ..{..};
			layout_struct_regex: Regex::new(r#"^\s*layout\s*\(\s*set\s*=\s*([0-9]+)\s*,\s*binding\s*=\s*([0-9]+)\s*\)\s*uniform\s*[a-zA-Z_0-9\s]*\{"#).unwrap(),
			// layout(location=..) in ..;
			in_regex: Regex::new(r#"^\s*layout\s*\(\s*location\s*=\s*([0-9]+)\s*\)\s*in\s*([a-zA-Z_0-9]+)\s*([a-zA-Z_0-9]+)"#).unwrap(),
			// layout(location=..) out ..;
			out_regex: Regex::new(r#"^\s*layout\s*\(\s*location\s*=\s*([0-9]+)\s*\)\s*out\s*([a-zA-Z_0-9]+)\s*([a-zA-Z_0-9]+)"#).unwrap(),
			// @default(..)
			default_value_regex: Regex::new(r#"^\s*@default\s*\(\s*([0-9.,\s]+)\)"#).unwrap(),
			struct_feild: Regex::new(r#"^\s*[a-zA-Z0-9]+\s*([a-zA-Z_0-9]+)"#).unwrap(),
			// #import "..."
            import_asset_path_regex: Regex::new(r#"^\s*#\s*import\s*"([a-zA-Z_0-9:]+)""#).unwrap(),
            // #import ...
            import_custom_path_regex: Regex::new(r"^\s*#\s*import\s*([a-zA-Z_0-9:]+)").unwrap(),
			// #define_import_path ...
			define_import_path_regex: Regex::new(r"^\s*#\s*define_import_path\s+([a-zA-Z_0-9:]+)").unwrap(),
			// #xx
			// other_cmd_regex: Regex::new(r"^\s*#\s*[a-zA-Z]+\s*(.+)").unwrap(),

			ifdef_regex: Regex::new(r"^\s*#\s*ifdef\s*([\w|\d|_]+)").unwrap(),
            ifndef_regex: Regex::new(r"^\s*#\s*ifndef\s*([\w|\d|_]+)").unwrap(),
            else_regex: Regex::new(r"^\s*#\s*else").unwrap(),
            endif_regex: Regex::new(r"^\s*#\s*endif").unwrap(),
			entry_regex: Regex::new(r#"^\s*void\s*main\s*\(\s*\)\s*(.+)"#).unwrap(),
		 }
	}
}
// layout(set = 0, binding = 0) uniform CameraMatrix {

// #[derive(Debug, Default, Clone)]
// pub struct ShaderVarIndice {
// 	binding: XHashMap<UniformSlot, (ShaderId, BindingType, String)>,
// 	input: XHashMap<u32, (ShaderId, String)>,
// }

// impl ShaderVarIndice {
// 	fn extend(&mut self, value: ShaderVarIndice) {
// 		for i in value.binding.into_iter() {
// 			self.binding.insert(i.0, i.1);
// 		}

// 		for i in value.input.into_iter() {
// 			self.input.insert(i.0, i.1);
// 		}
// 	}
// }


#[derive(Default)]
struct BuildTemp {
	// id_map_path: XHashMap<ShaderId, PathBuf>, // shader_id到路径的映射
	// path_map_id: XHashMap<PathBuf, ShaderId>, // 路径到shader_id的映射
	built_set: XHashSet<Atom/*路径*/>,
	built_map: XHashMap<Atom/*路径*/, Arc<Module>>,
	// built_vars: XHashMap<Atom, ShaderVarIndice>,
	built_slice: XHashMap<Atom, ShaderSlice>,
	bindings: XHashMap<Atom, XHashMap<UniformSlot, (BindingType, String)>>,
	inputs: XHashMap<Atom, Vec<BindingLocation>>,
}

#[derive(Default, Debug)]
pub struct PasreResult {
	pub shader_result: Vec<(String, String)>,
}

impl PasreResult {
	pub fn to_mod(&self) -> XHashMap<String, Vec<String>> {
		let mut s: XHashMap<String, Vec<String>> = XHashMap::default();
		for r in self.shader_result.iter() {
			let p = Path::new(&r.0);
			let filename = p.file_name().unwrap().to_string_lossy();
			let dir = r.0[0..r.0.len() - filename.len()].to_string();
			let filename = &filename[0..filename.len() - 3];
			match s.entry(dir) {
				Entry::Occupied(mut r) => {
					r.get_mut().push(filename.to_string());
				},
				Entry::Vacant(r) => {
					let mut h = Vec::default();
					h.push(filename.to_string());
					r.insert(h);
				},
			}
		}
		s
	}
}

#[derive(Error, Debug)]
pub enum CompileShaderError {
	#[error("shader is not exist: {0:?}")]
    ShaderNotExist(Atom),

	#[error("import key is not exist: {0:?}")]
    ImportNotFind(ShaderImport),

	#[error("var type is not support: {0:?}")]
    TypeNotSupport(String),

	#[error("validation var fail, expect: {0:?}, actual is: {1:?}")]
    ValidationVarFail(String, String),

	#[error("invalid import path: {0:?}")]
	InvalidImportPath(String),

	#[error(transparent)]
    WgslParse(#[from] naga::front::wgsl::ParseError),

	#[error("GLSL Parse Error: {0:?}")]
    GlslParse(Vec<naga::front::glsl::Error>),

    #[error(transparent)]
    SpirVParse(#[from] naga::front::spv::Error),

    #[error(transparent)]
    Validation(#[from] naga::WithSpan<naga::valid::ValidationError>),

	#[error(transparent)]
    WgslParse1(#[from] pi_naga::front::wgsl::ParseError),

	#[error("GLSL Parse Error: {0:?}")]
    GlslParse1(Vec<pi_naga::front::glsl::Error>),

    #[error(transparent)]
    SpirVParse1(#[from] pi_naga::front::spv::Error),

    #[error(transparent)]
    Validation1(#[from] pi_naga::WithSpan<naga::valid::ValidationError>),
}

// pub struct BuildInVarProcessor {
// 	version_regex: Regex,
// 	entry_regex: Regex,
// }

// impl Default for BuildInVarProcessor {
//     fn default() -> Self {
//         Self {
//             // #version ...
//             version_regex: Regex::new(r#"^\s*#\s*version\s*(.+)"#).unwrap(),
// 			entry_regex: Regex::new(r#"^\s*\s*void\s*main\s*\(\s*\)\s*(.+)"#).unwrap(),
//         }
//     }
// }

/// 提取shader中的uniform， 定义和实现对应的rust数据结构
fn compile_uniform(
	module: &Module, 
	bindings: &XHashMap<UniformSlot, (BindingType, String)>,
	inputs: &Vec<BindingLocation>,
	code_slice: &ShaderSlice,
) -> Result<UniformCode, CompileShaderError> {
	println!("module: {:?}", module);

	let mut out_code = Vec::new();
	let mut entrys = Vec::new();
	parse_input(&mut out_code, inputs);

	let (code_name, define_name) = match code_slice.stage {
		ShaderStageTy::Vert => ("VS_CODE", "VS_DEFINE"),
		ShaderStageTy::Frag => ("FS_CODE", "FS_DEFINE"),
		ShaderStageTy::Compute => ("CODE", "DEFINE"),
	}; 

	// 定义binding类型、uniform类型， 
	// 为bingding实现BindIndex tarit，并实现binding_type方法；为buffer类型的bingding实现BufferBind,
	// 为uniform实现Uniform tarit
	for (slot, (binding_type, binding_name)) in bindings.iter(){
		let bind_class_case_name = binding_name.to_class_case();
		let defines = code_slice.binding_defines.get(slot).map_or("".to_string(), |r| {r.iter().map(|r| {format!(r#"Define::new({}, {}[{}].clone())"#, r.0, define_name, r.1)}).collect::<Vec<String>>().join(",")});

		match binding_type {
			BindingType::Buffer(buffer_binding) => {
				let alignment_size = buffer_binding.alignment_size;
				let mut uniform_expand = Vec::new();
				for m in buffer_binding.merbers.iter() {
					let size = match m.ty {
						VarType::Mat => format!("TypeSize::Mat{{rows: {}, columns: {}}}", m.size/(m.span/4), m.span/4),
						VarType::Vector => format!("TypeSize::Vec({})", m.size),
						VarType::Scalar => format!("TypeSize::Scalar"),
						// _ => panic!("===="),
					};
					let (kind, default_number) = match m.kind {
						ScalarKind::Sint => ("u32", "0"),
						ScalarKind::Uint => ("i32", "0"),
						ScalarKind::Float => ("f32", "0.0"),
						ScalarKind::Bool => panic!("===="),
					};
					let default_value = match code_slice.default_value.get(&m.name) {
						Some(r) => r.clone(), 
						None => {
							// "None".to_string()
							let mut v = Vec::with_capacity(m.width);
							for _ in 0..m.size {
								v.push(default_number);
							}
							v.join(",")
						},
					};
					uniform_expand.push(format!(r#"
						BindingExpandDesc::new_buffer::<{}>("{}", &[{}],  TypeKind::{}, {}, ArrayLen::{:?})
					"#, kind, m.name, default_value, kind_to_ty(&m.kind),  size, m.arr_len
					));
				}

				// 定义BindLayout， 实现BufferSize
				out_code.push(format!("
					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set({}), binding({}){})]
					#[min_size({})]
					#[uniformbuffer]
					pub struct {}Bind; // storagebuffer: TODO
				", slot.group, slot.binding, arr_len_str(&buffer_binding.arr_len), alignment_size, bind_class_case_name));

				// 为Bind实现binding_type
				entrys.push(format!("
					meta.add_binding_entry({}, (
						{}Bind::as_layout_entry(visibility),
						BindingExpandDescList::new(vec![{}], merge_defines(defines, &[{}]))
					));
				", slot.group, bind_class_case_name, uniform_expand.join(","), defines));
				// uniform_expand.push(format!("
								
				// 			", kind, default_value, ty, m.name
				// 			));

				// 实现Uniform
				for buffer_member in buffer_binding.merbers.iter() {
					let ty = get_type(buffer_member.kind);

					let name = buffer_member.name.to_class_case();
					// 定义Uniform结构体
					out_code.push(format!("
						#[derive(Uniform)]
						#[uniform(offset({}), len({}), bind({}Bind))]
						pub struct {}Uniform<'a>(pub &'a[{}]);
					",
						buffer_member.alignment, buffer_member.size * buffer_member.width, bind_class_case_name, name, ty
					));
				}
			},
			BindingType::Image(dim, class, len) => {
				let (sample_type, multi) = match class {
					ImageClass::Sampled { kind, multi } => (
						match kind {
							ScalarKind::Sint => "Sint",
							ScalarKind::Uint => "Uint",
							ScalarKind::Float => "Float",
							ScalarKind::Bool => continue,
						},
						multi,
					),
					ImageClass::Depth { multi } => {
						("Depth", multi)
					}
					ImageClass::Storage { .. } => todo!(),
				};
				let dim = match dim {
					ImageDimension::D1 => "D1",
					ImageDimension::D2 => "D2",
					ImageDimension::D3 => "D2",
					ImageDimension::Cube => "Cube",
				};

				// "kind" {
				// 	let v = Ident::parse_any(content)?;
				// 	self.kind = Some(v);
				// } else if key.to_string() == "multi" {
				// 	self.multi = true;
				// } else if key.to_string() == "dim" {

				out_code.push(format!("
					#[derive(BindLayout, BindingType)]
					#[layout(set({}), binding({}){})]
					#[texture(dim({}), multi({}), kind({}))]
					pub struct {}Bind; // storagetexture: TODO
				", slot.group, slot.binding, arr_len_str(&len), dim, multi, sample_type, bind_class_case_name));

				// 为Bind实现binding_type
				entrys.push(format!(r#"
					meta.add_binding_entry({}, (
							{}Bind::as_layout_entry(visibility), 
							BindingExpandDescList::new(vec![BindingExpandDesc::new_texture("{}")], merge_defines(defines, &[{}]))));
				"#, slot.group, bind_class_case_name, binding_name, defines));
			}, 

			BindingType::Sampler(comparison, len) => {
				let filtering = if *comparison {
					"Comparison"
				} else {
					"Filtering"
				};
				out_code.push(format!("
					#[derive(BindLayout, BindingType)]
					#[layout(set({}), binding({}){})]
					#[sampler({})]
					pub struct {}Bind;
				", slot.group, slot.binding, arr_len_str(&len), filtering, bind_class_case_name));
				entrys.push(format!(r#"
					meta.add_binding_entry({}, ({}Bind::as_layout_entry(visibility), BindingExpandDescList::new(vec![BindingExpandDesc::new_sampler("{}")], merge_defines(defines, &[{}]))));
				"#, slot.group, bind_class_case_name, binding_name, defines));

			},
		};
	}

	let mut push_code = Vec::new();
	let mut codes = Vec::new();
	let mut push_meta = Vec::new();
	let mut j = 0;

	for code in code_slice.code.iter() {
		let defines = code.defines.iter().map(|r| {format!("Define::new({}, {}[{}].clone())", r.0, define_name, r.1 )}).collect::<Vec<String>>().join(",");
		match &code.content {
			CodeItem::Code{run_code, other_code} => {
				if run_code.as_str() != "" {
					codes.push(format!(r#"CodeSlice{{code:Atom::from("{}"), defines: vec![{}]}}"#, &run_code, defines));
					push_code.push(format!("codes.running.push({}[{}].clone().push_defines_front(defines));", code_name, j));
					j += 1;
				}
				if other_code.as_str() != "" {
					codes.push(format!(r#"CodeSlice{{code:Atom::from("{}"), defines: vec![{}]}}"#, &other_code, defines));
					push_code.push(format!("codes.define.push({}[{}].clone().push_defines_front(defines));", code_name, j));
					j += 1;
				}
			},
			CodeItem::Import(path) => {
				if let ShaderImport::Path(path) = path {
					push_code.push(format!("{}::push_code(codes, merge_defines(defines, &[{}]).as_slice());", path, defines));
					push_meta.push(format!("{}::push_meta(meta, visibility, merge_defines(defines, &[{}]).as_slice()));", path, defines));
				}
			},
		}
	}
	
	Ok (UniformCode {
		out_code: out_code.join("\n"),
		entrys,
		codes,
		push_code,
		push_meta,
	})
}


pub struct UniformCode {
	out_code: String,
	entrys: Vec<String>,
	codes: Vec<String>,
	push_code: Vec<String>,
	push_meta: Vec<String>,
}

// // 分类过的uniform
// #[derive(Default)]
// struct UniformWithClassify<'a> {
// 	buffer_info: Vec<LayoutInfo>,
// 	texture_sampler_info: Vec<&'a Type>,
// }

#[derive(Debug, Clone)]
enum BindingType {
	Buffer(LayoutInfo),
	Image(ImageDimension, ImageClass, ArrayLen),
	Sampler(bool, ArrayLen)
}

// pub fn compile(
//     shader_name: &str,
//     vs_shader: ProcessedShader,
//     fs_shader: ProcessedShader,
// ) -> Result<String, ShaderReflectError> {
//     let vs_module = match &vs_shader {
//         ProcessedShader::Wgsl(source) => naga::front::wgsl::parse_str(source)?,
//         ProcessedShader::Glsl(source, shader_stage) => {
//             let mut parser = naga::front::glsl::Parser::default();
//             parser
//                 .parse(&naga::front::glsl::Options::from(*shader_stage), source)
//                 .map_err(ShaderReflectError::GlslParse)?
//         }
//         ProcessedShader::SpirV(source) => naga::front::spv::parse_u8_slice(
//             source,
//             &naga::front::spv::Options {
//                 adjust_coordinate_space: false,
//                 ..naga::front::spv::Options::default()
//             },
//         )?,
//     };
//     let fs_module = match &fs_shader {
//         ProcessedShader::Wgsl(source) => naga::front::wgsl::parse_str(source)?,
//         ProcessedShader::Glsl(source, shader_stage) => {
//             let mut parser = naga::front::glsl::Parser::default();
//             parser
//                 .parse(&naga::front::glsl::Options::from(*shader_stage), source)
//                 .map_err(ShaderReflectError::GlslParse)?
//         }
//         ProcessedShader::SpirV(source) => naga::front::spv::parse_u8_slice(
//             source,
//             &naga::front::spv::Options {
//                 adjust_coordinate_space: false,
//                 ..naga::front::spv::Options::default()
//             },
//         )?,
//     };
//     println!("vs: {:?} \n fs: {:?}", vs_module, fs_module);

// 	let mut input_strs = Vec::new();

// 	parse_input(&vs_module, &mut input_strs);

//     let vs_uniforms = get_bindings(&vs_module);
//     let fs_uniforms = get_bindings(&fs_module);

//     let uniforms = merge_uniforms(
//         &vs_uniforms,
//         &fs_uniforms,
//         &vs_module.types,
//         &fs_module.types,
//     );

//     let mut functions = Vec::new();
// 	let mut buffer_uniforms = Vec::new();
// 	let mut buffer_bindings = Vec::new();
//     let mut groups = Vec::new();
//     for pi_ordmap::ordmap::Entry(group, bindings) in uniforms.iter(None, false) {
//         let mut bindings_layout_info = Vec::new();
//         let mut bindings_info = Vec::new();
//         let mut group_params = Vec::new();
// 		let mut group_layout_params = Vec::new();

//         let mut name1 = "".to_string();
// 		let mut i = 0;
// 		let mut is_all_buffer = true;
//         for pi_ordmap::ordmap::Entry((bind, visibility), (item, var_name, from)) in bindings.iter(None, false) {
//             let name = if let Some(r) = &item.name {
//                 r
//             } else {
//                 match var_name {
//                     Some(r) => r,
//                     None => continue,
//                 }
//             };
//             name1 = name1 + "_" + name.to_snake_case().as_str();

//             let mut infos = UniformWithClassify::default();
//             let mut layout_info = LayoutInfo::default();
//             get_layout_info(
//                 &item,
//                 &mut infos,
//                 &mut layout_info,
//                 match from {
//                     UniformFrom::Vert => &vs_module.types,
//                     UniformFrom::Frag => &fs_module.types,
//                 },
//             );

//             let mut visibility_str = Vec::new();
//             if visibility & (UniformFrom::Vert as usize) > 0 {
//                 visibility_str.push("wgpu::ShaderStages::VERTEX");
//             }
//             if visibility & (UniformFrom::Frag as usize) > 0 {
//                 visibility_str.push("wgpu::ShaderStages::FRAGMENT");
//             }
//             let mut tail = layout_info.size;

//             for item in infos.buffer_info.iter() {
//                 let ty = get_type(item.kind);

// 				let name = item.name.to_class_case();

// 				buffer_uniforms.push(format!("
// 				pub struct {}Uniform<'a>(pub &'a[{}]);
// 				impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for {}Uniform<'a> {{
// 					fn write_into(&self, index: u32, buffer: &mut [u8]) {{
// 						unsafe {{ std::ptr::copy_nonoverlapping(
// 							self.0.as_ptr() as usize as *const u8,
// 							buffer.as_mut_ptr().add(index as usize + {}),
// 							{},
// 						) }};
// 					}}
// 				}}
// 				",
//                     name, ty, name, item.alignment, item.size * item.width
//                 ));

//                 // let name = item.name.to_snake_case();
//                 // // set_uniform
//                 // functions.push(format!(
//                 //     "
// 				// 	pub fn set_{}(&mut self, index: &pi_render::rhi::block_alloc::BlockIndex, {}: &[{}]) -> Result<(), String> {{
// 				// 		self.context.lock().unwrap().set_uniform(index, {}, bytemuck::cast_slice({}))
// 				// 	}}
// 				// ",
//                 //     name, name, ty, item.alignment, name
//                 // ));;
//             }
// 			if infos.buffer_info.len() > 0 {
// 				group_params.push("buffer: &wgpu::Buffer".to_string());
// 				group_layout_params.push("has_dynamic_offset: bool".to_string());
// 				let name = name.to_class_case();
// 				buffer_bindings.push(format!("
// 					pub struct {}Bind;
// 					impl pi_render::rhi::dyn_uniform_buffer::Bind for {}Bind {{
// 						#[inline]
// 						fn min_size() -> usize {{
// 							{}
// 						}}

// 						fn index() -> pi_render::rhi::dyn_uniform_buffer::BindIndex {{
// 							pi_render::rhi::dyn_uniform_buffer::BindIndex::new({})
// 						}}
// 					}}
// 				", name, name, tail, i));
// 				bindings_layout_info.push(format!(
//                     "
// 					wgpu::BindGroupLayoutEntry {{
// 						binding: {},
// 						visibility: {},
// 						ty: wgpu::BindingType::Buffer {{
// 							ty: wgpu::BufferBindingType::Uniform,
// 							has_dynamic_offset,
// 							min_binding_size: wgpu::BufferSize::new({}),
// 						}},
// 						count: None, // TODO
// 					}}
// 				",
//                     bind,
//                     visibility_str.join(" | "),
//                     tail
//                 )); 
//                 bindings_info.push(format!(
//                     "
// 					wgpu::BindGroupEntry {{
// 						binding: {},
// 						resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {{
// 							buffer,
// 							offset: 0,
// 							size: Some(std::num::NonZeroU64::new({}).unwrap())
// 						}}),
// 					}}
// 				",
//                     bind,
//                     tail
//                 ));
// 			}

//             for item in infos.texture_sampler_info.iter() {
//                 let name = name.to_snake_case();
// 				is_all_buffer = false;

//                 match &item.inner {
//                     naga::TypeInner::Image {
//                         dim,
//                         class,
// 						..
//                     } => {
//                         let (sample_type, multi) = match class {
//                             naga::ImageClass::Sampled { kind, multi } => (
//                                 format!(
//                                     "wgpu::TextureSampleType::{}",
//                                     match kind {
//                                         ScalarKind::Sint => "Sint",
//                                         ScalarKind::Uint => "Uint",
//                                         ScalarKind::Float => "Float{filterable:true}",
//                                         ScalarKind::Bool => continue,
//                                     }
//                                 ),
//                                 multi,
//                             ),
//                             naga::ImageClass::Depth { multi } => {
//                                 ("wgpu::TextureSampleType::Depth".to_string(), multi)
//                             }
//                             naga::ImageClass::Storage { .. } => todo!(),
//                         };
//                         let dim = match dim {
//                             naga::ImageDimension::D1 => "wgpu::TextureViewDimension::D1",
//                             naga::ImageDimension::D2 => "wgpu::TextureViewDimension::D2",
//                             naga::ImageDimension::D3 => "wgpu::TextureViewDimension::D2",
//                             naga::ImageDimension::Cube => "wgpu::TextureViewDimension::Cube",
//                         };

//                         // wgpu::TextureSampleType

//                         bindings_layout_info.push(format!(
//                             "
// 							wgpu::BindGroupLayoutEntry {{
// 								binding: {},
// 								visibility:{},
// 								ty: wgpu::BindingType::Texture {{
// 									multisampled: {},
// 									sample_type: {},
// 									view_dimension: {},
// 								}},
// 								count: None, // TODO
// 							}}
// 						",
//                             bind,
//                             visibility_str.join(" | "),
//                             multi,
//                             sample_type,
//                             dim
//                         ));
//                         bindings_info.push(format!(
//                             "
// 							wgpu::BindGroupEntry {{
// 								binding: {},
// 								resource: wgpu::BindingResource::TextureView({}),
// 							}}
// 						",
//                             bind, name
//                         ));
//                         group_params.push(format!("{}: &wgpu::TextureView", name));
//                     }
//                     naga::TypeInner::Sampler { comparison } => {
//                         let filtering = if *comparison {
//                             "wgpu::SamplerBindingType::Comparison"
//                         } else {
//                             "wgpu::SamplerBindingType::Filtering"
//                         };
//                         bindings_layout_info.push(format!(
//                             "
// 							wgpu::BindGroupLayoutEntry {{
// 								binding: {},
// 								visibility: {},
// 								ty: wgpu::BindingType::Sampler({}),
// 								count: None,
// 							}}
// 						",
//                             bind,
//                             visibility_str.join(" | "),
//                             filtering
//                         ));
//                         bindings_info.push(format!(
//                             "
// 							wgpu::BindGroupEntry {{
// 								binding: {},
// 								resource: wgpu::BindingResource::Sampler({}),
// 							}}
// 						",
//                             bind, name
//                         ));
//                         group_params.push(format!("{}: &wgpu::Sampler", name));
//                     }
//                     _ => continue,
//                 }
//             }
// 			i += 1;
//         }

//         let name = name1.to_snake_case();
// 		let class_name = name.to_class_case();

//         // groups
//         groups.push(format!(
//             "
// 			pub struct {}Group;
// 			impl pi_render::rhi::dyn_uniform_buffer::Group for {}Group {{
// 				fn id() -> u32 {{
// 					{}
// 				}}

// 				fn create_layout(device: &pi_render::rhi::device::RenderDevice, has_dynamic_offset: bool) -> pi_render::rhi::bind_group_layout::BindGroupLayout {{
// 					device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {{
// 						label: Some(\"{} bindgroup layout\"),
// 						entries: &[
// 							{}
// 						],
// 					}})
// 				}}
// 			}}
// 		",
// 			class_name,
// 			class_name,
//             *group,

//             name,
//             bindings_layout_info.join(",")
//         ));

// 		if is_all_buffer {
// 			groups.push(format!(
// 				"
// 				impl pi_render::rhi::dyn_uniform_buffer::BufferGroup for {}Group {{
// 					fn create_bind_group(device: &pi_render::rhi::device::RenderDevice, layout: &pi_render::rhi::bind_group_layout::BindGroupLayout, buffer: &pi_render::rhi::buffer::Buffer) -> pi_render::rhi::bind_group::BindGroup {{
// 						device.create_bind_group(&wgpu::BindGroupDescriptor {{
// 							layout,
// 							entries: &[
// 								{}
// 							],
// 							label: Some(\"{} bindgroup\"),
// 						}})
// 					}}
// 				}}
// 			",
// 				class_name,
// 				bindings_info.join(","),
// 				name,
// 			));
// 		} else {
// 			functions.push(format!("
// 				pub fn create_bind_group_{}( device: &pi_render::rhi::device::RenderDevice, layout: &pi_render::rhi::bind_group_layout::BindGroupLayout, {}) -> pi_render::rhi::bind_group::BindGroup {{
// 					device.create_bind_group(&wgpu::BindGroupDescriptor {{
// 						layout,
// 						entries: &[
// 							{}
// 						],
// 						label: Some(\"{} bindgroup\"),
// 					}})
// 				}}
// 			", name, group_params.join(","), bindings_info.join(","), name));
// 		}
//     }

//     let share_name = shader_name.to_class_case();
//     let out_put = format!(
//         "
// 		{}
// 		{}
// 		{}
// 		{}
// 		pub struct {}Shader;

// 		impl {}Shader {{
// 			{}
// 		}}
// 	",
// 		input_strs.join("\n"),
// 		groups.join("\n"),
// 		buffer_bindings.join("\n"),
// 		buffer_uniforms.join("\n"),
//         share_name,
//         share_name,
//         functions.join("\n")
//     );

//     println!("out_put: {:?}", out_put);

//     return Ok(out_put);
// }

pub fn get_type(kind: ScalarKind) -> String {
    match kind {
        ScalarKind::Sint => "i32".to_string(),
        ScalarKind::Uint => "u32".to_string(),
        ScalarKind::Float => "f32".to_string(),
        ScalarKind::Bool => "bool".to_string(),
    }
}

pub fn arr_len_str(arr_len: &ArrayLen) -> String {
	match arr_len {
		ArrayLen::Constant(r) => format!(",count({})", r),
		_ => "".to_string(),
	}
}

pub fn arr_len_str1(arr_len: &ArrayLen) -> String {
	match arr_len {
		ArrayLen::Constant(r) => format!("std::num::NonZeroU32::new({})", r),
		_ => "None".to_string(),
	}
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BindingLocation {
	location: u32,
	interpolation: Option<Interpolation>,
	sampling: Option<Sampling>,
	name: String
}

fn get_inputs(vs_module: &Module, input_excludes: &XHashMap<u32, (ShaderId, String)>) -> Vec<BindingLocation> {
	let mut vec = Vec::new();
	for entry in vs_module.entry_points.iter() {
		if entry.stage == ShaderStage::Vertex {
			for arg in entry.function.arguments.iter() {
				if let (Some(name), Some(binding)) = (&arg.name, &arg.binding) {
					if let Binding::Location {location, interpolation, sampling} =  binding {
						if input_excludes.contains_key(location) {
							continue;
						}
						vec.push(BindingLocation {location: *location, interpolation: interpolation.clone(), sampling: sampling.clone(), name: name.clone()});
					}
					
				}
			}
		}
	}
	vec
}

fn parse_input(
	input_strs: &mut Vec<String>,
	inputs: &Vec<BindingLocation>
) {
	for BindingLocation{location, name, ..}  in inputs.iter() {
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

#[derive(Debug, Clone)]
struct LayoutInfo {
	/// uniform 名称
    name: String, 
	/// uniform 对齐
    alignment: u32,
	/// 数量
    size: usize,
	/// uniform 类型大小（sint， float， bool,uint）
    width: usize,
	/// 类型
    kind: ScalarKind,
	ty: VarType,

	/// 成员
	merbers: Vec<LayoutInfo>,
	/// len
	alignment_size: usize, // 包含所有成员的buffer长度（已对齐）
	span: usize, // 包含所有成员的buffer长度（未）

	// 如果是数组, 该值大于0
	arr_len: ArrayLen,
}

#[derive(Debug, Clone)]
pub enum VarType {
	Mat,
	Vector,
	Scalar,
}

#[derive(Debug, Clone)]
pub enum ArrayLen {
	Constant(usize),
	Dynamic,
	None
}


// #[derive(Debug, Clone)]
// pub enum VarType {
// 	Base(VarTypeBase),
// 	Array(VarTypeBase, ),
// }

// pub enum ArraySize {
// 	Constant(usize),
// 	Dynamic,
// }
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
			ty: VarType::Scalar,
			merbers: Default::default(),
			alignment_size: 0,
			span: 0,
			arr_len: ArrayLen::None,
        }
    }
}

fn kind_to_ty(kind: &ScalarKind) -> &'static str {
	match kind {
		ScalarKind::Sint => "SInt",
		ScalarKind::Uint => "UInt",
		ScalarKind::Float => "Float",
		ScalarKind::Bool => "",
	}
}

// fn compare_type(
//     ty1: &naga::Type,
//     ty2: &naga::Type,
//     types1: &naga::UniqueArena<naga::Type>,
//     types2: &naga::UniqueArena<naga::Type>,
// ) -> bool {
//     if &ty1.name != &ty2.name {
//         return false;
//     }

//     match (&ty1.inner, &ty2.inner) {
//         (
//             naga::TypeInner::Struct {
//                 members: members1,
//                 span: span1,
//             },
//             naga::TypeInner::Struct {
//                 members: members2,
//                 span: span2,
//             },
//         ) => {
//             if span1 != span2 || members1.len() != members2.len() {
//                 return false;
//             }
//             for i in 0..members1.len() {
//                 let (m1, m2) = (&members1[i], &members2[i]);
//                 if m1.offset != m2.offset || &m1.name != &m2.name {
//                     return false;
//                 }

//                 if !compare_type(&types1[m1.ty], &types2[m2.ty], types1, types1) {
//                     return false;
//                 }
//             }
//             return true;
//         }
//         _ => &ty1.inner == &ty2.inner,
//     }
// }

fn get_binding_types<'a, 'b>(
    ty: &'a Type,
    types: &'a UniqueArena<Type>,
	constants: &'a Arena<Constant>,
) -> Result<BindingType, CompileShaderError> {
    match &ty.inner {
        TypeInner::Image { dim, class, .. } => {
			Ok(BindingType::Image(*dim, *class, ArrayLen::None))
        },
		TypeInner::Sampler { comparison } => {
			Ok(BindingType::Sampler(*comparison, ArrayLen::None))
        },
		TypeInner::Array { base, size, .. } => {
			let ty = get_binding_types(&types[base.clone()], types, constants)?;
			let arr_len = match size {
				ArraySize::Constant(r) => {
					let c_ty = &constants[r.clone()];
					let len = match c_ty.inner {
						ConstantInner::Scalar { value, .. } => match value {
							ScalarValue::Uint(r) => r as usize,
							_ => return Err(CompileShaderError::ValidationVarFail("ScalarValue::Uint".to_string(), format!("{:?}", value))),
						},
						_ => return Err(CompileShaderError::ValidationVarFail("ConstantInner::Scalar".to_string(), format!("{:?}", c_ty.inner))),
					};
					ArrayLen::Constant(len)
				},
				ArraySize::Dynamic => ArrayLen::Dynamic
			};
			match ty {
				BindingType::Buffer(layout_info) => {
					let mut cur_info = LayoutInfo::default();
					cur_info.arr_len = arr_len;

					cur_info.width = layout_info.width;
					cur_info.size = layout_info.size;
					cur_info.kind = layout_info.kind;
					cur_info.alignment_size = layout_info.alignment_size;
					Ok(BindingType::Buffer(cur_info))
				},
				BindingType::Image(r1, r2, _r3) => {
					Ok(BindingType::Image(r1, r2, arr_len))
				},
				BindingType::Sampler(r1, _r2) => {
					Ok(BindingType::Sampler(r1, arr_len))
				},
			}
			
		},
        _ => Ok(BindingType::Buffer(get_buffer_binding_layout(ty, types, constants)?)),
    }
}

fn get_buffer_binding_layout<'a, 'b>(
    ty: &'a Type,
    types: &'a UniqueArena<Type>,
	constants: &'a Arena<Constant>,
) -> Result<LayoutInfo, CompileShaderError> {
	let mut cur_info = LayoutInfo::default();
    match &ty.inner {
        TypeInner::Scalar { width, kind } => {
            cur_info.width = *width as usize;
            cur_info.size = 1;
            cur_info.kind = kind.clone();
			cur_info.span = *width as usize;
			cur_info.ty = VarType::Scalar;
        }
        TypeInner::Vector { size, kind, width } => {
            cur_info.width = *width as usize;
            cur_info.size = *size as u8 as usize;
			cur_info.span = cur_info.size * cur_info.width;
            cur_info.kind = kind.clone();
			cur_info.ty = VarType::Vector;
        }
        TypeInner::Matrix {
            columns,
            rows,
            width,
        } => {
            cur_info.width = *width as usize;
            cur_info.size = (*columns as u8 * *rows as u8) as usize;
            cur_info.kind = ScalarKind::Float;
			cur_info.ty = VarType::Mat;
			cur_info.span = (*columns as u8 *width) as usize;
        }
        TypeInner::Atomic { kind, width } => {
            cur_info.size = 1;
            cur_info.width = *width as usize;
			cur_info.span = *width as usize;
            cur_info.kind = kind.clone();
			todo!();
        }
        TypeInner::Struct { members, span } => {
            for item in members.iter() {
                let mut layout_info = get_buffer_binding_layout(&types[item.ty], types, constants)?;
                if let Some(r) = &item.name {
                    layout_info.alignment = item.offset;
                    layout_info.name = r.clone();
					cur_info.merbers.push(layout_info)
                }
            }
			cur_info.span = *span as usize;
        }
        _ => return Err(CompileShaderError::TypeNotSupport(format!("{:?}", ty.inner))),
    };
	Ok(cur_info)
}

// fn get_layout_info<'a, 'b>(
//     ty: &'a Type,
//     infos: &'b mut UniformWithClassify<'a>,
//     cur_info: &mut LayoutInfo,
//     types: &'a UniqueArena<Type>,
// ) {
//     match &ty.inner {
//         TypeInner::Scalar { width, kind } => {
//             cur_info.width = *width as usize;
//             cur_info.size = 1;
//             cur_info.kind = kind.clone();
//         }
//         TypeInner::Vector { size, kind, width } => {
//             cur_info.width = *width as usize;
//             cur_info.size = unsafe { transmute::<_, u8>(*size) } as usize;
//             cur_info.kind = kind.clone();
//         }
//         TypeInner::Matrix {
//             columns,
//             rows,
//             width,
//         } => {
//             cur_info.width = *width as usize;
//             cur_info.size =
//                 unsafe { transmute::<_, u8>(*columns) * transmute::<_, u8>(*rows) } as usize;
//             cur_info.kind = ScalarKind::Float;
//         }
//         TypeInner::Atomic { kind, width } => {
//             cur_info.size = 1;
//             cur_info.width = *width as usize;
//             cur_info.kind = kind.clone();
//         }
//         TypeInner::Struct { members, span } => {
//             cur_info.size = *span as usize;
//             for item in members.iter() {
//                 let mut layout_info = LayoutInfo::default();
//                 get_layout_info(&types[item.ty], infos, &mut layout_info, types);
//                 if let Some(r) = &item.name {
//                     layout_info.alignment = item.offset;
//                     layout_info.name = r.clone();
//                     infos.buffer_info.push(layout_info);
//                 }
//             }
//         }
//         TypeInner::Image { .. } | TypeInner::Sampler { .. } => {
//             infos.texture_sampler_info.push(ty);
//         }
//         // TypeInner::Array { base, size, stride } => todo!(),
//         _ => (),
//     };
// }

// fn merge_binding<'a>(
//     vs_slots: &'a XHashMap<UniformSlot, (BindingType, String)>,
//     fs_slots: &'a XHashMap<UniformSlot, (BindingType, String)>,
// ) -> OrdMap<Tree<u32, OrdMap<Tree<(u32, usize), (&'a BindingType, &'a String, UniformFrom)>>>> {
//     let mut r = XHashMap::default();
//     for (k, item) in vs_slots.iter() {
// 		if fs_slots.contains_key(k) {
// 			r.insert(
// 				(
// 					k.group,
// 					k.binding,
// 					(UniformFrom::Vert as usize) | (UniformFrom::Frag as usize),
// 				),
// 				(&item.0, &item.1, UniformFrom::Vert),
// 			);
// 		}
//     }

//     for (k, item) in fs_slots.iter() {
//         if let None = vs_slots.get(k) {
//             r.insert(
//                 (k.group, k.binding, (UniformFrom::Frag as usize)),
//                 (&item.0, &item.1, UniformFrom::Frag),
//             );
//         }
//     }

//     let mut ret = OrdMap::new(Tree::new());

//     for ((group, bind, share), item) in r.into_iter() {
//         match ret.get(&group) {
//             None => {
//                 let mut map = OrdMap::new(Tree::new());
//                 map.insert((bind,share), (item.0, item.1, item.2));
//                 ret.insert(group, map);
//             }
//             Some(_r) => {
// 				let mut r = ret.delete(&group, true).unwrap().unwrap();
//                 r.insert((bind, share), (item.0, item.1, item.2));
// 				ret.insert(group, r);
//             }
//         };
//     }
//     ret
// }

// #[derive(Clone, Copy)]
// enum UniformFrom {
//     Vert = 1,
//     Frag = 2,
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniformSlot {
	group: u32,
	binding: u32,
}
impl UniformSlot {
	pub fn new(group: u32, binding: u32) -> Self {
		UniformSlot {group, binding}
	}
}

fn get_bindings(module: &Module, binding_excludes: &XHashMap<UniformSlot, (ShaderId, BindingType, String)>) -> XHashMap<UniformSlot, (BindingType, String)> {
    let mut uniforms = XHashMap::default();
    for item in module.global_variables.iter() {
        if item.1.space == AddressSpace::Uniform || item.1.space == AddressSpace::Handle {
            let ty = &module.types[item.1.ty];
            let (group, binding) = match &item.1.binding {
                Some(r) => (r.group, r.binding),
                None => panic!("Uniform does not set binding: {:?}", ty.name),
            };
			// // 如果是内建变量， 则忽略
			// if group == ShaderVarTypes::SET && binding == ShaderVarTypes::BINDIND  {
			// 	continue;
			// }
			if binding_excludes.contains_key(&UniformSlot::new(group, binding)) {
				continue;
			}
			println!("zzzz!!=========={:?}, {:?}", UniformSlot::new(group, binding), binding_excludes.keys());
            match uniforms.entry(UniformSlot::new(group, binding)) {
                Entry::Vacant(r) => {
					let mut binding_type = match get_binding_types(ty, &module.types, &module.constants) {
						Ok(r) => r,
						_ => continue,
					};
					// 排序并对齐
					if let BindingType::Buffer(info) = &mut binding_type {
						sort_and_alignment(info, group, binding);
					}
					let name = match &item.1.name {
						Some(r) => r.clone(),
						None => ty.name.clone().unwrap(),
					};
					println!("name======{:?}, {:?}, {:?}", item.1.name, ty.name, name);
                    r.insert((binding_type, name));
                }
                Entry::Occupied(_r) => panic!("Uniform setting binding repeated: {:?}", ty.name),
            };
        }
    }

    uniforms
}

// 根据类型对齐
fn type_alignment(size: usize) -> usize {
	if size <= 4 {
		return 4;
	} else if size <= 8 {
		return 8;
	} else {
		return 16;
	}
}

// 对binding中的元素排序并对齐
// 暂时不处理数组和非结构体类型的buffer
fn sort_and_alignment(layout_info: &mut LayoutInfo, set: u32, binding: u32) {
	match  layout_info.arr_len {
		ArrayLen::Constant(_) | ArrayLen::Dynamic => return,
		ArrayLen::None => (),
	}
	if layout_info.merbers.len() > 0 {
		// 根据内存占用，从大到小排序
		layout_info.merbers.sort_by(|a, b| {
			if a.width * a.size > b.width * b.size {
				Ordering::Less
			} else if a.width * a.size < b.width * b.size {
				Ordering::Greater
			} else {
				Ordering::Equal
			}
		});

		let mut offset = 0;
		for m in layout_info.merbers.iter_mut() {
			let alignment = type_alignment(m.span);
			let remain = offset % alignment;
			if remain > 0 {
				offset += alignment - remain;
			}
			m.alignment = offset as u32;
			offset += m.width * m.size;
		}
		let remain = offset % 16;
		if remain > 0 {
			let patch = 16 - remain;
			let (size, ty, alignment) = if patch == 12 {
				(3, VarType::Vector, offset - remain)
			} else if patch == 4 {
				(1, VarType::Scalar, offset)
			} else {
				(patch/4, VarType::Vector, offset)
			};
			layout_info.merbers.push(LayoutInfo { 
				name: "PATCH__".to_string() + set.to_string().as_str() + "_" + binding.to_string().as_str(), 
				alignment: alignment as u32, 
				size: size, 
				width: 4, 
				kind: ScalarKind::Float, 
				ty, 
				merbers: Vec::new(), 
				alignment_size: 0, 
				span: 0, 
				arr_len: ArrayLen::None
			} );
			if patch == 12 {
				// patch一个vec3， 则前一个变量一定为四字节，交换两变量位置，以符合对齐要求
				let len = layout_info.merbers.len();
				layout_info.merbers.swap(len - 1, len - 2);
				let last = layout_info.merbers.last_mut().unwrap();
				last.alignment += 12;
			}
			offset += patch;
		}
		layout_info.alignment_size = offset;
	}
}

// #[derive(Error, Debug)]
// pub enum ShaderReflectError {
//     #[error(transparent)]
//     WgslParse(#[from] naga::front::wgsl::ParseError),

//     #[error("GLSL Parse Error: {0:?}")]
//     GlslParse(Vec<naga::front::glsl::Error>),

//     #[error(transparent)]
//     SpirVParse(#[from] naga::front::spv::Error),

//     #[error(transparent)]
//     Validation(#[from] naga::WithSpan<naga::valid::ValidationError>),
// }


struct CodeLoaderFromFile;
impl CodeLoader for CodeLoaderFromFile {
	fn load(&self, path: &PathBuf) -> Result<Vec<u8>, ProcessShaderError> {
		match std::fs::read(path) {
			Ok(r) => Ok(r),
			_ => Err(ProcessShaderError::LoadFail(path.clone()))
		}
	}
}

// fn canonicalize(src_path: &PathBuf, cur_path: &PathBuf, str: &str) -> Result<PathBuf, CompileShaderError> {
// 	let mut arr = str.split("::");
// 	let first = match arr.next() {
// 		Some(r) => r,
// 		None => return Err(CompileShaderError::InvalidImportPath(str.to_string()))
// 	};
// 	let p = if first == "super" {
// 		cur_path.join("../")
// 	} else if first == "crate"{
		
// 	}
// 	todo!()
// }

// #[test]
// fn test() {
//     use std::process::Command;

// 	let vs_code = include_str!("../source/text.vert");
//     let fs_code = include_str!("../source/text.frag");
// 	compile_and_out(
// 		"text", 
// 		ProcessedShader::Glsl(Cow::Borrowed(vs_code), naga::ShaderStage::Vertex),
// 		ProcessedShader::Glsl(Cow::Borrowed(fs_code), naga::ShaderStage::Fragment), 
// 		Path::new("out/"));

   

//     // let root_dir = std::env::current_dir().unwrap();
//     // println!("root_dir: {:?}", root_dir);

//     // let r = compile(
//     //     "Color",
//     //     ProcessedShader::Glsl(Cow::Borrowed(vs_code), naga::ShaderStage::Vertex),
//     //     ProcessedShader::Glsl(Cow::Borrowed(fs_code), naga::ShaderStage::Fragment),
//     // );

// 	// let file_name = "color.rs";
//     // match r {
//     //     Ok(r) => std::fs::write(root_dir.join("output").join(file_name), r),
//     //     Err(r) => panic!("{:?}", r),
//     // };

//     // Command::new("rustfmt")
//     //     .arg(format!("{}", file_name))
// 	// 	.arg("--config")
// 	// 	.arg("hard_tabs=true")
//     //     .current_dir(root_dir.join("output"))
//     //     .status()
//     //     .unwrap();
// }

// #[test]
// fn testx() {
// 	let name = "ColorMatr";
// 	println!("{}, {}", name.to_snake_case(), name.to_snake_case().to_class_case());
// }

// #[test]
// fn test1() {
//     use std::process::Command;

// 	let vs_code = r#"
// #version 450
// #import "yy.glsl"
// layout(set = 1, binding = 0) uniform ColorMaterial {
// 	mat4 world;
// 	float zz;
// 	vec4 xxxx; // default vec4(50.0, 0.0, 0.0, 0.0)
// };
// void main() {
// }
// "#;
//     let fs_code = "
// #version 450
// void main() {
// }
// ";
// 	compile_and_out(
// 		"text", 
// 		ProcessedShader::Glsl(Cow::Borrowed(vs_code), naga::ShaderStage::Vertex),
// 		ProcessedShader::Glsl(Cow::Borrowed(fs_code), naga::ShaderStage::Fragment), 
// 		Path::new("out/"));
// }

// #[cfg(test)]
// mod test {
// 	use std::path::PathBuf;

// use pi_hash::XHashMap;
// 	use render_core::rhi::shader::{Shader, CodeLoader, ProcessShaderError, ShaderProcessor, AllDefineds, ShaderCodeMgr};
	
// 	struct CodeLoaderFromFile;
// 	impl CodeLoader for CodeLoaderFromFile {
// 		fn load(&self, path: &PathBuf) -> Result<Vec<u8>, ProcessShaderError> {
// 			match std::fs::read(path) {
// 				Ok(r) => Ok(r),
// 				_ => Err(ProcessShaderError::LoadFail(path.clone()))
// 			}
// 		}
// 	}

// 	#[test]
// 	fn test_import_path() {
	

// 		let vs_code = r#"
// #import "./camera.glsl"
// void main() {
// }
// "#;
// 		let mut shaders = XHashMap::default();
// 		let shader = Shader::from_glsl(vs_code, naga::ShaderStage::Vertex);
// 		let id = shader.id();
// 		shaders.insert(id, (shader, Some("source/xx.vert".to_string())));
		
// 		let processor = ShaderProcessor::new(CodeLoaderFromFile);
// 		let result = processor
// 		.process(&id, &AllDefineds, &mut ShaderCodeMgr{shaders, import_shaders: XHashMap::default()} )
// 		.unwrap();


// 		println!("======{:?}", result);
// 	}
// }

// #[test]
// fn test() {
// 	use std::path::{Path, PathBuf};
// 	use std::str::FromStr;

// 	let r = PathBuf::from_str("render_compile::src").unwrap();

// 	let path = Path::new("src/lib.rs");
// 	let path1 = Path::new("render_compile::aa.rs");
// 	let byte = std::fs::read_to_string(path1);
// 	let a: Box<Path> = path.canonicalize().unwrap().into();
// 	let b: Box<Path> = r.into();
// 	println!("====={:?}, {:?}, {:?}", a.to_string_lossy(), b, path1);
// 	println!("====={:?}", byte);
// // assert_eq!(a, PathBuf::from("/foo/test/bar.rs"));
// }

// layout (set = 2, binding = 1) uniform texture2D tex2d[];
#[test]
fn test_uniform_grammar() {

	let mut parser = pi_naga::front::glsl::Parser::default();
	let modlue = parser
				.parse(&pi_naga::front::glsl::Options::from(pi_naga::ShaderStage::Vertex), r#"
#version 450
layout(set=0, binding=0) uniform texture2DMS depth;

void main() {
	gl_Position.z = depth/60000.0 + depth1.x;
}
				"#);
	println!("modle================={:?}, \nmodle================={:?}", modlue, parser);
}

// #[test]
// fn test_gen_base() {
// 	let mut m = Parser::default();

// 	let share_slice = r#"
// 	#define_import_path aa::bb
// 	layout(set=0,binding=0) uniform vec2 camera;
// "#;
// 	let shader = Shader::from_glsl(share_slice, naga::ShaderStage::Vertex);
// 	let id = shader.id();
// 	if let Some(import_path) = shader.import_path() {
// 		m.shader_mgr.import_shaders.insert(import_path.clone(), id);
// 		m.shader_mgr.shaders.insert(id, (shader, Some("share.glsl".to_string())));
// 	}

// 	let vert_slice = r#"
// 	#import aa::bb
// 	layout(location = 0) in vec2 position;
// 	layout(set = 1, binding = 0) uniform ColorMaterial1 {
// 		mat4 world;
// 	};
// 	void main() {
// 	}
// "#;
// 	let shader = Shader::from_glsl(vert_slice, naga::ShaderStage::Vertex);
// 	let id = shader.id();
// 	if let Some(import_path) = shader.import_path() {
// 		m.shader_mgr.import_shaders.insert(import_path.clone(), id);
// 	}
// 	m.shader_mgr.shaders.insert(id, (shader, Some("test.vert".to_string())));

// 	let vert_slice = r#"
// 	#import aa::bb
// 	layout(set = 2, binding = 0) uniform ColorMaterial {
// 		mat4 world;
// 	};
// 	void main() {
// 	}
// "#;
// 	let shader = Shader::from_glsl(vert_slice, naga::ShaderStage::Vertex);
// 	let id = shader.id();
// 	if let Some(import_path) = shader.import_path() {
// 		m.shader_mgr.import_shaders.insert(import_path.clone(), id);
// 	}
// 	m.shader_mgr.shaders.insert(id, (shader, Some("test.frag".to_string())));

// 	let mut gen_paths = XHashSet::default();
// 	gen_paths.extend(["test.frag".to_string(), "test.vert".to_string()].into_iter());
// 	let result = m.parse1(&gen_paths);

// 	println!("result: {:?}", result);
// }



