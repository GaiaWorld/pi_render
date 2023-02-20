use std::sync::Arc;

use crate::unifrom_code::{UniformSamplerDesc, TBindDescToShaderCode, UniformTextureDesc};

use super::TShaderBind;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindTextureWithSampler(pub ShaderBindTexture, pub ShaderBindSampler);
impl ShaderBindTextureWithSampler {
    pub fn vs_define_code(&self, set: u32) -> String {
        let mut result = self.0.desc.vs_code(set, self.0.bind);
        result += self.1.desc.vs_code(set, self.1.bind).as_str();

        result
    }
    pub fn fs_define_code(&self, set: u32) -> String {
        let mut result = self.0.desc.fs_code(set, self.0.bind);
        result += self.1.desc.fs_code(set, self.1.bind).as_str();

        result
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindTexture {
    pub bind: u32,
    pub desc: Arc<UniformTextureDesc>,
}
impl ShaderBindTexture {
    pub fn new(bind: u32, desc: Arc<UniformTextureDesc>) -> Self {
        Self { bind, desc }
    }
    pub fn vs_define_code(&self, set: u32) -> String {
        let mut result = self.desc.vs_code(set, self.bind);

        result
    }
    pub fn fs_define_code(&self, set: u32) -> String {
        let mut result = self.desc.fs_code(set, self.bind);

        result
    }
}
impl TShaderBind for ShaderBindTexture {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.bind,
                visibility: self.desc.stage,
                ty: wgpu::BindingType::Texture {
                    sample_type: self.desc.tex_sampler_type,
                    view_dimension: self.desc.dimension,
                    multisampled: self.desc.multisampled
                },
                count: None,
            }
        );
    }

    fn bind(&self) -> u32 {
        self.bind
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindSampler {
    pub bind: u32,
    pub desc: Arc<UniformSamplerDesc>,
}
impl ShaderBindSampler {
    pub fn new(bind: u32, desc: Arc<UniformSamplerDesc>) -> Self {
        Self { bind, desc }
    }
    pub fn vs_define_code(&self, set: u32) -> String {
        let mut result = self.desc.vs_code(set, self.bind);

        result
    }
    pub fn fs_define_code(&self, set: u32) -> String {
        let mut result = self.desc.fs_code(set, self.bind);

        result
    }
}
impl TShaderBind for ShaderBindSampler {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.bind,
                visibility: self.desc.stage,
                ty: wgpu::BindingType::Sampler(self.desc.ty),
                count: None,
            }
        );
    }

    fn bind(&self) -> u32 {
        self.bind
    }
}