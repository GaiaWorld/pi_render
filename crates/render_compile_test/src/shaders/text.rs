
			use pi_render::rhi::shader::{ShaderMeta, CodeSlice, BlockCodeAtom, InOut, Define,  ShaderVarying, ShaderInput, ShaderOutput, ShaderProgram, BindingExpandDescList,AsLayoutEntry,merge_defines,BindingExpandDesc};
			use render_derive::{BindLayout,BindingType,Input};
			
			
					#[derive(BindLayout, BindingType)]
					#[layout(set(2), binding(0))]
					#[sampler(Filtering)]
					pub struct SampBind;
				

					#[derive(BindLayout, BindingType)]
					#[layout(set(2), binding(1))]
					#[texture(dim(D2), multi(false), kind(Float))]
					pub struct Tex2DBind; // storagetexture: TODO
				
			
			
				#[derive(Input)]
				#[location(0)]
				pub struct PositionVert;
			

				#[derive(Input)]
				#[location(1)]
				pub struct UvVert;
			

				#[derive(Input)]
				#[location(2)]
				pub struct ColorVert;
			

			pub struct ProgramMeta;
			impl ShaderProgram for ProgramMeta {
				fn create_meta() -> pi_render::rhi::shader::ShaderMeta {
					let mut meta_ = ShaderMeta::default();
					meta_.name = std::any::type_name::<Self>().to_string();
					let _defines: &[Define] = &[];
					let meta = &mut meta_;
					let visibility = wgpu::ShaderStages::VERTEX;
					
					super::camera::push_meta(meta, visibility, &[]);
					let visibility = wgpu::ShaderStages::FRAGMENT;

					
					_meta.add_binding_entry(2, (SampBind::as_layout_entry(_visibility), BindingExpandDescList::new(vec![BindingExpandDesc::new_sampler("samp")], merge_defines(_defines, &[]))));
				

					_meta.add_binding_entry(2, (
							Tex2DBind::as_layout_entry(_visibility), 
							BindingExpandDescList::new(vec![BindingExpandDesc::new_texture("tex2d")], merge_defines(_defines, &[]))));
				
					super::sdf::push_meta(meta, visibility, &[]);
					let visibility = wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT;

					
					super::ui_meterial::push_meta(meta, visibility, &[]);
					push_vs_code(&mut meta.vs);
					push_fs_code(&mut meta.fs);
					meta.varyings = ShaderVarying(vec![
						
				InOut::new("vVertexPosition", "vec2", 0, vec![])
			,
				InOut::new("vUv", "vec2", 1, vec![])
			,
				InOut::new("vColor", "vec4", 2, vec![Define::new(true, VERTEX_COLOR_DEFINE.clone())])
			
					]);
					meta.ins = ShaderInput(vec![
						
				InOut::new("position", "vec2", 0, vec![])
			,
				InOut::new("uv", "vec2", 1, vec![])
			,
				InOut::new("color", "vec4", 2, vec![Define::new(true, VERTEX_COLOR_DEFINE.clone())])
			
					]);
					meta.outs = ShaderOutput(vec![
						
				InOut::new("o_Target", "vec4", 0, vec![])
			
					]);
					meta_
				}
			}
			fn push_vs_code(_codes: &mut BlockCodeAtom) {
				let _defines: &[Define] = &[];
				_codes.define.push(VS_CODE[0].clone().push_defines_front(_defines));
super::camera::push_code(codes, merge_defines(_defines, &[]).as_slice());
super::ui_meterial::push_code(codes, merge_defines(_defines, &[]).as_slice());
_codes.running.push(VS_CODE[1].clone().push_defines_front(_defines));
_codes.running.push(VS_CODE[2].clone().push_defines_front(_defines));
			}

			fn push_fs_code(_codes: &mut BlockCodeAtom) {
				let _defines: &[Define] = &[];
				_codes.define.push(FS_CODE[0].clone().push_defines_front(_defines));
super::ui_meterial::push_code(codes, merge_defines(_defines, &[]).as_slice());
super::sdf::push_code(codes, merge_defines(_defines, &[]).as_slice());
_codes.running.push(FS_CODE[1].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[2].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[3].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[4].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[5].clone().push_defines_front(_defines));
			}

			lazy_static! {
				static ref VS_CODE: Vec<CodeSlice> = vec![CodeSlice{code:pi_atom::Atom::from("#version 450
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	vVertexPosition = position;
	vec4 p = view * world * vec4(position.x, position.y, 1.0, 1.0);
	gl_Position = project * vec4(floor(p.x + 0.5 ), floor(p.y + 0.5), 1.0, 1.0);
	gl_Position.z = depth/60000.0;
	vUv = uv/texture_size;
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	vColor = color;
"), defines: vec![Define::new(true, VERTEX_COLOR_DEFINE.clone())]}];
				static ref FS_CODE: Vec<CodeSlice> = vec![CodeSlice{code:pi_atom::Atom::from("#version 450
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	vec4 c = color;
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("        c = vColor;
"), defines: vec![Define::new(true, VERTEX_COLOR_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	vec4 samp = texture(sampler2D(tex2d, samp), vUv);
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("		c.rgb = c.rgb * samp.g  + samp.r * strokeColorOrURect.rgb;
"), defines: vec![Define::new(true, STROKE_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	c.a = c.a * (1.0 - samp.b);
	o_Target = c;
"), defines: vec![]}];
				pub static ref VERTEX_COLOR_DEFINE: pi_atom::Atom = pi_atom::Atom::from("VERTEX_COLOR");
pub static ref STROKE_DEFINE: pi_atom::Atom = pi_atom::Atom::from("STROKE");
			}
		