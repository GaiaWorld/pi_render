
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
						
				InOut::new("vUv", "vec2", 0, vec![])
			,
				InOut::new("vVertexPosition", "vec2", 1, vec![])
			
					]);
					meta.ins = ShaderInput(vec![
						
				InOut::new("position", "vec2", 0, vec![])
			,
				InOut::new("uv", "vec2", 1, vec![])
			
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
_codes.running.push(FS_CODE[6].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[7].clone().push_defines_front(_defines));
			}

			lazy_static! {
				static ref VS_CODE: Vec<CodeSlice> = vec![CodeSlice{code:pi_atom::Atom::from("#version 450

#extension GL_OES_standard_derivatives : require

precision highp float;
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	vVertexPosition = position;
	gl_Position = project * view * world * vec4(position.x, position.y, 1.0, 1.0);
	gl_Position.z = depth/60000.0;
	vUv = uv;
"), defines: vec![]}];
				static ref FS_CODE: Vec<CodeSlice> = vec![CodeSlice{code:pi_atom::Atom::from("#version 450
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	vec4 color=texture(sampler2D(tex2d,samp),vUv);
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	color.a=color.a*calc_alpha(vVertexPosition);
"), defines: vec![Define::new(true, BORDER_RADIUS_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	color.a=color.a*calc_alpha(vVertexPosition);
"), defines: vec![Define::new(true, SECTOR_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	color.a=color.a*calc_alpha(vVertexPosition);
"), defines: vec![Define::new(true, RECT_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	color.a=color.a*calc_alpha(vVertexPosition);
"), defines: vec![Define::new(true, ELLIPSE_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	color.a=color.a*calc_alpha(vVertexPosition);
"), defines: vec![Define::new(true, CIRCLE_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	o_Target=vec4(color.rgb*color.a,color.a);
"), defines: vec![]}];
				pub static ref CIRCLE_DEFINE: pi_atom::Atom = pi_atom::Atom::from("CIRCLE");
pub static ref BORDER_RADIUS_DEFINE: pi_atom::Atom = pi_atom::Atom::from("BORDER_RADIUS");
pub static ref SECTOR_DEFINE: pi_atom::Atom = pi_atom::Atom::from("SECTOR");
pub static ref RECT_DEFINE: pi_atom::Atom = pi_atom::Atom::from("RECT");
pub static ref ELLIPSE_DEFINE: pi_atom::Atom = pi_atom::Atom::from("ELLIPSE");
			}
		