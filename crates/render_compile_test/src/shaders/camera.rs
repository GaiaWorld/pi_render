
				use pi_render::rhi::shader::{ ShaderMeta, CodeSlice, BlockCodeAtom, BindingExpandDesc, TypeKind, ArrayLen, TypeSize, AsLayoutEntry, Define, merge_defines, BindingExpandDescList};
				use render_derive::{BindLayout, BufferSize, Uniform, BindingType};
				use pi_atom::Atom;
				
					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(0), binding(0))]
					#[min_size(144)]
					#[uniformbuffer]
					pub struct CameraMatrixBind; // storagebuffer: TODO
				

						#[derive(Uniform)]
						#[span(offset(0), len(64))]
						pub struct ProjectUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(64), len(64))]
						pub struct ViewUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(128), len(12))]
						pub struct Patch00Uniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(140), len(4))]
						pub struct F32Uniform<'a>(pub &'a[f32]);
					

					#[derive(BindLayout, BindingType)]
					#[layout(set(2), binding(1))]
					#[texture(dim(D2), multi(false), kind(Float))]
					pub struct Tex2DBind; // storagetexture: TODO
				

					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(3), binding(1))]
					#[min_size(48)]
					#[uniformbuffer]
					pub struct Matrix2Bind; // storagebuffer: TODO
				

						#[derive(Uniform)]
						#[span(offset(0), len(16))]
						pub struct DdddUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(16), len(16))]
						pub struct FfffUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(32), len(12))]
						pub struct EeeeUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(44), len(4))]
						pub struct Patch31Uniform<'a>(pub &'a[f32]);
					

					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(3), binding(2),count(3))]
					#[min_size(0)]
					#[uniformbuffer]
					pub struct Arry1Bind; // storagebuffer: TODO
				

					#[derive(BindLayout, BindingType)]
					#[layout(set(2), binding(0))]
					#[sampler(Filtering)]
					pub struct SampBind;
				

					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(3), binding(0))]
					#[min_size(48)]
					#[uniformbuffer]
					pub struct Matrix1Bind; // storagebuffer: TODO
				

						#[derive(Uniform)]
						#[span(offset(0), len(16))]
						pub struct AaaaUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(16), len(16))]
						pub struct CcccUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(32), len(8))]
						pub struct BbbbUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[span(offset(40), len(8))]
						pub struct Patch30Uniform<'a>(pub &'a[f32]);
					
				pub fn push_meta(meta: &mut ShaderMeta, visibility: wgpu::ShaderStages, defines: &[Define]) {
					
					meta.add_binding_entry(0, (
						CameraMatrixBind::as_layout_entry(visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("project", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("view", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("PATCH__0_0", &[0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(3), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("f32", &[0.0],  TypeKind::Float, TypeSize::Scalar, ArrayLen::None)
					], merge_defines(defines, &[Define::new(true, DEFINE[0].clone())]))
					));
				

					meta.add_binding_entry(2, (
							Tex2DBind::as_layout_entry(visibility), 
							BindingExpandDescList::new(vec![BindingExpandDesc::new_texture("tex2d")], merge_defines(defines, &[]))));
				

					meta.add_binding_entry(3, (
						Matrix2Bind::as_layout_entry(visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("dddd", &[0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("ffff", &[0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("eeee", &[0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(3), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("PATCH__3_1", &[0.0],  TypeKind::Float, TypeSize::Scalar, ArrayLen::None)
					], merge_defines(defines, &[]))
					));
				

					meta.add_binding_entry(3, (
						Arry1Bind::as_layout_entry(visibility),
						BindingExpandDescList::new(vec![], merge_defines(defines, &[]))
					));
				

					meta.add_binding_entry(2, (SampBind::as_layout_entry(visibility), BindingExpandDescList::new(vec![BindingExpandDesc::new_sampler("samp")], merge_defines(defines, &[]))));
				

					meta.add_binding_entry(3, (
						Matrix1Bind::as_layout_entry(visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("aaaa", &[0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("cccc", &[0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("bbbb", &[0.0,0.0],  TypeKind::Float, TypeSize::Vec(2), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("PATCH__3_0", &[0.0,0.0],  TypeKind::Float, TypeSize::Vec(2), ArrayLen::None)
					], merge_defines(defines, &[]))
					));
				
					
				}

				pub fn push_code(codes: &mut BlockCodeAtom, defines: &[Define]) {
					
				}

				lazy_static! {
					static ref CODE: Vec<CodeSlice> = vec![];
					static ref DEFINE: Vec<Atom> = vec![Atom::from("EEE")];
				}
			