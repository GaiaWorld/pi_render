use std::sync::Arc;

use crate::{
    renderer::{
        bind_buffer::{BindBufferAllocator, BindBufferRange},
        shader::TShaderBindCode, buildin_var::ShaderVarUniform,
        bind::{TKeyBind, KeyBindLayoutBuffer, KeyBindBuffer},
        shader_stage::EShaderStage
    },
    render_3d::shader::*, rhi::device::RenderDevice
};


/// 数据从 Skeleton 创建, 以 Arc 数据拷贝到 ModelBind
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindModelAboutSkinValue {
    pub(crate) skin: ESkinCode,
    pub(crate) data: BindBufferRange,
}
impl ShaderBindModelAboutSkinValue {

    pub const OFFSET_BONE_TEX_SIZE:         wgpu::BufferAddress = 0;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 0 + 4 * 4;

    pub fn new(
        skin: &ESkinCode,
        _: &RenderDevice,
        allocator: &mut BindBufferAllocator,
        cache: Option<BindBufferRange>,
    ) -> Option<Self> {
        let size = match skin {
            ESkinCode::None => 0,
            ESkinCode::UBO(_, bones, cache) => bones.use_bytes() * (*cache as usize),
            ESkinCode::RowTexture(_) => ShaderBindModelAboutSkinValue::TOTAL_SIZE as usize,
            ESkinCode::FramesTexture(_) => ShaderBindModelAboutSkinValue::TOTAL_SIZE as usize,
        };

        if let Some(cache) = cache {
            Some(Self {
                skin: skin.clone(),
                data: cache,
            })
        } else {
            if size > 0 {
                if let Some(buffer) = allocator.allocate(size as u32) {
                    Some(Self {
                        skin: skin.clone(),
                        data: buffer,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
    pub fn key_layout(&self, binding: u16) -> KeyBindLayoutBuffer {
        KeyBindLayoutBuffer {
            binding,
            visibility: EShaderStage::VERTEX,
            min_binding_size: self.data.size(),
        }
    }
    pub fn data(&self) -> &BindBufferRange {
        &self.data
    }
    pub fn vs_define_code(skin: &ESkinCode, set: u32, binding: u32) -> String {
        let mut result = String::from("");
        match skin {
            ESkinCode::None => {},
            ESkinCode::UBO(_, bone, cache) => {
                result += ShaderSetBind::code_set_bind_head(set, binding).as_str();
                result += " Bone {\r\n";
                result += ShaderSetBind::code_uniform_array("mat4", ShaderVarUniform::BONE_MATRICES, bone.count() * (*cache as u32)).as_str();
                result += "};\r\n";

                result += skin.define_code().as_str();
            },
            _ => {
                result += ShaderSetBind::code_set_bind_head(set, binding).as_str();
                result += " Bone {\r\n";
                result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::BONE_TEX_SIZE).as_str();
                result += "};\r\n";

                result += skin.define_code().as_str();
            },
        }

        result
    }

    pub fn fs_define_code(_: u32, _: u32) -> String {
        String::from("")
    }
    // pub fn vs_running_code(skin: &ESkinCode) -> String {
    //     let mut result = String::from("");
    //     match skin {
    //         ESkinCode::None => {},
    //         ESkinCode::UBO(_, _) => {
    //             result += skin.running_code().as_str();
    //         },
    //         _ => {
    //             result += skin.running_code().as_str();
    //         },
    //     }

    //     result
    // }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseSkinValue {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindModelAboutSkinValue>,
}
impl BindUseSkinValue {
    pub fn new(
        bind: u32,
        data: Arc<ShaderBindModelAboutSkinValue>
    ) -> Self {
        Self { bind, data }
    }
    pub fn data(&self) -> &ShaderBindModelAboutSkinValue {
        &self.data
    }
    // pub fn vs_running_code(&self, _: u32) -> String {
    //     let mut result = String::from("");
    //     match self.data.skin {
    //         ESkinCode::None => {},
    //         ESkinCode::UBO(_, _) => {
    //             result += self.data.skin.running_code().as_str();
    //         },
    //         _ => {
    //             result += self.data.skin.running_code().as_str();
    //         },
    //     }

    //     result
    // }
}
impl TShaderBindCode for BindUseSkinValue {
    fn vs_define_code(&self, set: u32) -> String {
        let mut result = String::from("");
        match self.data.skin {
            ESkinCode::None => {},
            ESkinCode::UBO(_, bone, cache) => {
                result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
                result += " Bone {\r\n";
                result += ShaderSetBind::code_uniform_array("mat4", ShaderVarUniform::BONE_MATRICES, bone.count() * (cache as u32)).as_str();
                result += "};\r\n";

                result += self.data.skin.define_code().as_str();
            },
            _ => {
                result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
                result += " Bone {\r\n";
                result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::BONE_TEX_SIZE).as_str();
                result += "};\r\n";

                result += self.data.skin.define_code().as_str();
            },
        }

        result
    }

    fn fs_define_code(&self, _: u32) -> String {
        String::from("")
    }

}
impl TKeyBind for BindUseSkinValue {
    fn key_bind(&self) -> Option<crate::renderer::bind::EKeyBind> {
        match self.data.skin {
            ESkinCode::None => {
                None
            },
            _ => {
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
            },
        }
    }
}
