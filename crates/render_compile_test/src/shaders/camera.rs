
				use pi_render::rhi::shader::{ ShaderMeta, CodeSlice, BlockCodeAtom, Define, BindingExpandDesc,BindingExpandDescList,AsLayoutEntry,merge_defines,TypeSize,ArrayLen,TypeKind};
				use render_derive::{Uniform,BufferSize,BindLayout,BindingType};
				
					#[derive(BindLayout, BufferSize, BindingType)]
					#[layout(set(0), binding(0))]
					#[min_size(128)]
					#[uniformbuffer]
					pub struct CameraBind; // storagebuffer: TODO
				

						#[derive(Uniform)]
						#[uniform(offset(0), len(64), bind(CameraBind))]
						pub struct ProjectUniform<'a>(pub &'a[f32]);
					

						#[derive(Uniform)]
						#[uniform(offset(64), len(64), bind(CameraBind))]
						pub struct ViewUniform<'a>(pub &'a[f32]);
					
				pub fn push_meta(_meta: &mut ShaderMeta, _visibility: wgpu::ShaderStages, _defines: &[Define]) {
					
					_meta.add_binding_entry(0, (
						CameraBind::as_layout_entry(_visibility),
						BindingExpandDescList::new(vec![
						BindingExpandDesc::new_buffer::<f32>("project", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					,
						BindingExpandDesc::new_buffer::<f32>("view", &[0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0],  TypeKind::Float, TypeSize::Mat{rows: 4, columns: 4}, ArrayLen::None)
					], merge_defines(_defines, &[]))
					));
				
					
				}

				pub fn push_code(_codes: &mut BlockCodeAtom, _defines: &[Define]) {
					
				}

				lazy_static! {
					static ref CODE: Vec<CodeSlice> = vec![];
					
				}
			