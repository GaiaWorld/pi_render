
			use pi_render::rhi::shader::{BindingExpandDesc, TypeKind, TypeSize, ArrayLen, ShaderMeta, CodeSlice, BlockCodeAtom, InOut, AsLayoutEntry, Define, merge_defines, BindingExpandDescList,  ShaderVarying, ShaderInput, ShaderOutput};
			use render_derive::{BindLayout, BufferSize, Uniform, BindingType};
			use pi_atom::Atom;
			use pi_map::vecmap::VecMap;
			
			
					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(1), binding(1))]
					#[min_size(64)]
					#[uniformbuffer]
					pub struct ColorMaterial1Bind; // storagebuffer: TODO
				

						#[derive(Uniform)]
						#[span(offset(0), len(64))]
						pub struct World1Uniform<'a>(pub &'a[f32]);
					
			
					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(1), binding(0))]
					#[min_size(96)]
					#[uniformbuffer]
					pub struct ColorMaterialBind; // storagebuffer: TODO
				

						#[derive(Uniform)]
						#[span(offset(0), len(64))]
						pub struct WorldUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(64), len(16))]
						pub struct ColorUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(80), len(12))]
						pub struct Patch10Uniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(92), len(4))]
						pub struct DepthUniform<'a>(pub &'a[f32]);
					

			pub struct ProgramMeta;
			impl ProgramMeta {
				pub fn create_meta() -> pi_render::rhi::shader::ShaderMeta {
					let mut meta_ = ShaderMeta::default();
					let defines: &[Define] = &[];
					let meta = &mut meta_;
					let visibility = wgpu::ShaderStages::VERTEX;
					
					super::camera::push_meta(meta, visibility, &[Define::new(true, VS_DEFINE[1].clone())]);
					let visibility = wgpu::ShaderStages::FRAGMENT;

					
					meta.add_binding_entry(1, (
						ColorMaterial1Bind::as_layout_entry(visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("world1", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					], merge_defines(defines, &[Define::new(true, FS_DEFINE[0].clone())]))
					));
				
					
					let visibility = wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT;

					
					meta.add_binding_entry(1, (
						ColorMaterialBind::as_layout_entry(visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("world", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("color", &[1.,2.,3.,4.],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("PATCH__1_0", &[0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(3), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("depth", &[0.0],  TypeKind::Float, TypeSize::Scalar, ArrayLen::None)
					], merge_defines(defines, &[Define::new(true, VS_DEFINE[2].clone())]))
					));
				
					
					push_vs_code(&mut meta.vs);
					push_fs_code(&mut meta.fs);
					meta.varyings = ShaderVarying(vec![
						
				InOut::new("vVertexPosition", "vec2", 0, vec![])
			
					]);
					meta.ins = ShaderInput(vec![
						
				InOut::new("position", "vec2", 0, vec![])
			,
				InOut::new("color", "vec2", 1, vec![Define::new(true, VS_DEFINE[0].clone())])
			
					]);
					meta.outs = ShaderOutput(vec![
						
				InOut::new("o_Target", "vec4", 0, vec![])
			
					]);
					meta_
				}
			}
			fn push_vs_code(codes: &mut BlockCodeAtom) {
				let defines: &[Define] = &[];
				codes.define.push(VS_CODE[0].clone().push_defines_front(defines));
super::camera::push_code(codes, merge_defines(defines, &[Define::new(true, VS_DEFINE[1].clone())]).as_slice());
codes.running.push(VS_CODE[1].clone().push_defines_front(defines));
codes.running.push(VS_CODE[2].clone().push_defines_front(defines));
codes.define.push(VS_CODE[3].clone().push_defines_front(defines));
			}

			fn push_fs_code(codes: &mut BlockCodeAtom) {
				let defines: &[Define] = &[];
				codes.define.push(FS_CODE[0].clone().push_defines_front(defines));
codes.running.push(FS_CODE[1].clone().push_defines_front(defines));
codes.running.push(FS_CODE[2].clone().push_defines_front(defines));
			}

			lazy_static! {
				static ref VS_CODE: Vec<CodeSlice> = vec![CodeSlice{code:Atom::from("#version 450
"), defines: vec![]},
CodeSlice{code:Atom::from("	vVertexPosition=position;
"), defines: vec![Define::new(true, VS_DEFINE[3].clone())]},
CodeSlice{code:Atom::from("	gl_Position=project*view*world*vec4(position.x,position.y,1.,1.);
	gl_Position.z=depth/60000.+.2;
"), defines: vec![]},
CodeSlice{code:Atom::from("void main1(){

	gl_Position.z=depth/60000.+.2+bb;

}
"), defines: vec![]}];
				static ref FS_CODE: Vec<CodeSlice> = vec![CodeSlice{code:Atom::from("#version 450

precision highp float;
"), defines: vec![]},
CodeSlice{code:Atom::from("	vec4 c=color.rgba;
"), defines: vec![]},
CodeSlice{code:Atom::from("	o_Target=c;
"), defines: vec![Define::new(true, FS_DEFINE[1].clone())]}];
				static ref VS_DEFINE: Vec<Atom> = vec![Atom::from("AAA"),Atom::from("BBB"),Atom::from("CCC"),Atom::from("DD")];
				static ref FS_DEFINE: Vec<Atom> = vec![Atom::from("FFF"),Atom::from("GGG")];
			}
		