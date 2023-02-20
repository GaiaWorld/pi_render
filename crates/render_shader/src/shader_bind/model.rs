use std::num::NonZeroU64;

use crate::{skin_code::ESkinCode, set_bind::ShaderSetBind, buildin_var::ShaderVarUniform};

use super::TShaderBind;


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindModelAboutMatrix(pub u32);
impl ShaderBindModelAboutMatrix {

    pub const OFFSET_WORLD_MATRIX:          wgpu::BufferAddress = 0;
    pub const OFFSET_WORLD_MATRIX_INV:      wgpu::BufferAddress = 16 * 4;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 16 * 4 + 16 * 4;
}
impl TShaderBind for ShaderBindModelAboutMatrix {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.0,
                visibility: wgpu::ShaderStages ::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
                count: None,
            }
        );
    }
    fn bind(&self) -> u32 {
        self.0
    }
}

/// 数据从 Skeleton 创建, 以 Arc 数据拷贝到 ModelBind
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindModelAboutSkinValue {
    pub bind: u32,
    pub skin: ESkinCode,
}
impl ShaderBindModelAboutSkinValue {

    pub const OFFSET_BONE_TEX_SIZE:         wgpu::BufferAddress = 0;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 0 + 4 * 4;

    pub fn new(bind: u32, skin: &ESkinCode) -> Self {
        Self {
            bind,
            skin: skin.clone(),
        }
    }
    pub fn vs_define_code(&self, set: u32) -> String {
        let mut result = String::from("");
        match self.skin {
            ESkinCode::None => {},
            ESkinCode::UBO(_, bone) => {
                result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
                result += " Bone {\r\n";
                result += ShaderSetBind::code_uniform_array("mat4", ShaderVarUniform::BONE_MATRICES, bone.count()).as_str();
                result += "};\r\n";

                result += self.skin.define_code().as_str();
            },
            _ => {
                result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
                result += " Bone {\r\n";
                result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::BONE_TEX_SIZE).as_str();
                result += "};\r\n";

                result += self.skin.define_code().as_str();
            },
        }

        result
    }
    pub fn vs_running_code(&self, set: u32) -> String {
        let mut result = String::from("");
        match self.skin {
            ESkinCode::None => {},
            ESkinCode::UBO(_, bone) => {
                result += self.skin.running_code().as_str();
            },
            _ => {
                result += self.skin.running_code().as_str();
            },
        }

        result
    }
}
impl TShaderBind for ShaderBindModelAboutSkinValue {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        match self.skin {
            ESkinCode::None => {},
            ESkinCode::UBO(_, bones) => {
                entries.push(
                        wgpu::BindGroupLayoutEntry {
                        binding: self.bind,
                        visibility: wgpu::ShaderStages ::VERTEX,
                        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(bones.use_bytes() as wgpu::BufferAddress) },
                        count: None,
                    }
                );
            },
            ESkinCode::RowTexture(_) => {
                entries.push(
                        wgpu::BindGroupLayoutEntry {
                        binding: self.bind,
                        visibility: wgpu::ShaderStages ::VERTEX,
                        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
                        count: None,
                    }
                );
            },
            ESkinCode::FramesTexture(_) => {
                entries.push(
                        wgpu::BindGroupLayoutEntry {
                        binding: self.bind,
                        visibility: wgpu::ShaderStages ::VERTEX,
                        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
                        count: None,
                    }
                );
            },
        }
    }
    fn bind(&self) -> u32 {
        self.bind
    }
}
