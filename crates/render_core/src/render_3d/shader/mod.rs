
use pi_atom::Atom;

use crate::rhi::shader::BindingExpandDesc;

mod block_code;
mod instance_code;
mod skin_code;
mod varying_code;
mod vs_begin_code;
mod shader_defines;
mod uniform_sampler;
mod uniform_texture;
mod uniform_value;
mod shader;
mod shader_effect_meta;

pub use block_code::*;
pub use instance_code::*;
pub use skin_code::*;
pub use varying_code::*;
pub use vs_begin_code::*;
pub use shader_defines::*;
pub use uniform_sampler::*;
pub use uniform_texture::*;
pub use uniform_value::*;
pub use shader::*;
pub use shader_effect_meta::*;

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