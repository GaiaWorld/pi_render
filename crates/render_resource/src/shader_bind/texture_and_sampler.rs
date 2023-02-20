use std::sync::Arc;

use pi_atom::Atom;
use render_shader::{unifrom_code::{UniformSamplerDesc, TBindDescToShaderCode, UniformTextureDesc, EffectUniformTextureWithSamplerUseinfo}, shader::{TShaderBindCode, ShaderEffectMeta}, texture_sampler_code::SamplerDesc};

use crate::bind_group::bind::TKeyBind;

use super::TShaderBind;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseTextureWithSampler(pub BindUseTexture, pub BindUseSampler);
impl BindUseTextureWithSampler {
    pub fn vs_define_code(&self, set: u32) -> String {
        let mut result = self.0.vs_define_code(set);
        result += self.1.vs_define_code(set).as_str();

        result
    }
    pub fn fs_define_code(&self, set: u32) -> String {
        let mut result = self.0.fs_define_code(set);
        result += self.1.fs_define_code(set).as_str();

        result
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EffectTextureAndSamplerBinds {
    pub list: Vec<(Arc<ShaderBindTexture>, Arc<ShaderBindSampler>)>,
}
impl EffectTextureAndSamplerBinds {
    pub fn new(
        effect_textures: &EffectUniformTextureWithSamplerUseinfo,
    ) -> Self {
        let mut list = vec![];
        effect_textures.0.iter().for_each(|item| {
            list.push((
                Arc::new(ShaderBindTexture::new(item.0.url.clone(), item.1.clone())),
                Arc::new(ShaderBindSampler::new(item.0.sample.clone(), item.2.clone())),
            ));
        });

        Self {
            list
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindTexture {
    pub(crate) data: Atom,
    pub(crate) desc: Arc<UniformTextureDesc>,
}
impl ShaderBindTexture {
    pub fn new(data: Atom, desc: Arc<UniformTextureDesc>) -> Self {
        Self { data, desc }
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseTexture {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindTexture>,
}
impl TShaderBindCode for BindUseTexture {
    fn vs_define_code(&self, set: u32) -> String {
        self.data.desc.vs_code(set, self.bind)
    }
    fn fs_define_code(&self, set: u32) -> String {
        self.data.desc.fs_code(set, self.bind)
    }
}
impl TShaderBind for BindUseTexture {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.bind,
                visibility: self.data.desc.stage,
                ty: wgpu::BindingType::Texture {
                    sample_type: self.data.desc.tex_sampler_type,
                    view_dimension: self.data.desc.dimension,
                    multisampled: self.data.desc.multisampled
                },
                count: None,
            }
        );
    }

    fn bind(&self) -> u32 {
        self.bind
    }
}
impl TKeyBind for BindUseTexture {
    fn key_bind(&self) -> crate::bind_group::bind::KeyBind {
        crate::bind_group::bind::KeyBind::Texture(
            crate::bind_group::bind::KeyBindTexture {
                bind: self.bind,
                id_texture: self.data.data.clone(),
                entry: wgpu::BindGroupLayoutEntry {
                    binding: self.bind,
                    visibility: self.data.desc.stage,
                    ty: wgpu::BindingType::Texture {
                        sample_type: self.data.desc.tex_sampler_type,
                        view_dimension: self.data.desc.dimension,
                        multisampled: self.data.desc.multisampled
                    },
                    count: None,
                },
            }
        )
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindSampler {
    pub(crate) data: SamplerDesc,
    pub(crate) desc: Arc<UniformSamplerDesc>,
}
impl ShaderBindSampler {
    pub fn new(data: SamplerDesc, desc: Arc<UniformSamplerDesc>) -> Self {
        Self { data, desc }
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseSampler {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindSampler>,
}
impl TShaderBindCode for BindUseSampler {
    fn vs_define_code(&self, set: u32) -> String {
        self.data.desc.vs_code(set, self.bind)
    }
    fn fs_define_code(&self, set: u32) -> String {
        self.data.desc.fs_code(set, self.bind)
    }
}
impl TShaderBind for BindUseSampler {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.bind,
                visibility: self.data.desc.stage,
                ty: wgpu::BindingType::Sampler(self.data.desc.ty),
                count: None,
            }
        );
    }

    fn bind(&self) -> u32 {
        self.bind
    }
}
impl TKeyBind for BindUseSampler {
    fn key_bind(&self) -> crate::bind_group::bind::KeyBind {
        crate::bind_group::bind::KeyBind::Sampler(
            crate::bind_group::bind::KeyBindSampler {
                bind: self.bind,
                id_sampler: self.data.data.clone(),
                entry: wgpu::BindGroupLayoutEntry {
                    binding: self.bind,
                    visibility: self.data.desc.stage,
                    ty: wgpu::BindingType::Sampler(self.data.desc.ty),
                    count: None,
                },
            }
        )
    }
}