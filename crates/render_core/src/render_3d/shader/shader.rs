use std::sync::Arc;

use pi_assets::asset::Asset;
use pi_atom::Atom;
use wgpu::ShaderSource;

use crate::{
    renderer::{
        shader::{TShaderSetBlock, TKeyShaderSetBlock, KeyShader, Shader, KeyShaderMeta},
        buildin_data::EDefaultTexture,
        shader_stage::EShaderStage, attributes::KeyShaderFromAttributes
    },
    rhi::device::RenderDevice, render_3d::bind_groups::{scene::KeyShaderSetScene, model::KeyShaderSetModel, texture_sampler::KeyShaderSetTextureSamplers}
};

use super::{
    block_code::{BlockCode, BlockCodeAtom, TToBlockCodeAtom},
    varying_code::{VaryingCode, Varyings},
    shader_defines::ShaderDefinesSet,
    uniform_value::{MaterialValueBindDesc, UniformPropertyMat4, UniformPropertyMat2, UniformPropertyVec4, UniformPropertyVec2, UniformPropertyFloat, UniformPropertyInt, UniformPropertyUint}, 
    uniform_texture::{UniformTexture2DDesc, EffectUniformTexture2DDescs, UniformSamplerDesc},
    instance_code::EInstanceCode
};


pub trait TShaderBlockCode {
    fn vs_define_code(&self) -> String;
    fn fs_define_code(&self) -> String;
    fn vs_running_code(&self) -> String;
    fn fs_running_code(&self) -> String;
}

impl TShaderBlockCode for KeyShaderFromAttributes {
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EKeyShader3DSetBlock {
    Scene(KeyShaderSetScene),
    Model(KeyShaderSetModel),
    TextureSampler(KeyShaderSetTextureSamplers),
}
impl TKeyShaderSetBlock for EKeyShader3DSetBlock {}

pub type KeyShader3D = KeyShader<4, EKeyShader3DSetBlock>;

pub type Shader3D = Shader<4, EKeyShader3DSetBlock>;

