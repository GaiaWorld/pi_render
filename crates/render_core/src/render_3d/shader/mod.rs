use std::hash::Hash;

use pi_atom::Atom;

use crate::rhi::shader::BindingExpandDesc;

pub mod block_code;
pub mod instance_code;
pub mod skin_code;
pub mod varying_code;
pub mod vs_begin_code;
pub mod shader_defines;
pub mod uniform_sampler;
pub mod uniform_texture;
pub mod uniform_value;
pub mod shader;
pub mod shader_effect_meta;

pub type UniformPropertyName = Atom;

pub trait TUnifromShaderProperty {
    fn tag(&self) -> &UniformPropertyName;
}

impl TUnifromShaderProperty for BindingExpandDesc {
    fn tag(&self) -> &UniformPropertyName {
        &self.name
    }
}

pub trait TBindDescToShaderCode {
    fn vs_code(&self, set: u32, bind: u32) -> String;
    fn fs_code(&self, set: u32, bind: u32) -> String;
}

pub struct ShaderSetBind;
impl ShaderSetBind {
    pub const SET_SCENE_ABOUT: u32 = 0;
    pub const SET_MODEL_ABOUT: u32 = 1;
    pub const SET_EFFECT_ABOUT: u32 = 2;
    pub const SET_OTHER: u32 = 3;
    // pub const 
    pub fn code_uniform(kind: &str, name: &str) -> String {
        String::from(kind) + " " + name + ";\r\n"
    }
    pub fn code_uniform_array(kind: &str, name: &str, num: u32) -> String {
        String::from(kind) + " " + name + "[" + num.to_string().as_str() + "]" + ";\r\n"
    }
    pub fn code_set_bind_head(set: u32, bind: u32) -> String {
        let mut result = String::from("layout(set = ");
        result += set.to_string().as_str();
        result += ", binding = ";
        result += bind.to_string().as_str();
        result += ") uniform ";

        result
    }
    pub fn code_set_bind_readonly_buffer(set: u32, bind: u32) -> String {
        let mut result = String::from("layout(set = ");
        result += set.to_string().as_str();
        result += ", binding = ";
        result += bind.to_string().as_str();
        result += ") readonly buffer ";

        result
    }
    pub fn code_set_bind_texture2d(set: u32, bind: u32, name: &str) -> String {
        Self::code_set_bind_head(set, bind) + Self::code_uniform("texture2D", name).as_str()
    }
    pub fn code_set_bind_sampler(set: u32, bind: u32, tex_name: &str) -> String {
        let name = String::from("sampler") + tex_name;
        Self::code_set_bind_head(set, bind) + Self::code_uniform("sampler", &name).as_str()
    }
}