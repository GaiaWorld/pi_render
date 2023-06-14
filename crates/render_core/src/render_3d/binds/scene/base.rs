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
pub struct ShaderBindSceneAboutBase {
    pub(crate) data: BindBufferRange,
}
impl ShaderBindSceneAboutBase {

    pub const OFFSET_VIEW_MATRIX:           wgpu::DynamicOffset = 0;
    pub const SIZE_VIEW_MATRIX:             wgpu::DynamicOffset = 16 * 4;
    pub const OFFSET_PROJECT_MATRIX:        wgpu::DynamicOffset = Self::OFFSET_VIEW_MATRIX + Self::SIZE_VIEW_MATRIX;
    pub const SIZE_PROJECT_MATRIX:          wgpu::DynamicOffset = 16 * 4;
    pub const OFFSET_VIEW_PROJECT_MATRIX:   wgpu::DynamicOffset = Self::OFFSET_PROJECT_MATRIX + Self::SIZE_PROJECT_MATRIX;
    pub const SIZE_VIEW_PROJECT_MATRIX:     wgpu::DynamicOffset = 16 * 4;
    pub const OFFSET_CAMERA_POSITION:       wgpu::DynamicOffset = Self::OFFSET_VIEW_PROJECT_MATRIX + Self::SIZE_VIEW_PROJECT_MATRIX;
    pub const SIZE_CAMERA_POSITION:         wgpu::DynamicOffset = 4 * 4;
    pub const OFFSET_CAMERA_DIRECTION:      wgpu::DynamicOffset = Self::OFFSET_CAMERA_POSITION + Self::SIZE_CAMERA_POSITION;
    pub const SIZE_CAMERA_DIRECTION:        wgpu::DynamicOffset = 4 * 4;
    pub const OFFSET_CAMERA_ROTATION:       wgpu::DynamicOffset = Self::OFFSET_CAMERA_DIRECTION + Self::SIZE_CAMERA_DIRECTION;
    pub const SIZE_CAMERA_ROTATION:         wgpu::DynamicOffset = 16 * 4;
    
    pub const TOTAL_SIZE:                   wgpu::DynamicOffset = Self::OFFSET_CAMERA_ROTATION + Self::SIZE_CAMERA_ROTATION;
    pub fn new(
        allocator: &mut BindBufferAllocator,
    ) -> Option<Self> {
        if let Some(data) = allocator.allocate(Self::TOTAL_SIZE) {
            Some(Self { data })
        } else {
            None
        }
    }
    pub fn key_layout(&self, binding: u16) -> KeyBindLayoutBuffer {
        KeyBindLayoutBuffer {
            binding,
            visibility: EShaderStage::VERTEXFRAGMENT,
            min_binding_size: self.data.size(),
        }
    }
    pub fn data(&self) -> &BindBufferRange {
        &self.data
    }
    pub fn vs_define_code(&self, set: u32, bind: u32, ) -> String {
        let mut result = String::from("");
        result += ShaderSetBind::code_set_bind_head(set, bind).as_str();
        result += " Camera {\r\n";
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::VIEW_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::PROJECT_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::VIEW_PROJECT_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::CAMERA_POSITION).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::CAMERA_DIRECTION).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::VIEW_ROTATION_MATRIX_INV).as_str();
        result += "};\r\n";
        result
    }
    pub fn fs_define_code(&self, set: u32, bind: u32) -> String {
        self.vs_define_code(set, bind)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseSceneAboutCamera {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindSceneAboutBase>,
}
impl BindUseSceneAboutCamera {
    pub fn new(bind: u32, data: Arc<ShaderBindSceneAboutBase>) -> Self {
        Self { bind, data }
    }
}
impl TShaderBindCode for BindUseSceneAboutCamera {
    fn vs_define_code(&self, set: u32) -> String {
        self.data.vs_define_code(set, self.bind)
    }
    fn fs_define_code(&self, set: u32) -> String {
        self.vs_define_code(set)
    }
}
impl TKeyBind for BindUseSceneAboutCamera {
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
