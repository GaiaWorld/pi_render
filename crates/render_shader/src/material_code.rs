use pi_assets::asset::Asset;
use pi_atom::Atom;
use render_geometry::{vertex_data::{VertexBufferLayouts}, vertex_code::TVertexShaderCode};

use crate::{block_code::BlockCode, varying_code::{Varyings, VaryingCode}, vs_begin_code::{AttributesRef, VSBeginCode}, skin_code::ESkinCode, unifrom_code::{MaterialValueBindDesc, MaterialTextureBindDesc, TBindGroupToShaderCode, TValueBindToShaderCode}, scene_about_code::ERenderTag, shader::ResShader};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyMaterial(pub Atom);

/// 材质代码
/// 
#[derive(Debug, Clone)]
pub struct ResMaterailMeta {
    pub uniforms: MaterialValueBindDesc,
    pub textures: Option<MaterialTextureBindDesc>,
    attribute_ref: AttributesRef,
    varyings: Varyings,
    /// 顶点代码片段
    vs: BlockCode,
    /// 像素代码片段
    fs: BlockCode,
    pub size: usize,
}
impl ResMaterailMeta {
    pub fn new(
        uniforms: MaterialValueBindDesc,
        textures: Option<MaterialTextureBindDesc>,
        attribute_ref: AttributesRef,
        varyings: Varyings,
        vs: BlockCode,
        fs: BlockCode,
    ) -> Self {
        let size = attribute_ref.size() + varyings.size() + vs.size() + fs.size();
        Self { uniforms, textures, attribute_ref, varyings, vs, fs, size }
    }
    pub fn build(
        &self,
        vertex_layouts: &VertexBufferLayouts,
        skin: &ESkinCode,
        scene_about: &ERenderTag,
    ) -> (String, String) {
        let vs = self.vs_blocks(vertex_layouts, skin, scene_about);
        let fs = self.fs_blocks(scene_about);

        (
            ResShader::define_code(&vs) + ResShader::running_code(&vs).as_str(),
            ResShader::define_code(&fs) + ResShader::running_code(&fs).as_str(),
        )
    }
    pub fn vs_blocks(
        &self,
        vertex_layouts: &VertexBufferLayouts,
        skin: &ESkinCode,
        scene_about: &ERenderTag,
    ) -> Vec<BlockCode> {
        let mut result = vec![];

        // EntryPoint
        result.push(BlockCode {
            define: String::from("#version 450\r\n"),
            running: String::from("void main() {\r\n"),
        });
        
        // attributes
        result.push(BlockCode {
            define: vertex_layouts.vs_defines_code(),
            running: vertex_layouts.vs_running_code(),
        });

        // SceneAbout
        result.push(scene_about.vs_code());

        // skin
        result.push(BlockCode {
            define: String::from(""),
            running: skin.vs_begin_code(),
        });

        // attributes ref
        result.push(BlockCode {
            define: VaryingCode::vs_code(&self.varyings),
            running: VSBeginCode::code(&self.attribute_ref),
        });
        
        // uniform value
        result.push(BlockCode {
            define: self.uniforms.vs_code(),
            running: String::from(""),
        });
        
        // uniform tex
        if let Some(textures) = &self.textures {
            result.push(BlockCode {
                define: textures.vs_code(),
                running: String::from(""),
            });
        }
        
        // vertex
        result.push(self.vs.clone());
        
        // EntryPoint
        result.push(BlockCode {
            define: String::from(""),
            running: String::from("}\r\n"),
        });

        result
    }
    pub fn fs_blocks(
        &self,
        scene_about: &ERenderTag,
    ) -> Vec<BlockCode> {
        let mut result = vec![];

        // EntryPoint
        result.push(BlockCode {
            define: String::from("#version 450\r\n"),
            running: String::from("void main() {\r\n"),
        });

        // SceneAbout
        result.push(scene_about.fs_code());
        
        // uniform value
        result.push(BlockCode {
            define: self.uniforms.fs_code(),
            running: String::from(""),
        });
        
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
        result.push(self.fs.clone());
        
        // EntryPoint
        result.push(BlockCode {
            define: String::from(""),
            running: String::from("}\r\n"),
        });

        result
    }
}

impl Asset for ResMaterailMeta {
    type Key = KeyMaterial;

    fn size(&self) -> usize {
        self.size()
    }
}

#[cfg(test)]
mod test {
    use pi_atom::Atom;
    use render_geometry::vertex_data::{TVertexBufferDesc, VertexAttribute, EVertexDataKind, VertexBufferLayouts};

    use crate::{unifrom_code::{MaterialValueBindDesc, MaterialTextureBindDesc, UniformTextureDesc}, shader::ResShader, skin_code::ESkinCode, varying_code::Varying, vs_begin_code::AttributeRefCode, scene_about_code::ERenderTag};

    use super::ResMaterailMeta;


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

        let desc = ResMaterailMeta::new(
            MaterialValueBindDesc {
                set: 1,
                bind: 1,
                stage: wgpu::ShaderStages::VERTEX_FRAGMENT,
                mat4_list: vec![],
                mat2_list: vec![],
                vec4_list: vec![Atom::from("emissive")],
                vec2_list: vec![],
                float_list: vec![],
                int_list: vec![],
                uint_list: vec![],
            },
            None,
            crate::vs_begin_code::AttributesRef(
                vec![
                    AttributeRefCode { 
                        format: String::from("vec3"),
                        name: String::from("position"),
                        kind: Some(EVertexDataKind::Position),
                    },
                    AttributeRefCode { 
                        format: String::from("vec3"),
                        name: String::from("normal"),
                        kind: Some(EVertexDataKind::Normal),
                    },
                ]
            ),
            crate::varying_code::Varyings(
                vec![
                    Varying { 
                        format: String::from("vec3"),
                        name: String::from("v_normal"),
                    },
                    Varying { 
                        format: String::from("vec3"),
                        name: String::from("v_pos"),
                    },
                ]
            ),
            crate::block_code::BlockCode { 
                define: String::from(""), 
                running: String::from("
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
            crate::block_code::BlockCode { 
                define: String::from(""), 
                running: String::from("
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
            &&ESkinCode::None,
            &ERenderTag::MainCamera
        );
        let fs = desc.fs_blocks(
            &ERenderTag::MainCamera
        );
        let (vs, fs) = desc.build(&reslayouts, &ESkinCode::None, &ERenderTag::MainCamera);
        println!("{}", vs);
        println!("{}", fs);
    }
}