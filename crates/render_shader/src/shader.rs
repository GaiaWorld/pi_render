use std::{sync::Arc};

use pi_assets::asset::Asset;
use pi_atom::Atom;
use render_core::rhi::{device::RenderDevice, shader::ShaderMeta};

use crate::{
    block_code::{BlockCode, BlockCodeAtom, TToBlockCodeAtom},
    varying_code::{Varyings, VaryingCode},
    unifrom_code::{
        MaterialValueBindDesc,
        UniformTextureDesc,
        UniformSamplerDesc,
        UniformPropertyMat4,
        UniformPropertyMat2,
        UniformPropertyVec2,
        UniformPropertyVec4,
        UniformPropertyFloat,
        UniformPropertyUint,
        UniformPropertyInt,
        vec_u8_to_f32_16, vec_u8_to_f32_4, vec_u8_to_f32_2, vec_u8_to_f32, vec_u8_to_u32, vec_u8_to_i32,
        EffectUniformTextureDescs
    },
    instance_code::EInstanceCode,
    shader_set::{KeyShaderModelAbout, KeyShaderSceneAbout},
    attributes::{ShaderAttribute}, shader_defines::{KeyShaderDefines, ShaderDefinesSet}, buildin_data::EDefaultTexture
};

pub trait TShaderBindCode {
    fn vs_define_code(&self, set: u32) -> String;
    fn fs_define_code(&self, set: u32) -> String;
}

pub trait TShaderSetCode {
    fn vs_define_code(&self) -> String;
    fn fs_define_code(&self) -> String;
    fn vs_running_code(&self) -> String;
    fn fs_running_code(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyShaderEffect(pub Atom);

/// 材质代码
/// 
#[derive(Debug, Clone)]
pub struct ShaderEffectMeta {
    pub uniforms: Arc<MaterialValueBindDesc>,
    pub textures: Arc<EffectUniformTextureDescs>,
    // pub samplers: Vec<UniformSamplerDesc>,
    pub varyings: Varyings,
    /// 顶点代码片段
    vs: BlockCodeAtom,
    /// 像素代码片段
    fs: BlockCodeAtom,
    pub size: usize,
    pub defines: ShaderDefinesSet,
}

impl From<(ShaderMeta, Vec<Atom>, Vec<Atom>)> for ShaderEffectMeta {
    fn from(
        value: (ShaderMeta, Vec<Atom>, Vec<Atom>),
    ) -> Self {
        let (value, vs_defines, fs_defines) = value;
        
        let mut uniforms: MaterialValueBindDesc = MaterialValueBindDesc::default();
        let mut textures: Vec<UniformTextureDesc> = vec![];
        let mut samplers: Vec<Arc<UniformSamplerDesc>> = vec![];

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
                            wgpu::BindingType::Buffer { ty, has_dynamic_offset, min_binding_size } => {
                                info.list.iter().for_each(|uniform| {
                                    if let Some(value) = &uniform.buffer_expand {
                                        match value.ty.ty {
                                            render_core::rhi::shader::TypeKind::Float => {
                                                match value.ty.size {
                                                    render_core::rhi::shader::TypeSize::Mat { rows, columns } => {
                                                        if rows == 4 {
                                                            uniforms.mat4_list.push(UniformPropertyMat4(uniform.name.clone(), vec_u8_to_f32_16(&value.default_value)));
                                                        } else if rows == 2 {
                                                            uniforms.mat2_list.push(UniformPropertyMat2(uniform.name.clone(), vec_u8_to_f32_4(&value.default_value)));
                                                        }
                                                    },
                                                    render_core::rhi::shader::TypeSize::Vec(v) => {
                                                        if v == 4 {
                                                            uniforms.vec4_list.push(UniformPropertyVec4(uniform.name.clone(), vec_u8_to_f32_4(&value.default_value)));
                                                        } else if v == 2 {
                                                            uniforms.vec2_list.push(UniformPropertyVec2(uniform.name.clone(), vec_u8_to_f32_2(&value.default_value)));
                                                        }
                                                    },
                                                    render_core::rhi::shader::TypeSize::Scalar => {
                                                        uniforms.float_list.push(UniformPropertyFloat(uniform.name.clone(), vec_u8_to_f32(&value.default_value)));
                                                    },
                                                }
                                            },
                                            render_core::rhi::shader::TypeKind::Sint => {
                                                uniforms.int_list.push(UniformPropertyInt(uniform.name.clone(), vec_u8_to_i32(&value.default_value)));
                                            },
                                            render_core::rhi::shader::TypeKind::Uint => {
                                                uniforms.uint_list.push(UniformPropertyUint(uniform.name.clone(), vec_u8_to_u32(&value.default_value)));
                                            },
                                        }
                                    }
                                });
                            },
                            wgpu::BindingType::Sampler(val) => {
                                // let val = UniformSamplerDesc {
                                //     slotname: info.list.get(0).unwrap().name.clone(),
                                //     ty: val,
                                //     stage: entry.visibility,
                                // };
                                // samplers.push(val);
                            },
                            wgpu::BindingType::Texture { sample_type, view_dimension, multisampled } => {
                                let val = UniformTextureDesc::new(
                                    info.list.get(0).unwrap().name.clone(),
                                    sample_type,
                                    view_dimension,
                                    multisampled,
                                    entry.visibility,
                                    EDefaultTexture::White,
                                );
                                textures.push(val);
                            },
                            wgpu::BindingType::StorageTexture { access, format, view_dimension } => {
                                
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
        let size = varyings.size() + vs.size() + fs.size();

        Self::new(uniforms, textures, varyings, vs, fs, defines)
    }
}
impl Asset for ShaderEffectMeta {
    type Key = KeyShaderEffect;
    fn size(&self) -> usize {
        self.size
    }
}
impl ShaderEffectMeta {
    pub fn new(
        mut uniforms: MaterialValueBindDesc,
        mut textures: Vec<UniformTextureDesc>,
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
            textures: Arc::new(EffectUniformTextureDescs::from(arc_textures)),
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
    pub fn vs_blocks<T0: TShaderSetCode, T1: TShaderSetCode, T2: TShaderSetCode, T3: TShaderSetCode>(
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

        ResShader::define_code(&result) + ResShader::running_code(&result).as_str()
    }
    pub fn fs_blocks<T0: TShaderSetCode, T1: TShaderSetCode, T2: TShaderSetCode, T3: TShaderSetCode>(
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
        
        ResShader::define_code(&result) + ResShader::running_code(&result).as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyShaderFromAttributes(pub Vec<ShaderAttribute>);
impl TShaderSetCode for KeyShaderFromAttributes {
    fn vs_define_code(&self) -> String {
        let mut result = String::from("");
        self.0.iter().for_each(|attr| {
            result += attr.define_code().as_str();
        });

        result
    }

    fn fs_define_code(&self) -> String {
        String::from("")
    }

    fn vs_running_code(&self) -> String {
        let mut result = String::from("");
        self.0.iter().for_each(|attr| {
            result += attr.running_code().as_str();
        });

        result
    }

    fn fs_running_code(&self) -> String {
        String::from("")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyShaderFromEffect(pub KeyShaderEffect, pub KeyShaderSceneAbout, pub KeyShaderModelAbout);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyShader {
    pub key_attributes: KeyShaderFromAttributes,
    pub key_effect: KeyShaderFromEffect,
    pub defines_key: u128,
}

#[derive(Debug)]
pub struct ResShader {
    pub vs: wgpu::ShaderModule,
    pub vs_point: &'static str,
    pub fs: wgpu::ShaderModule,
    pub fs_point: &'static str,
}
impl ResShader {
    pub fn build<T0: TShaderSetCode, T1: TShaderSetCode, T2: TShaderSetCode>(
        device: &RenderDevice,
        preshaderkey: &KeyShaderEffect,
        preshader: &ShaderEffectMeta,
        key_shader_defines: KeyShaderDefines,
        key_attributes: &KeyShaderFromAttributes,
        instance: &EInstanceCode,
        set_0: &T0,
        set_1: &T1,
        // effect_value: &ShaderSetEffectValueAbout,
        set_2: Option<&T2>,
        set_3: Option<&T2>
    ) -> Self {
        let vs = preshader.vs_blocks(
            key_attributes,
            instance,
            set_0,
            set_1,
            // effect_value,
            set_2,
            set_3
        );
        let fs = preshader.fs_blocks(
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
            label: Some((preshaderkey.0.to_string() + "-VS").as_str()),
            source: wgpu::ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(vs.as_str()),
                stage: naga::ShaderStage::Vertex,
                defines: naga::FastHashMap::default(),
            },
        });

        let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some((preshaderkey.0.to_string() + "-FS").as_str()),
            source: wgpu::ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(fs.as_str()),
                stage: naga::ShaderStage::Fragment,
                defines: naga::FastHashMap::default(),
            },
        });

        Self { vs, vs_point: "main", fs, fs_point: "main" }
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

impl Asset for ResShader {
    type Key = KeyShader;

    fn size(&self) -> usize {
        10 * 1024
    }
}

#[cfg(test)]
mod test {
    use std::{ops::Range, sync::Arc};

    use pi_atom::Atom;

    use crate::{
        unifrom_code::{
            MaterialValueBindDesc, UniformPropertyName, TUnifromShaderProperty, UniformTextureDesc
        }, 
        shader::{ShaderEffectMeta, KeyShaderFromAttributes},
        varying_code::Varying,
        instance_code::EInstanceCode,
        attributes::{EVertexDataKind, ShaderAttribute},
        shader_defines::ShaderDefinesSet,
        buildin_data::EDefaultTexture
    };

    use super::TShaderSetCode;


    pub struct Uni(pub UniformPropertyName);
    impl TUnifromShaderProperty for Uni {
        fn tag(&self) -> &UniformPropertyName {
            &self.0
        }
    }

    pub struct TestSet(pub String);
    impl TShaderSetCode for TestSet {
        fn vs_define_code(&self) -> String {
            self.0.clone() + " vs_define_code \r\n"
        }

        fn fs_define_code(&self) -> String {
            self.0.clone() + " fs_define_code \r\n"
        }

        fn vs_running_code(&self) -> String {
            self.0.clone() + " vs_running_code \r\n"
        }

        fn fs_running_code(&self) -> String {
            self.0.clone() + " fs_running_code \r\n"
        }
    }

    #[test]
    fn material_test() {
        let meshdes = vec![
            ShaderAttribute { kind: EVertexDataKind::Position, location: 0 },
            ShaderAttribute { kind: EVertexDataKind::Normal, location: 1 },
        ];

        let textures: Vec<UniformTextureDesc> = vec![
            UniformTextureDesc::new(
                Atom::from("_BoneTex"), 
                wgpu::TextureSampleType::Float { filterable: true}, 
                wgpu::TextureViewDimension::D2, 
                false, 
                wgpu::ShaderStages::VERTEX,
                EDefaultTexture::White,
            ),
        ];

        let desc = ShaderEffectMeta::new(
            MaterialValueBindDesc {
                stage: wgpu::ShaderStages::VERTEX_FRAGMENT,
                mat4_list: vec![],
                mat2_list: vec![],
                vec4_list: vec![],
                // vec4_list: vec![UniformPropertyVec4(Atom::from("emissive"), [1.,1.,1.,1.])],
                vec2_list: vec![],
                float_list: vec![],
                int_list: vec![],
                uint_list: vec![],
            },
            textures,
            // vec![],
            crate::varying_code::Varyings(
                vec![
                    Varying { 
                        format: Atom::from("vec3"),
                        name: Atom::from("v_normal"),
                    },
                    Varying { 
                        format: Atom::from("vec3"),
                        name: Atom::from("v_pos"),
                    },
                ]
            ),
            crate::block_code::BlockCodeAtom { 
                define: Atom::from(""), 
                running: Atom::from("
vec3 position = A_POSITION;
vec3 normal = A_NORMAL;
mat4 finalWorld = PI_ObjectToWorld;

vec4 positionUpdate =  vec4(position, 1.);
vec4 worldPos =  finalWorld * positionUpdate;
// vec4 worldPos =  positionUpdate;

gl_Position = PI_MATRIX_VP * worldPos;
// gl_Position = positionUpdate;

v_pos = worldPos.xyz;

mat3 normalWorld = mat3(finalWorld);
v_normal = normal; // normalize(vec3(finalWorld * vec4(normal, 1.0)));
")
            },
            crate::block_code::BlockCodeAtom { 
                define: Atom::from(""), 
                running: Atom::from("
vec4 baseColor = vec4(1., 1., 1., 1.);

baseColor.rgb *= emissive.rgb * emissive.a;

float alpha = 1.0;

// float level = dot(v_normal, vec3(0., 0., -1.));
baseColor.rgb = mix(baseColor.rgb, v_normal, 0.5);
// baseColor.rgb = (v_pos + vec3(1., 1., 1.)) / 2.;

gl_FragColor = vec4(baseColor.rgb, alpha);
")
            },
            ShaderDefinesSet::default()
        );
        let useinfo = desc.textures.use_info(vec![]);

        let reslayouts = KeyShaderFromAttributes(meshdes);

        let vs = desc.vs_blocks(
            &reslayouts,
            &EInstanceCode(EInstanceCode::NONE),
            &TestSet(String::from("ShaderSetSceneAbout")),
            &TestSet(String::from("ShaderSetModel")),
            Some(&TestSet(String::from("ShaderSetTextureAndSampler"))),
            Some(&TestSet(String::from("T3"))),
        );
        let fs = desc.fs_blocks(
            &TestSet(String::from("ShaderSetSceneAbout")),
            &TestSet(String::from("ShaderSetModel")),
            Some(&TestSet(String::from("ShaderSetTextureAndSampler"))),
            Some(&TestSet(String::from("T3"))),
        );
        println!("{}", vs);
        println!("{}", fs);
    }
}