use std::sync::Arc;

use pi_assets::asset::{Asset};
use pi_atom::Atom;
use wgpu::ShaderSource;

use crate::{
    renderer::{
        shader::{TShaderSetBlock, KeyShaderMeta},
        buildin_data::EDefaultTexture,
        shader_stage::EShaderStage,
        attributes::KeyShaderFromAttributes
    },
    rhi::device::RenderDevice
};

use super::{
    block_code::{BlockCode, BlockCodeAtom, TToBlockCodeAtom},
    varying_code::{VaryingCode, Varyings},
    shader_defines::ShaderDefinesSet,
    uniform_value::{MaterialValueBindDesc, UniformPropertyMat4, UniformPropertyMat2, UniformPropertyVec4, UniformPropertyVec2, UniformPropertyFloat, UniformPropertyInt, UniformPropertyUint}, 
    uniform_texture::{UniformTexture2DDesc, EffectUniformTexture2DDescs},
    instance_code::EInstanceCode, shader::{Shader3D, TShaderBlockCode}
};

/// 材质代码
/// 
#[derive(Debug, Clone)]
pub struct ShaderEffectMeta {
    pub uniforms: Arc<MaterialValueBindDesc>,
    pub textures: Arc<EffectUniformTexture2DDescs>,
    // pub samplers: Vec<UniformSamplerDesc>,
    pub varyings: Varyings,
    /// 顶点代码片段
    vs: BlockCodeAtom,
    /// 像素代码片段
    fs: BlockCodeAtom,
    pub size: usize,
    pub defines: ShaderDefinesSet,
}

impl From<(crate::rhi::shader::ShaderMeta, Vec<Atom>, Vec<Atom>)> for ShaderEffectMeta {
    fn from(
        value: (crate::rhi::shader::ShaderMeta, Vec<Atom>, Vec<Atom>),
    ) -> Self {
        let (value, vs_defines, fs_defines) = value;
        
        let mut uniforms: MaterialValueBindDesc = MaterialValueBindDesc::default();
        let mut textures: Vec<UniformTexture2DDesc> = vec![];
        // let mut samplers: Vec<Arc<UniformSamplerDesc>> = vec![];

        let len = value.bindings.buffer_uniform_expands.len();
        for index in 0..len {
            let bindinfo = value.bindings.buffer_uniform_expands.get(index);
            let layout = value.bindings.bind_group_entrys.get(index);

            if let (Some(layout), Some(bindinfo)) = (layout, bindinfo) {
                let len = layout.len();

                for j in 0..len {
                    let entry = layout.get(j);
                    let info = bindinfo.get(j);
                    if let (Some(entry), Some(info)) = (entry, info) {
                        match entry.ty {
                            wgpu::BindingType::Buffer { ty: _, has_dynamic_offset: _, min_binding_size: _ } => {
                                info.list.iter().for_each(|uniform| {
                                    if let Some(value) = &uniform.buffer_expand {
                                        match value.ty.ty {
                                            crate::rhi::shader::TypeKind::Float => {
                                                match value.ty.size {
                                                    crate::rhi::shader::TypeSize::Mat { rows, columns: _ } => {
                                                        if rows == 4 {
                                                            uniforms.mat4_list.push(UniformPropertyMat4(uniform.name.clone(), crate::render_3d::vec_u8_to_f32_16(&value.default_value)));
                                                        } else if rows == 2 {
                                                            uniforms.mat2_list.push(UniformPropertyMat2(uniform.name.clone(), crate::render_3d::vec_u8_to_f32_4(&value.default_value)));
                                                        }
                                                    },
                                                    crate::rhi::shader::TypeSize::Vec(v) => {
                                                        if v == 4 {
                                                            uniforms.vec4_list.push(UniformPropertyVec4(uniform.name.clone(), crate::render_3d::vec_u8_to_f32_4(&value.default_value)));
                                                        } else if v == 2 {
                                                            uniforms.vec2_list.push(UniformPropertyVec2(uniform.name.clone(), crate::render_3d::vec_u8_to_f32_2(&value.default_value)));
                                                        }
                                                    },
                                                    crate::rhi::shader::TypeSize::Scalar => {
                                                        uniforms.float_list.push(UniformPropertyFloat(uniform.name.clone(), crate::render_3d::vec_u8_to_f32(&value.default_value)));
                                                    },
                                                }
                                            },
                                            crate::rhi::shader::TypeKind::Sint => {
                                                uniforms.int_list.push(UniformPropertyInt(uniform.name.clone(), crate::render_3d::vec_u8_to_i32(&value.default_value)));
                                            },
                                            crate::rhi::shader::TypeKind::Uint => {
                                                uniforms.uint_list.push(UniformPropertyUint(uniform.name.clone(), crate::render_3d::vec_u8_to_u32(&value.default_value)));
                                            },
                                        }
                                    }
                                });
                            },
                            wgpu::BindingType::Sampler(_) => {
                                // let val = UniformSamplerDesc {
                                //     slotname: info.list.get(0).unwrap().name.clone(),
                                //     ty: val,
                                //     stage: entry.visibility,
                                // };
                                // samplers.push(val);
                            },
                            wgpu::BindingType::Texture { sample_type, view_dimension, multisampled } => {
                                match view_dimension {
                                    wgpu::TextureViewDimension::D1 => todo!(),
                                    wgpu::TextureViewDimension::D2 => {
                                        let val = UniformTexture2DDesc::new(
                                            info.list.get(0).unwrap().name.clone(),
                                            sample_type,
                                            multisampled,
                                            EShaderStage::new(entry.visibility),
                                            EDefaultTexture::White,
                                        );
                                        textures.push(val);
                                    },
                                    wgpu::TextureViewDimension::D2Array => todo!(),
                                    wgpu::TextureViewDimension::Cube => todo!(),
                                    wgpu::TextureViewDimension::CubeArray => todo!(),
                                    wgpu::TextureViewDimension::D3 => todo!(),
                                }
                            },
                            wgpu::BindingType::StorageTexture { access: _, format: _, view_dimension: _ } => {
                                
                            },
                        }
                    }
                }
            }
        }
        let defines = ShaderDefinesSet::from((&vs_defines, &fs_defines));
        let vs = value.vs.to_block_code();
        let fs = value.fs.to_block_code();
        let varyings = Varyings::from(&value.varyings);

        Self::new(uniforms, textures, varyings, vs, fs, defines)
    }
}
impl Asset for ShaderEffectMeta {
    type Key = KeyShaderMeta;
    fn size(&self) -> usize {
        self.size
    }
}
impl ShaderEffectMeta {
    pub fn new(
        mut uniforms: MaterialValueBindDesc,
        mut textures: Vec<UniformTexture2DDesc>,
        // samplers: Vec<UniformSamplerDesc>,
        varyings: Varyings,
        vs: BlockCodeAtom,
        fs: BlockCodeAtom,
        defines: ShaderDefinesSet,
    ) -> Self {
        let size = varyings.size() + vs.size() + fs.size();
        uniforms.sort();

        let mut arc_textures = vec![];
        textures.drain(..).for_each(|item| {
            arc_textures.push(Arc::new(item));
        });
        arc_textures.sort_by(|a, b| { a.slotname.cmp(&b.slotname) });

        Self {
            uniforms: Arc::new(uniforms),
            textures: Arc::new(EffectUniformTexture2DDescs::from(arc_textures)),
            // samplers,
            varyings,
            vs,
            fs,
            size,
            defines
        }
    }
    pub fn uniform_count(&self) -> usize {
        self.uniforms.mat4_list.len()
        + self.uniforms.mat2_list.len()
        + self.uniforms.vec4_list.len()
        + self.uniforms.vec2_list.len()
        + self.uniforms.float_list.len()
        + self.uniforms.int_list.len()
        + self.uniforms.uint_list.len()
    }
    pub fn vs_blocks<T0: TShaderSetBlock, T1: TShaderSetBlock, T2: TShaderSetBlock, T3: TShaderSetBlock>(
        &self,
        vertex_layouts: &KeyShaderFromAttributes,
        instance: &EInstanceCode,
        scene_about: &T0,
        model_about: &T1,
        // effect_value_about: &ShaderSetEffectValueAbout,
        set_2: Option<&T2>,
        set_3: Option<&T3>
    ) -> String {
        let mut result = vec![];

        // EntryPoint
        result.push(BlockCode {
            define: String::from("#version 450\r\n"),
            running: String::from("void main() {\r\nvec4 A_COLOR4 = vec4(1., 1., 1., 1.);\r\nvec3 A_NORMAL = vec3(0., 1., 0.);\r\n"),
        });
        
        // attributes
        result.push(BlockCode {
            define: vertex_layouts.vs_define_code(),
            running: vertex_layouts.vs_running_code(),
        });
        
        // attributes
        result.push(BlockCode {
            define: String::from(""),
            running: instance.vs_running_code(),
        });

        // SceneAbout
        result.push(BlockCode {
            define: scene_about.vs_define_code(),
            running: String::from(""),
        });

        // Model
        result.push(BlockCode {
            define: model_about.vs_define_code(),
            running: model_about.vs_running_code(),
        });
        
        // // uniform value
        // // if self.uniform_count() > 0 {
        //     result.push(BlockCode {
        //         define: self.uniforms.vs_code(effect_value_about.set, effect_value_about.bind_effect.bind()),
        //         running: String::from(""),
        //     });
        // // }

        // texture samplers
        if let Some(set_2) = set_2 {
            result.push(BlockCode {
                define: set_2.vs_define_code(),
                running: set_2.vs_running_code(),
            });
        }
        
        if let Some(set_3) = set_3 {
            result.push(BlockCode {
                define: set_3.vs_define_code(),
                running: set_3.vs_running_code(),
            });
        }

        // attributes ref
        result.push(BlockCode {
            define: VaryingCode::vs_code(&self.varyings),
            running: String::from(""),
        });
        
        // vertex
        result.push(self.vs.to_block_code());
        
        // EntryPoint
        result.push(BlockCode {
            define: String::from(""),
            running: String::from("\r\n}\r\n"),
        });

        Self::define_code(&result) + Self::running_code(&result).as_str()
    }
    pub fn fs_blocks<T0: TShaderSetBlock, T1: TShaderSetBlock, T2: TShaderSetBlock, T3: TShaderSetBlock>(
        &self,
        scene_about: &T0,
        model_about: &T1,
        set_2: Option<&T2>,
        set_3: Option<&T3>
    ) -> String {
        let mut result = vec![];

        // EntryPoint
        result.push(BlockCode {
            define: String::from("#version 450\r\n"),
            running: String::from("void main() {\r\n"),
        });

        // Model
        result.push(BlockCode {
            define: model_about.fs_define_code(),
            running: model_about.fs_running_code(),
        });
        
        // // uniform value
        // // if self.uniform_count() > 0 {
        //     result.push(BlockCode {
        //         define: self.uniforms.fs_code(effect_value_about.set, effect_value_about.bind_effect.bind()),
        //         running: String::from(""),
        //     });
        // // }

        // texture samplers
        if let Some(set_2) = set_2 {
            result.push(BlockCode {
                define: set_2.fs_define_code(),
                running: set_2.fs_running_code(),
            });
        }
        
        if let Some(set_3) = set_3 {
            result.push(BlockCode {
                define: set_3.fs_define_code(),
                running: set_3.fs_running_code(),
            });
        }

        // Varying
        result.push(BlockCode {
            define: VaryingCode::fs_code(&self.varyings),
            running: String::from(""),
        });

        // fragment
        result.push(self.fs.to_block_code());

        // SceneAbout
        result.push(BlockCode {
            define: scene_about.fs_define_code(),
            running: scene_about.fs_running_code(),
        });

        // EntryPoint
        result.push(BlockCode {
            define: String::from(""),
            running: String::from("\r\n}\r\n"),
        });
        
        Self::define_code(&result) + Self::running_code(&result).as_str()
    }

    pub fn build<T0: TShaderSetBlock, T1: TShaderSetBlock, T2: TShaderSetBlock>(
        &self,
        device: &RenderDevice,
        key_meta: &KeyShaderMeta,
        key_attributes: &KeyShaderFromAttributes,
        instance: &EInstanceCode,
        set_0: &T0,
        set_1: &T1,
        // effect_value: &ShaderSetEffectValueAbout,
        set_2: Option<&T2>,
        set_3: Option<&T2>
    ) -> Shader3D {

        let vs = self.vs_blocks(
            key_attributes,
            instance,
            set_0,
            set_1,
            // effect_value,
            set_2,
            set_3
        );
        let fs = self.fs_blocks(
            set_0,
            set_1,
            set_2,
            set_3
        );

        println!(">>>>>>>>>>>> Shader");

        let root_dir = std::env::current_dir().unwrap();
        let file_name = "temp.vert";
        std::fs::write(root_dir.join(file_name), vs.as_str());
        
        let file_name = "temp.frag";
        std::fs::write(root_dir.join(file_name), fs.as_str());

        let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some((key_meta.to_string() + "-VS").as_str()),
            source: ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(vs.as_str()),
                stage: naga::ShaderStage::Vertex,
                defines: naga::FastHashMap::default(),
            },
        });

        let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some((key_meta.to_string() + "-FS").as_str()),
            source: ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(fs.as_str()),
                stage: naga::ShaderStage::Fragment,
                defines: naga::FastHashMap::default(),
            },
        });

        Shader3D { vs, vs_point: "main", fs, fs_point: "main", p: std::marker::PhantomData  }
    }
    
    pub fn define_code(
        list: &Vec<BlockCode>,
    ) -> String {
        let mut result = String::from("");
        list.iter().for_each(|item| {
            result += item.define.as_str();
        });

        result
    }
    pub fn running_code(
        list: &Vec<BlockCode>,
    ) -> String {
        let mut result = String::from("");
        list.iter().for_each(|item| {
            result += item.running.as_str();
        });

        result
    }
}
