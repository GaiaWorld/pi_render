
			use pi_render::rhi::shader::{ShaderMeta, CodeSlice, BlockCodeAtom, InOut, Define,  ShaderVarying, ShaderInput, ShaderOutput, ShaderProgram, TypeKind,ArrayLen,AsLayoutEntry,BindingExpandDescList,merge_defines,TypeSize,BindingExpandDesc};
			use render_derive::{Uniform,BufferSize,Input,BindLayout,BindingType};
			
			
					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(1), binding(1))]
					#[min_size(64)]
					#[uniformbuffer]
					pub struct ColorMaterial1Bind; // storagebuffer: TODO
				

						#[derive(Uniform)]
						#[uniform(offset(0), len(64), bind(ColorMaterial1Bind))]
						pub struct World1Uniform<'a>(pub &'a[f32]);
					
			
					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(1), binding(0))]
					#[min_size(96)]
					#[uniformbuffer]
					pub struct ColorMaterialBind; // storagebuffer: TODO
				

						#[derive(Uniform)]
						#[uniform(offset(0), len(64), bind(ColorMaterialBind))]
						pub struct WorldUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(64), len(16), bind(ColorMaterialBind))]
						pub struct ColorUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(80), len(12), bind(ColorMaterialBind))]
						pub struct Patch10Uniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(92), len(4), bind(ColorMaterialBind))]
						pub struct DepthUniform<'a>(pub &'a[f32]);
					
			
				#[derive(Input)]
				#[location(0)]
				pub struct PositionVert;
			

				#[derive(Input)]
				#[location(1)]
				pub struct ColorVert;
			

			pub struct ProgramMeta;
			impl ShaderProgram for ProgramMeta {
				fn create_meta() -> pi_render::rhi::shader::ShaderMeta {
					let mut meta_ = ShaderMeta::default();
					meta_.name = std::any::type_name::<Self>().to_string();
					let _defines: &[Define] = &[];
					let meta = &mut meta_;
					let visibility = wgpu::ShaderStages::VERTEX;
					
					super::camera1::push_meta(meta, visibility, &[Define::new(true, BBB_DEFINE.clone())]);
					let visibility = wgpu::ShaderStages::FRAGMENT;

					
					_meta.add_binding_entry(1, (
						ColorMaterial1Bind::as_layout_entry(_visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("world1", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					], merge_defines(_defines, &[Define::new(true, FFF_DEFINE.clone())]))
					));
				
					
					let visibility = wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT;

					
					_meta.add_binding_entry(1, (
						ColorMaterialBind::as_layout_entry(_visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("world", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("color", &[1.,2.,3.,4.],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("PATCH__1_0", &[0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(3), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("depth", &[0.0],  TypeKind::Float, TypeSize::Scalar, ArrayLen::None)
					], merge_defines(_defines, &[Define::new(true, CCC_DEFINE.clone())]))
					));
				
					
					push_vs_code(&mut meta.vs);
					push_fs_code(&mut meta.fs);
					meta.varyings = ShaderVarying(vec![
						
				InOut::new("vVertexPosition", "vec2", 0, vec![])
			
					]);
					meta.ins = ShaderInput(vec![
						
				InOut::new("position", "vec2", 0, vec![])
			,
				InOut::new("color", "vec2", 1, vec![Define::new(true, AAA_DEFINE.clone())])
			
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
super::camera1::push_code(codes, merge_defines(_defines, &[Define::new(true, BBB_DEFINE.clone())]).as_slice());
_codes.running.push(VS_CODE[1].clone().push_defines_front(_defines));
_codes.running.push(VS_CODE[2].clone().push_defines_front(_defines));
_codes.define.push(VS_CODE[3].clone().push_defines_front(_defines));
			}

			fn push_fs_code(_codes: &mut BlockCodeAtom) {
				let _defines: &[Define] = &[];
				_codes.define.push(FS_CODE[0].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[1].clone().push_defines_front(_defines));
_codes.running.push(FS_CODE[2].clone().push_defines_front(_defines));
			}

			lazy_static! {
				static ref VS_CODE: Vec<CodeSlice> = vec![CodeSlice{code:pi_atom::Atom::from("#version 450
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	vVertexPosition=position;
"), defines: vec![Define::new(true, DD_DEFINE.clone())]},
CodeSlice{code:pi_atom::Atom::from("	gl_Position=project*view*world*vec4(position.x,position.y,1.,1.);
	gl_Position.z=depth/60000.+.2;
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("void main1(){

	gl_Position.z=depth/60000.+.2+bb;

}
"), defines: vec![]}];
				static ref FS_CODE: Vec<CodeSlice> = vec![CodeSlice{code:pi_atom::Atom::from("#version 450

precision highp float;
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	vec4 c=color.rgba;
"), defines: vec![]},
CodeSlice{code:pi_atom::Atom::from("	o_Target=c;
"), defines: vec![Define::new(true, GGG_DEFINE.clone())]}];
				pub static ref CCC_DEFINE: pi_atom::Atom = pi_atom::Atom::from("CCC");
pub static ref GGG_DEFINE: pi_atom::Atom = pi_atom::Atom::from("GGG");
pub static ref AAA_DEFINE: pi_atom::Atom = pi_atom::Atom::from("AAA");
pub static ref FFF_DEFINE: pi_atom::Atom = pi_atom::Atom::from("FFF");
pub static ref BBB_DEFINE: pi_atom::Atom = pi_atom::Atom::from("BBB");
pub static ref DD_DEFINE: pi_atom::Atom = pi_atom::Atom::from("DD");
			}
		