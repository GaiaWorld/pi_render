use std::ops::Deref;

use pi_assets::asset::Asset;
use pi_atom::Atom;
use pi_share::Share;
use render_core::rhi::device::RenderDevice;
use render_data_container::{UniformValueBindKey, vertex_layout_key::KeyVertexLayouts};
use render_geometry::{vertex_data::{VertexBufferLayouts}, vertex_code::TVertexShaderCode};

use crate::{block_code::{BlockCode, BlockCodeAtom}, varying_code::{Varyings, VaryingCode}, vs_begin_code::{AttributesRef, VSBeginCode}, skin_code::ESkinCode, unifrom_code::{MaterialValueBindDesc, MaterialTextureBindDesc, TBindGroupToShaderCode, TValueBindToShaderCode, TUnifromShaderProperty}, instance_code::EInstanceCode, shader_set::{ShaderSetSceneAbout, ShaderSetModelAbout}};


#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyShaderEffect(pub Atom);

/// 材质代码
/// 
#[derive(Debug, Clone)]
pub struct ShaderEffectMeta<
    TMat4: TUnifromShaderProperty,
    TMat2: TUnifromShaderProperty,
    TVec4: TUnifromShaderProperty,
    TVec2: TUnifromShaderProperty,
    TFloat: TUnifromShaderProperty,
    TInt: TUnifromShaderProperty,
    TUint: TUnifromShaderProperty,
> {
    pub uniforms: MaterialValueBindDesc<TMat4, TMat2, TVec4, TVec2, TFloat, TInt, TUint, >,
    pub textures: Option<MaterialTextureBindDesc>,
    pub varyings: Varyings,
    /// 顶点代码片段
    vs: BlockCodeAtom,
    /// 像素代码片段
    fs: BlockCodeAtom,
    pub size: usize,
}

impl<
    TMat4: TUnifromShaderProperty,
    TMat2: TUnifromShaderProperty,
    TVec4: TUnifromShaderProperty,
    TVec2: TUnifromShaderProperty,
    TFloat: TUnifromShaderProperty,
    TInt: TUnifromShaderProperty,
    TUint: TUnifromShaderProperty,
> ShaderEffectMeta<TMat4, TMat2, TVec4, TVec2, TFloat, TInt, TUint, > {
    pub fn new(
        uniforms: MaterialValueBindDesc<TMat4, TMat2, TVec4, TVec2, TFloat, TInt, TUint, >,
        textures: Option<MaterialTextureBindDesc>,
        varyings: Varyings,
        vs: BlockCodeAtom,
        fs: BlockCodeAtom,
    ) -> Self {
        let size = varyings.size() + vs.size() + fs.size();
        Self { uniforms, textures, varyings, vs, fs, size }
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
    pub fn vs_blocks(
        &self,
        vertex_layouts: &VertexBufferLayouts,
        instance: &EInstanceCode,
        scene_about: &ShaderSetSceneAbout,
        model_about: &ShaderSetModelAbout,
    ) -> String {
        let mut result = vec![];

        // EntryPoint
        result.push(BlockCode {
            define: String::from("#version 450\r\n"),
            running: String::from("void main() {\r\n vec4 A_COLOR4 = vec4(1., 1., 1., 1.);\r\n"),
        });
        
        // attributes
        result.push(BlockCode {
            define: vertex_layouts.vs_defines_code(),
            running: vertex_layouts.vs_running_code(),
        });
        
        // attributes
        result.push(BlockCode {
            define: String::from(""),
            running: instance.vs_running_code(),
        });

        // SceneAbout
        result.push(BlockCode {
            define: scene_about.define_code(),
            running: scene_about.vs_running_code(),
        });

        // Model
        result.push(BlockCode {
            define: model_about.define_code(),
            running: model_about.running_code(),
        });

        // attributes ref
        result.push(BlockCode {
            define: VaryingCode::vs_code(&self.varyings),
            running: String::from(""),
        });
        
        // uniform value
        if self.uniform_count() > 0 {
            result.push(BlockCode {
                define: self.uniforms.vs_code(),
                running: String::from(""),
            });
        }

        // uniform tex
        if let Some(textures) = &self.textures {
            result.push(BlockCode {
                define: textures.vs_code(),
                running: String::from(""),
            });
        }
        
        // vertex
        result.push(self.vs.to_block_code());
        
        // EntryPoint
        result.push(BlockCode {
            define: String::from(""),
            running: String::from("}\r\n"),
        });

        ResShader::define_code(&result) + ResShader::running_code(&result).as_str()
    }
    pub fn fs_blocks(
        &self,
        scene_about: &ShaderSetSceneAbout,
    ) -> String {
        let mut result = vec![];

        // EntryPoint
        result.push(BlockCode {
            define: String::from("#version 450\r\n"),
            running: String::from("void main() {\r\n"),
        });
        
        // uniform value
        if self.uniform_count() > 0 {
            result.push(BlockCode {
                define: self.uniforms.fs_code(),
                running: String::from(""),
            });
        }

        // uniform tex
        if let Some(textures) = &self.textures {
            result.push(BlockCode {
                define: textures.fs_code(),
                running: String::from(""),
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
            define: scene_about.define_code(),
            running: scene_about.fs_running_code(),
        });

        // EntryPoint
        result.push(BlockCode {
            define: String::from(""),
            running: String::from("}\r\n"),
        });
        
        ResShader::define_code(&result) + ResShader::running_code(&result).as_str()
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct KeyShader {
    pub shader: KeyShaderEffect,
    pub vs_layouts: KeyVertexLayouts,
    pub defines_key: u128,
    pub scene_about: ShaderSetSceneAbout,
    pub model_about: ShaderSetModelAbout,
}

#[derive(Debug)]
pub struct ResShader {
    pub vs: wgpu::ShaderModule,
    pub vs_point: &'static str,
    pub fs: wgpu::ShaderModule,
    pub fs_point: &'static str,
}
impl ResShader {
    pub fn build<
        TMat4: TUnifromShaderProperty,
        TMat2: TUnifromShaderProperty,
        TVec4: TUnifromShaderProperty,
        TVec2: TUnifromShaderProperty,
        TFloat: TUnifromShaderProperty,
        TInt: TUnifromShaderProperty,
        TUint: TUnifromShaderProperty,
    >(
        device: &RenderDevice,
        preshaderkey: &KeyShaderEffect,
        preshader: &ShaderEffectMeta<TMat4, TMat2, TVec4, TVec2, TFloat, TInt, TUint, >,
        vertex_layouts: &VertexBufferLayouts,
        instance: &EInstanceCode,
        scene_about: &ShaderSetSceneAbout,
        model_about: &ShaderSetModelAbout,
    ) -> Self {
        let vs = preshader.vs_blocks(vertex_layouts, instance, scene_about, model_about);
        let fs = preshader.fs_blocks(scene_about);

        println!("{}", vs);
        println!("{}", fs);

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
    use pi_atom::Atom;
    use render_geometry::vertex_data::{TVertexBufferDesc, VertexAttribute, EVertexDataKind, VertexBufferLayouts};

    use crate::{unifrom_code::{MaterialValueBindDesc, MaterialTextureBindDesc, UniformTextureDesc, UniformPropertyName, TUnifromShaderProperty}, shader::{ResShader, ShaderEffectMeta}, skin_code::ESkinCode, varying_code::Varying, vs_begin_code::AttributeRefCode, instance_code::EInstanceCode, shader_set::{ShaderSetSceneAbout, ShaderSetModelAbout}};


    #[derive(Debug)]
    pub struct TestVertexBufferDesc {
        pub attributes: Vec<VertexAttribute>,
        pub step_mode: wgpu::VertexStepMode,
    }
    impl TVertexBufferDesc for TestVertexBufferDesc {
        fn attributes(&self) -> &Vec<VertexAttribute> {
            &self.attributes
        }

        fn step_mode(&self) -> wgpu::VertexStepMode {
            self.step_mode
        }
    }

    pub struct Uni(pub UniformPropertyName);
    impl TUnifromShaderProperty for Uni {
        fn tag(&self) -> &UniformPropertyName {
            &self.0
        }
    }

    #[test]
    fn material_test() {
        let meshdes = vec![
            TestVertexBufferDesc {
                attributes: vec![
                    VertexAttribute { kind: EVertexDataKind::Position, format: wgpu::VertexFormat::Float32x3 },
                ],
                step_mode: wgpu::VertexStepMode::Vertex,
            },
            TestVertexBufferDesc { 
                attributes: vec![
                    VertexAttribute { kind: EVertexDataKind::Normal, format: wgpu::VertexFormat::Float32x3 }
                ],
                step_mode: wgpu::VertexStepMode::Vertex,
            }
        ];

        let desc = ShaderEffectMeta::<Uni, Uni, Uni, Uni, Uni, Uni, Uni>::new(
            MaterialValueBindDesc {
                set: 1,
                bind: 1,
                stage: wgpu::ShaderStages::VERTEX_FRAGMENT,
                mat4_list: vec![],
                mat2_list: vec![],
                vec4_list: vec![Uni(Atom::from("emissive"))],
                vec2_list: vec![],
                float_list: vec![],
                int_list: vec![],
                uint_list: vec![],
            },
            None,
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
        );

        let reslayouts = VertexBufferLayouts::from(&meshdes);

        let vs = desc.vs_blocks(
            &reslayouts,
            &EInstanceCode(EInstanceCode::NONE),
            &ShaderSetSceneAbout::new(0, false, false, false, false),
            &ShaderSetModelAbout::new(2, None)
        );
        let fs = desc.fs_blocks(
            &ShaderSetSceneAbout::new(0, false, false, false, false),
        );
        println!("{}", vs);
        println!("{}", fs);
    }
}