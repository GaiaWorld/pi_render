use std::sync::Arc;

use pi_assets::mgr::AssetMgr;
use pi_share::Share;

use crate::{
    renderer::{
        buffer::{RWBufferRange, AssetRWBuffer},
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
    pub const OFFSET_WORLD_MATRIX_INV:      wgpu::DynamicOffset = 16 * 4;

    pub const TOTAL_SIZE:                   wgpu::DynamicOffset = 16 * 4 + 16 * 4;

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
        result += "};\r\n";
        result
    }

    pub fn fs_define_code(set: u32, binding: u32) -> String {
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
        result += "};\r\n";
        result
    }

    fn fs_define_code(&self, set: u32) -> String {
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