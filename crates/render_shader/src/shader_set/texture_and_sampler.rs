use std::sync::Arc;

use pi_atom::Atom;

use crate::{shader_bind::{ShaderBindTexture, ShaderBindSampler, TShaderBind, ShaderBindTextureWithSampler}, unifrom_code::{UniformTextureDesc, UniformSamplerDesc, UniformTextureWithSamplerUseinfo, UniformPropertyName}, skin_code::ESkinCode, shader::{TShaderCode, ShaderEffectMeta}};

pub trait TUniformSamplerDesc {
    fn uniform_sampler_desc(&self) -> Arc<UniformSamplerDesc>;
}
impl TUniformSamplerDesc for Arc<UniformSamplerDesc> {
    fn uniform_sampler_desc(&self) -> Arc<UniformSamplerDesc> {
        self.clone()
    }
}
// impl TUniformSamplerDesc for UniformTextureWithSamplerUseinfo {
//     fn uniform_sampler_desc(&self) -> Arc<UniformSamplerDesc> {
//         Arc::new(
//                 UniformSamplerDesc {
//                 slotname: UniformPropertyName::from(String::from("sampler") + self.texture_uniform.slotname.as_str()),
//                 ty: if self.param.filter { wgpu::SamplerBindingType::Filtering } else { wgpu::SamplerBindingType::NonFiltering },
//                 stage: self.texture_uniform.stage,
//             }
//         )
//     } 
// }

pub trait TUniformTextureDesc {
    fn uniform_texture_desc(&self) -> Arc<UniformTextureDesc>;
}
impl TUniformTextureDesc for Arc<UniformTextureDesc> {
    fn uniform_texture_desc(&self) -> Arc<UniformTextureDesc> {
        self.clone()
    }
}

pub struct EffectTextureDesc(pub Vec<()>);

/// * 纹理bindgroup
/// * 包含效果纹理 包含功能纹理
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderSetTextureAndSampler {
    pub set: u32,
    pub bind_tex_skin: Option<ShaderBindTextureWithSampler>,
    pub bind_tex_effc: Vec<ShaderBindTextureWithSampler>,
}
impl ShaderSetTextureAndSampler {
    pub fn new(
        set: u32,
        skin: &ESkinCode,
        effect_textures: &UniformTextureWithSamplerUseinfo,
    ) -> Self {
        let mut bind = 0;

        let mut bind_textures = vec![];
        effect_textures.0.iter().for_each(|useinfo| {
            let bind_texture = bind; bind += 1;
            let bind_sampler = bind; bind += 1;
            bind_textures.push(
                ShaderBindTextureWithSampler(
                    ShaderBindTexture::new(bind_texture, useinfo.1.clone()),
                    ShaderBindSampler::new(bind_sampler, useinfo.2.clone())
                )
            );
        });

        let bind_model_skin = match skin {
            ESkinCode::None => {
                None
            },
            ESkinCode::UBO(_, _) => {
                None
            },
            _ => {
                let bind_model_skin_tex = bind; bind += 1;
                let bind_model_skin_samp = bind; bind += 1;
                Some(
                    ShaderBindTextureWithSampler(
                        ShaderBindTexture::new(bind_model_skin_tex, ESkinCode::uniform_desc_tex()),
                        ShaderBindSampler::new(bind_model_skin_samp, ESkinCode::uniform_desc_sampler()),
                    )
                )
            }
        };

        Self {
            set,
            bind_tex_skin: bind_model_skin,
            bind_tex_effc: bind_textures,
        }
    }

    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut entries = vec![];

        self.bind_tex_effc.iter().for_each(|item| {
            item.0.layout_entry(&mut entries);
            item.1.layout_entry(&mut entries);
        });
        // self.bind_samplers.iter().for_each(|sampler| {
        //     sampler.layout_entry(&mut entries);
        // });

        if let Some(item) = &self.bind_tex_skin {
            item.0.layout_entry(&mut entries);
            item.1.layout_entry(&mut entries);
        }

        entries
    }
}
impl TShaderCode for ShaderSetTextureAndSampler {
    fn vs_define_code(&self) -> String {
        let mut result = String::from("");

        self.bind_tex_effc.iter().for_each(|item| {
            result += item.0.vs_define_code(self.set).as_str();
            result += item.1.vs_define_code(self.set).as_str();
        });

        if let Some(item) = &self.bind_tex_skin {
            result += item.0.vs_define_code(self.set).as_str();
            result += item.1.vs_define_code(self.set).as_str();
        }

        result
    }

    fn fs_define_code(&self) -> String {
        let mut result = String::from("");

        self.bind_tex_effc.iter().for_each(|item| {
            result += item.0.fs_define_code(self.set).as_str();
            result += item.1.fs_define_code(self.set).as_str();
        });

        if let Some(item) = &self.bind_tex_skin {
            result += item.0.fs_define_code(self.set).as_str();
            result += item.1.fs_define_code(self.set).as_str();
        }

        result
    }

    fn vs_running_code(&self) -> String {
        String::from("")
    }

    fn fs_running_code(&self) -> String {
        String::from("")
    }
}
