use pi_atom::Atom;
use render_resource::sampler::SamplerDesc;

use crate::{shader_data_kind::AsShaderDataKind, shader_set::ShaderSetEffectAbout, shader_bind::ShaderBindEffectValue};

pub enum ErrorUniformSlot {
    NotFoundProperty
}

pub enum UniformValueKind {
    Mat4,
    Mat2,
    Vec4,
    Vec2,
    Float,
    Int,
    Uint,
    TextureD1,
    TextureD2,
    TextureD3,
}

impl AsShaderDataKind for UniformValueKind {
    fn code_kind(&self) -> String {
        match self {
            UniformValueKind::Mat4              => String::from("mat4"),
            UniformValueKind::Mat2              => String::from("mat2"),
            UniformValueKind::Vec4              => String::from("vec4"),
            UniformValueKind::Vec2              => String::from("vec2"),
            UniformValueKind::Float             => String::from("float"),
            UniformValueKind::Int               => String::from("int"),
            UniformValueKind::Uint              => String::from("uint"),
            UniformValueKind::TextureD1         => String::from("texture2D"),
            UniformValueKind::TextureD2         => String::from("texture2D"),
            UniformValueKind::TextureD3         => String::from("textureCube"),
        }
    }
}

pub trait TUnifromShaderProperty {
    fn tag(&self) -> &UniformPropertyName;
}

pub type UniformPropertyName = Atom;


pub trait TTextureBindToShaderCode {
    fn vs_code(&self, set: u32, bind_off: u32, index: u32) -> String;
    fn fs_code(&self, set: u32, bind_off: u32, index: u32) -> String;
}
pub trait TValueBindToShaderCode {
    fn vs_code(&self, effect_about: &ShaderSetEffectAbout) -> String;
    fn fs_code(&self, effect_about: &ShaderSetEffectAbout) -> String;
}

#[derive(Debug, Clone)]
pub struct UniformTextureDesc {
    pub slotname: UniformPropertyName,
    pub sampler_binding_type: wgpu::SamplerBindingType,
    pub tex_sampler_type: wgpu::TextureSampleType,
    pub dimension: wgpu::TextureViewDimension,
    pub multisampled: bool,
    pub stage: wgpu::ShaderStages,
}
impl UniformTextureDesc {
    pub fn new2d(
        slotname: UniformPropertyName,
        stage: wgpu::ShaderStages,
    ) -> Self {
        Self {
            slotname,
            sampler_binding_type: wgpu::SamplerBindingType::Filtering,
            tex_sampler_type: wgpu::TextureSampleType::Float { filterable: true },
            dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
            stage,
        }
    }
    pub fn size(&self) -> usize {
        self.slotname.as_bytes().len() + 1 + 1 + 1 + 1
    }
    fn _code(&self, set: u32, bind_off: u32, index: u32) -> String {
        let mut result = String::from("");

        // layout(set = 2, binding = 0) uniform texture2D _MainTex;
        result += "layout(set = ";
        result += set.to_string().as_str();
        result += ", binding = ";
        result += (index * 2 + 0 + bind_off).to_string().as_str();
        result += ") uniform texture2D ";
        result += self.slotname.as_str();
        result += ";\r\n";
        // layout(set = 2, binding = 1) uniform sampler sampler_MainTex;
        result += "layout(set = ";
        result += set.to_string().as_str();
        result += ", binding = ";
        result += (index * 2 + 1 + bind_off).to_string().as_str();
        result += ") uniform sampler sampler";
        result += self.slotname.as_str();
        result += ";\r\n";

        result
    }
}

impl TTextureBindToShaderCode for UniformTextureDesc {
    fn vs_code(&self, set: u32, bind_off: u32, index: u32) -> String {
        if self.stage & wgpu::ShaderStages::VERTEX == wgpu::ShaderStages::VERTEX {
            self._code(set, bind_off, index)
        } else {
            String::from("")
        }
    }
    fn fs_code(&self, set: u32, bind_off: u32, index: u32) -> String {
        if self.stage & wgpu::ShaderStages::FRAGMENT == wgpu::ShaderStages::FRAGMENT {
            self._code(set, bind_off, index)
        } else {
            String::from("")
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialTextureBindDesc {
    pub list: Vec<UniformTextureDesc>,
}
impl MaterialTextureBindDesc {
    pub fn size(&self) -> usize {
        let mut size = 0;
        self.list.iter().for_each(|item| {
            size += item.size();
        });

        size
    }

    pub fn query_slot(&self, name: &UniformPropertyName) -> Result<usize, ErrorUniformSlot> {
        let mut index = usize::MAX;

        let mut i = 0;
        for item in self.list.iter() {
            if &item.slotname == name {
                index = i;
                break;
            }
            i += 1;
        }
        
        if index > self.list.len() {
            Err(ErrorUniformSlot::NotFoundProperty)
        } else {
            Ok(index)
        }
    }

    pub fn layout_entries(&self, effect_about: &ShaderSetEffectAbout, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        let mut i = 0;
        self.list.iter().for_each(|item| {
            entries.push(
                wgpu::BindGroupLayoutEntry {
                    binding: i * 2 + 0 + effect_about.tex_start_bind(),
                    visibility: item.stage,
                    ty: wgpu::BindingType::Texture {
                        sample_type: item.tex_sampler_type,
                        view_dimension: item.dimension,
                        multisampled: item.multisampled
                    },
                    count: None,
                }
            );
            
            entries.push(
                wgpu::BindGroupLayoutEntry {
                    binding: i * 2 + 1 + effect_about.tex_start_bind(),
                    visibility: item.stage,
                    ty: wgpu::BindingType::Sampler(item.sampler_binding_type),
                    count: None,
                }
            );

            i += 1;
        });
    }

    pub fn label(&self) -> String {
        let mut result = String::from("");
        
        self.list.iter().for_each(|item| {
            result += "#";
            result += item.slotname.as_str();
        });

        result
    }

    pub fn vs_code(&self, effect_about: &ShaderSetEffectAbout) -> String {
        let mut result = String::from("");

        let mut index = 0;
        self.list.iter().for_each(|item| {
            result += item.vs_code(effect_about.set(), effect_about.tex_start_bind(), index).as_str();

            index += 1;
        });

        result
    }

    pub fn fs_code(&self, effect_about: &ShaderSetEffectAbout) -> String {
        let mut result = String::from("");

        let mut index = 0;
        self.list.iter().for_each(|item| {
            result += item.fs_code(effect_about.set(), effect_about.tex_start_bind(), index).as_str();

            index += 1;
        });

        result
    }
}

#[derive(Debug, Clone)]
pub struct MaterialValueBindDesc<
    TMat4: TUnifromShaderProperty,
    TMat2: TUnifromShaderProperty,
    TVec4: TUnifromShaderProperty,
    TVec2: TUnifromShaderProperty,
    TFloat: TUnifromShaderProperty,
    TInt: TUnifromShaderProperty,
    TUint: TUnifromShaderProperty,
> {
    pub stage: wgpu::ShaderStages,
    pub mat4_list: Vec<TMat4>,
    pub mat2_list: Vec<TMat2>,
    pub vec4_list: Vec<TVec4>,
    pub vec2_list: Vec<TVec2>,
    pub float_list: Vec<TFloat>,
    pub int_list: Vec<TInt>,
    pub uint_list: Vec<TUint>,
}
impl<
    TMat4: TUnifromShaderProperty,
    TMat2: TUnifromShaderProperty,
    TVec4: TUnifromShaderProperty,
    TVec2: TUnifromShaderProperty,
    TFloat: TUnifromShaderProperty,
    TInt: TUnifromShaderProperty,
    TUint: TUnifromShaderProperty,
> MaterialValueBindDesc<
    TMat4,
    TMat2,
    TVec4,
    TVec2,
    TFloat,
    TInt,
    TUint,
> {
    pub fn none(stage: wgpu::ShaderStages) -> Self {
        Self { stage, mat4_list: vec![], mat2_list: vec![], vec4_list: vec![], vec2_list: vec![], float_list: vec![], int_list: vec![], uint_list: vec![] }
    }
    pub fn size(&self) -> usize {
        let mut size = 0;
        self.mat4_list.iter().for_each(|item| {
            size += item.tag().as_bytes().len();
        });
        
        self.mat2_list.iter().for_each(|item| {
            size += item.tag().as_bytes().len();
        });
        
        self.vec4_list.iter().for_each(|item| {
            size += item.tag().as_bytes().len();
        });
        
        self.vec2_list.iter().for_each(|item| {
            size += item.tag().as_bytes().len();
        });
        
        self.float_list.iter().for_each(|item| {
            size += item.tag().as_bytes().len();
        });
        
        self.int_list.iter().for_each(|item| {
            size += item.tag().as_bytes().len();
        });
        
        self.uint_list.iter().for_each(|item| {
            size += item.tag().as_bytes().len();
        });

        size
    }
    pub fn label(&self) -> String {
        let mut result = String::from("");

        self.mat4_list.iter().for_each(|name| {
            result += "#";
            result += name.tag().as_str();
        });
        
        self.mat2_list.iter().for_each(|name| {
            result += "#";
            result += name.tag().as_str();
        });

        self.vec4_list.iter().for_each(|name| {
            result += "#";
            result += name.tag().as_str();
        });

        self.vec2_list.iter().for_each(|name| {
            result += "#";
            result += name.tag().as_str();
        });

        self.float_list.iter().for_each(|name| {
            result += "#";
            result += name.tag().as_str();
        });

        self.uint_list.iter().for_each(|name| {
            result += "#";
            result += name.tag().as_str();
        });

        result
    }
    fn _code(&self, set: u32, index: u32) -> String {
        let mut result = String::from("");
        
        result += "layout(set = ";
        result += set.to_string().as_str();
        result += ", binding = ";
        result += index.to_string().as_str();
        result += ") uniform MatParam {\r\n";

        self.mat4_list.iter().for_each(|name| {
            result += "mat4 ";
            result += &name.tag();
            result += ";\r\n";
        });
        
        self.mat2_list.iter().for_each(|name| {
            result += "mat2 ";
            result += &name.tag();
            result += ";\r\n";
        });
        
        self.vec4_list.iter().for_each(|name| {
            result += "vec4 ";
            result += &name.tag();
            result += ";\r\n";
        });
        
        self.vec2_list.iter().for_each(|name| {
            result += "vec2 ";
            result += &name.tag();
            result += ";\r\n";
        });
        let fill_vec2_count    = self.vec2_list.len() % 2;
        for i in 0..fill_vec2_count {
            result += "vec2 _placeholder_vec2_";
            result += &i.to_string();
            result += ";\r\n";
        }
        
        self.float_list.iter().for_each(|name| {
            result += "float ";
            result += &name.tag();
            result += ";\r\n";
        });
        
        self.int_list.iter().for_each(|name| {
            result += "int ";
            result += &name.tag();
            result += ";\r\n";
        });
        
        self.uint_list.iter().for_each(|name| {
            result += "uint ";
            result += &name.tag();
            result += ";\r\n";
        });
        let fill_int_count    = (self.float_list.len() + self.int_list.len() + self.uint_list.len()) % 4;
        for i in 0..fill_int_count {
            result += "uint _placeholder_int_";
            result += &i.to_string();
            result += ";\r\n";
        }

        result += "};\r\n";

        result
    }
}
impl<
    TMat4: TUnifromShaderProperty,
    TMat2: TUnifromShaderProperty,
    TVec4: TUnifromShaderProperty,
    TVec2: TUnifromShaderProperty,
    TFloat: TUnifromShaderProperty,
    TInt: TUnifromShaderProperty,
    TUint: TUnifromShaderProperty,
> TValueBindToShaderCode for MaterialValueBindDesc<
    TMat4,
    TMat2,
    TVec4,
    TVec2,
    TFloat,
    TInt,
    TUint,
> {
    fn vs_code(&self, effect_about: &ShaderSetEffectAbout) -> String {
        if self.stage & wgpu::ShaderStages::VERTEX == wgpu::ShaderStages::VERTEX {
            self._code(effect_about.set(), ShaderBindEffectValue::BIND)
        } else {
            String::from("")
        }
    }

    fn fs_code(&self, effect_about: &ShaderSetEffectAbout) -> String {
        if self.stage & wgpu::ShaderStages::FRAGMENT == wgpu::ShaderStages::FRAGMENT {
            self._code(effect_about.set(), ShaderBindEffectValue::BIND)
        } else {
            String::from("")
        }
    }
}


#[cfg(test)]
mod test {
    use pi_atom::Atom;

    use crate::{unifrom_code::{TValueBindToShaderCode}, shader_set::ShaderSetEffectAbout};

    use super::{MaterialTextureBindDesc, UniformTextureDesc, MaterialValueBindDesc, UniformPropertyName, TUnifromShaderProperty};

    pub struct Uni(pub UniformPropertyName);
    impl TUnifromShaderProperty for Uni {
        fn tag(&self) -> &UniformPropertyName {
            &self.0
        }
    }

    #[test]
    fn uniform_code() {
        let texdesc = MaterialTextureBindDesc {
            list: vec![
                UniformTextureDesc { 
                    slotname: Atom::from("_EmissiveTex"), 
                    sampler_binding_type: wgpu::SamplerBindingType::Filtering, 
                    tex_sampler_type: wgpu::TextureSampleType::Float { filterable: true}, 
                    dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false, 
                    stage: wgpu::ShaderStages::FRAGMENT
                },
                UniformTextureDesc {
                    slotname: Atom::from("_MainTex"),
                    sampler_binding_type: wgpu::SamplerBindingType::Filtering, 
                    tex_sampler_type: wgpu::TextureSampleType::Float{ filterable: true}, 
                    dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false, 
                    stage: wgpu::ShaderStages::FRAGMENT
                },
                UniformTextureDesc {
                    slotname: Atom::from("_BoneTex"),
                    sampler_binding_type: wgpu::SamplerBindingType::Filtering, 
                    tex_sampler_type: wgpu::TextureSampleType::Float{ filterable: true}, 
                    dimension: wgpu::TextureViewDimension::D2, 
                    multisampled: false, 
                    stage: wgpu::ShaderStages::VERTEX
                },
            ],
        };

        let valuedesc = MaterialValueBindDesc::<Uni, Uni, Uni, Uni, Uni, Uni, Uni> {
            stage: wgpu::ShaderStages::VERTEX_FRAGMENT,
            mat4_list: vec![Uni(Atom::from("emissiveMatrics"))],
            mat2_list: vec![],
            vec4_list: vec![Uni(Atom::from("emissiveColor")), Uni(Atom::from("baseColor"))],
            vec2_list: vec![],
            float_list: vec![],
            int_list: vec![],
            uint_list: vec![],
        };

        let effect_about: ShaderSetEffectAbout = ShaderSetEffectAbout::new(Atom::from("Test"), 2, 16 * 4 + 8 * 4, 3);

        println!("texdesc.vs_code ");
        println!("{}", texdesc.vs_code(&effect_about));
        println!("texdesc.fs_code ");
        println!("{}", texdesc.fs_code(&effect_about));
        println!("valuedesc.vs_code ");
        println!("{}", valuedesc.vs_code(&effect_about));
        println!("valuedesc.fs_code ");
        println!("{}", valuedesc.fs_code(&effect_about));
    }
}