
				use pi_render::rhi::shader::{ ShaderMeta, CodeSlice, BlockCodeAtom, Define, BindingExpandDesc,BindingExpandDescList,AsLayoutEntry,merge_defines,TypeSize,ArrayLen,TypeKind};
				use render_derive::{Uniform,BufferSize,BindLayout,BindingType};
				
					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(0), binding(0))]
					#[min_size(144)]
					#[uniformbuffer]
					pub struct CameraMatrixBind; // storagebuffer: TODO
				

						#[derive(Uniform)]
						#[uniform(offset(0), len(64), bind(CameraMatrixBind))]
						pub struct ProjectUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(64), len(64), bind(CameraMatrixBind))]
						pub struct ViewUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(128), len(12), bind(CameraMatrixBind))]
						pub struct Patch00Uniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(140), len(4), bind(CameraMatrixBind))]
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
						#[uniform(offset(0), len(16), bind(Matrix2Bind))]
						pub struct DdddUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(16), len(16), bind(Matrix2Bind))]
						pub struct FfffUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(32), len(12), bind(Matrix2Bind))]
						pub struct EeeeUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(44), len(4), bind(Matrix2Bind))]
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
						#[uniform(offset(0), len(16), bind(Matrix1Bind))]
						pub struct AaaaUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(16), len(16), bind(Matrix1Bind))]
						pub struct CcccUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(32), len(8), bind(Matrix1Bind))]
						pub struct BbbbUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(40), len(8), bind(Matrix1Bind))]
						pub struct Patch30Uniform<'a>(pub &'a[f32]);
					
				pub fn push_meta(_meta: &mut ShaderMeta, _visibility: wgpu::ShaderStages, _defines: &[Define]) {
					
					_meta.add_binding_entry(0, (
						CameraMatrixBind::as_layout_entry(_visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("project", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("view", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("PATCH__0_0", &[0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(3), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("f32", &[0.0],  TypeKind::Float, TypeSize::Scalar, ArrayLen::None)
					], merge_defines(_defines, &[Define::new(true, EEE_DEFINE.clone())]))
					));
				

					_meta.add_binding_entry(2, (
							Tex2DBind::as_layout_entry(_visibility), 
							BindingExpandDescList::new(vec![BindingExpandDesc::new_texture("tex2d")], merge_defines(_defines, &[]))));
				

					_meta.add_binding_entry(3, (
						Matrix2Bind::as_layout_entry(_visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("dddd", &[0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("ffff", &[0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("eeee", &[0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(3), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("PATCH__3_1", &[0.0],  TypeKind::Float, TypeSize::Scalar, ArrayLen::None)
					], merge_defines(_defines, &[]))
					));
				

					_meta.add_binding_entry(3, (
						Arry1Bind::as_layout_entry(_visibility),
						BindingExpandDescList::new(vec![], merge_defines(_defines, &[]))
					));
				

					_meta.add_binding_entry(2, (SampBind::as_layout_entry(_visibility), BindingExpandDescList::new(vec![BindingExpandDesc::new_sampler("samp")], merge_defines(_defines, &[]))));
				

					_meta.add_binding_entry(3, (
						Matrix1Bind::as_layout_entry(_visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("aaaa", &[0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("cccc", &[0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("bbbb", &[0.0,0.0],  TypeKind::Float, TypeSize::Vec(2), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("PATCH__3_0", &[0.0,0.0],  TypeKind::Float, TypeSize::Vec(2), ArrayLen::None)
					], merge_defines(_defines, &[]))
					));
				
					
				}

				pub fn push_code(_codes: &mut BlockCodeAtom, _defines: &[Define]) {
					
				}

				lazy_static! {
					static ref CODE: Vec<CodeSlice> = vec![];
					pub static ref EEE_DEFINE: pi_atom::Atom = pi_atom::Atom::from("EEE");
				}
			