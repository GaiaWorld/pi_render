use std::sync::Arc;

use crate::{
    renderer::{
        bind_buffer::{BindBufferAllocator, BindBufferRange},
        shader::TShaderBindCode, buildin_var::ShaderVarUniform,
        bind::{TKeyBind, KeyBindLayoutBuffer, KeyBindBuffer},
        shader_stage::EShaderStage
    },
    render_3d::shader::ShaderSetBind
};



#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindModelAboutMatrix {
    pub(crate) data: BindBufferRange,
}
impl ShaderBindModelAboutMatrix {

    pub const OFFSET_WORLD_MATRIX:          wgpu::DynamicOffset = 0;
    pub const SIZE_WORLD_MATRIX:            wgpu::DynamicOffset = 16 * 4;
    pub const OFFSET_WORLD_MATRIX_INV:      wgpu::DynamicOffset = Self::OFFSET_WORLD_MATRIX + Self::SIZE_WORLD_MATRIX;
    pub const SIZE_WORLD_MATRIX_INV:        wgpu::DynamicOffset = 16 * 4;
    pub const OFFSET_VELOCITY:              wgpu::DynamicOffset = Self::OFFSET_WORLD_MATRIX_INV + Self::SIZE_WORLD_MATRIX_INV;
    pub const SIZE_VELOCITY:                wgpu::DynamicOffset = 4 * 4;
    pub const OFFSET_U32_A:                 wgpu::DynamicOffset = Self::OFFSET_VELOCITY + Self::SIZE_VELOCITY;
    pub const SIZE_U32_A:                   wgpu::DynamicOffset = 1 * 4;
    pub const OFFSET_U32_B:                 wgpu::DynamicOffset = Self::OFFSET_U32_A + Self::SIZE_U32_A;
    pub const SIZE_U32_B:                   wgpu::DynamicOffset = 1 * 4;
    pub const OFFSET_U32_C:                 wgpu::DynamicOffset = Self::OFFSET_U32_B + Self::SIZE_U32_B;
    pub const SIZE_U32_C:                   wgpu::DynamicOffset = 1 * 4;
    pub const OFFSET_U32_D:                 wgpu::DynamicOffset = Self::OFFSET_U32_C + Self::SIZE_U32_C;
    pub const SIZE_U32_D:                   wgpu::DynamicOffset = 1 * 4;

    pub const TOTAL_SIZE:                   wgpu::DynamicOffset = Self::OFFSET_U32_D + Self::SIZE_U32_D;

    pub fn new(
        allocator: &mut BindBufferAllocator,
    ) -> Option<Self> {
        if let Some(range) = allocator.allocate(ShaderBindModelAboutMatrix::TOTAL_SIZE) {
            Some(
                Self {
                    data: range,
                }
            )
        } else {
            None
        }
    }
    pub fn data(&self) -> &BindBufferRange {
        &self.data
    }
    pub fn key_layout(&self, binding: u16) -> KeyBindLayoutBuffer {
        KeyBindLayoutBuffer {
            binding,
            visibility: EShaderStage::VERTEXFRAGMENT,
            min_binding_size: self.data.size(),
        }
    }
    pub fn vs_define_code(set: u32, binding: u32) -> String {
        let mut result = String::from("");
        result += ShaderSetBind::code_set_bind_head(set, binding).as_str();
        result += " Model {\r\n";
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::_WORLD_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::_WORLD_MATRIX_INV).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::_VELOCITY).as_str();
        result += ShaderSetBind::code_uniform("uint", ShaderVarUniform::_SKIN_BONE_OFFSET0).as_str();
        result += ShaderSetBind::code_uniform("uint", ShaderVarUniform::_SKIN_BONE_OFFSET1).as_str();
        result += ShaderSetBind::code_uniform("uint", "placeholder_0").as_str();
        result += ShaderSetBind::code_uniform("uint", "placeholder_1").as_str();
        result += "};\r\n";
        result
    }

    pub fn fs_define_code(_: u32, _: u32) -> String {
        String::from("")
    }

}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseModelMatrix {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindModelAboutMatrix>,
}
impl BindUseModelMatrix {
    pub fn new(
        bind: u32,
        data: Arc<ShaderBindModelAboutMatrix>
    ) -> Self {
        Self { bind, data }
    }
}
impl TShaderBindCode for BindUseModelMatrix {
    fn vs_define_code(&self, set: u32) -> String {
        let mut result = String::from("");
        result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
        result += " Model {\r\n";
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::_WORLD_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::_WORLD_MATRIX_INV).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::_VELOCITY).as_str();
        result += ShaderSetBind::code_uniform("uint", ShaderVarUniform::_SKIN_BONE_OFFSET0).as_str();
        result += ShaderSetBind::code_uniform("uint", ShaderVarUniform::_SKIN_BONE_OFFSET1).as_str();
        result += ShaderSetBind::code_uniform("uint", "placeholder_0").as_str();
        result += ShaderSetBind::code_uniform("uint", "placeholder_1").as_str();
        result += "};\r\n";
        result
    }

    fn fs_define_code(&self, _: u32) -> String {
        String::from("")
    }

}
impl TKeyBind for BindUseModelMatrix {
    fn key_bind(&self) -> Option<crate::renderer::bind::EKeyBind> {
        Some(
            crate::renderer::bind::EKeyBind::Buffer(
                KeyBindBuffer {
                    data: self.data.data.clone(),
                    layout: Arc::new(
                        KeyBindLayoutBuffer {
                            binding: self.bind as u16,
                            visibility: EShaderStage::VERTEXFRAGMENT,
                            min_binding_size: self.data.data.size()
                        }
                    ) 
                }
            )
        )
    }
}