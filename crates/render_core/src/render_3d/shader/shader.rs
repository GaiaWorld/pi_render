
use crate::{
    renderer::{
        shader::{TKeyShaderSetBlock, KeyShader, Shader},
        attributes::KeyShaderFromAttributes
    },
    render_3d::bind_groups::{scene::KeyShaderSetScene, model::KeyShaderSetModel, texture_sampler::KeyShaderSetTextureSamplers}
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

