
				use pi_render::rhi::shader::{ ShaderMeta, CodeSlice, BlockCodeAtom, Define, BindingExpandDesc,BindingExpandDescList,AsLayoutEntry,merge_defines,TypeSize,ArrayLen,TypeKind};
				use render_derive::{Uniform,BufferSize,BindLayout,BindingType};
				
					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(1), binding(0))]
					#[min_size(176)]
					#[uniformbuffer]
					pub struct GeoBind; // storagebuffer: TODO
				

						#[derive(Uniform)]
						#[uniform(offset(0), len(64), bind(GeoBind))]
						pub struct WorldUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(64), len(64), bind(GeoBind))]
						pub struct ClipSdfUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(128), len(16), bind(GeoBind))]
						pub struct ColorUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(144), len(16), bind(GeoBind))]
						pub struct StrokeColorOrURectUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(160), len(8), bind(GeoBind))]
						pub struct TextureSizeOrBottomLeftBorderUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(168), len(4), bind(GeoBind))]
						pub struct DepthUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(172), len(4), bind(GeoBind))]
						pub struct BlurUniform<'a>(pub &'a[f32]);
					
				pub fn push_meta(_meta: &mut ShaderMeta, _visibility: wgpu::ShaderStages, _defines: &[Define]) {
					
					_meta.add_binding_entry(1, (
						GeoBind::as_layout_entry(_visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("world", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("clipSdf", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("color", &[0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("strokeColorOrURect", &[0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Vec(4), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("textureSizeOrBottomLeftBorder", &[0.0,0.0],  TypeKind::Float, TypeSize::Vec(2), ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("depth", &[0.0],  TypeKind::Float, TypeSize::Scalar, ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("blur", &[0.0],  TypeKind::Float, TypeSize::Scalar, ArrayLen::None)
					], merge_defines(_defines, &[]))
					));
				
					
				}

				pub fn push_code(_codes: &mut BlockCodeAtom, _defines: &[Define]) {
					
				}

				lazy_static! {
					static ref CODE: Vec<CodeSlice> = vec![];
					
				}
			