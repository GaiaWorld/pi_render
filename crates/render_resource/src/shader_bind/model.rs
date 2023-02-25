use std::{num::NonZeroU64, sync::Arc};

use pi_assets::mgr::AssetMgr;
use pi_share::Share;
use render_core::{rhi::device::RenderDevice, renderer::bind_buffer::{BindBufferRange, BindBufferAllocator, AssetBindBuffer}};
use render_shader::{skin_code::ESkinCode, set_bind::ShaderSetBind, buildin_var::ShaderVarUniform, shader::TShaderBindCode};

use crate::{buffer::dyn_mergy_buffer::{DynMergyBufferRange, DynMergyBufferAllocator}, bind_group::bind::TKeyBind};

use super::{TShaderBind, TRenderBindBufferData};


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindModelAboutMatrix {
    pub(crate) data: BindBufferRange,
}
impl ShaderBindModelAboutMatrix {

    pub const OFFSET_WORLD_MATRIX:          wgpu::DynamicOffset = 0;
    pub const OFFSET_WORLD_MATRIX_INV:      wgpu::DynamicOffset = 16 * 4;

    pub const TOTAL_SIZE:                   wgpu::DynamicOffset = 16 * 4 + 16 * 4;

    pub fn new(
        device: &RenderDevice,
        allocator: &mut BindBufferAllocator,
        asset_mgr: &Share<AssetMgr<AssetBindBuffer>>,
    ) -> Option<Self> {
        if let Some(range) = allocator.allocate(ShaderBindModelAboutMatrix::TOTAL_SIZE, asset_mgr) {
            Some(
                Self { data: range }
            )
        } else {
            None
        }
    }
    pub fn data(&self) -> &BindBufferRange {
        &self.data
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
impl TShaderBind for BindUseModelMatrix {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.bind,
                visibility: wgpu::ShaderStages ::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindModelAboutMatrix::TOTAL_SIZE as wgpu::BufferAddress) },
                count: None,
            }
        );
    }
    fn bind(&self) -> u32 {
        self.bind
    }
}
impl TKeyBind for BindUseModelMatrix {
    fn key_bind(&self) -> crate::bind_group::bind::KeyBind {
        crate::bind_group::bind::KeyBind::Buffer(
            crate::bind_group::bind::KeyBindBuffer {
                bind: self.bind,
                id_buffer: self.data.data.clone(),
                entry: wgpu::BindGroupLayoutEntry {
                    binding: self.bind,
                    visibility: wgpu::ShaderStages ::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindModelAboutMatrix::TOTAL_SIZE as wgpu::BufferAddress) },
                    count: None,
                },
            }
        )
    }
}
impl TRenderBindBufferData for BindUseModelMatrix {
    fn buffer(&self) -> &render_core::rhi::buffer::Buffer {
        self.data.data.buffer()
    }

    fn dyn_offset(&self) -> wgpu::DynamicOffset {
        self.data.data.offset()
    }
}

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
        device: &RenderDevice,
        allocator: &mut BindBufferAllocator,
        asset_mgr: &Share<AssetMgr<AssetBindBuffer>>,
    ) -> Option<Self> {
        let size = match skin {
            ESkinCode::None => 0,
            ESkinCode::UBO(_, bones) => bones.use_bytes(),
            ESkinCode::RowTexture(_) => ShaderBindModelAboutSkinValue::TOTAL_SIZE as usize,
            ESkinCode::FramesTexture(_) => ShaderBindModelAboutSkinValue::TOTAL_SIZE as usize,
        };

        if size > 0 {
            if let Some(buffer) = allocator.allocate(size as u32, asset_mgr) {
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
    pub fn data(&self) -> &BindBufferRange {
        &self.data
    }
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
    pub fn vs_running_code(&self, set: u32) -> String {
        let mut result = String::from("");
        match self.data.skin {
            ESkinCode::None => {},
            ESkinCode::UBO(_, bone) => {
                result += self.data.skin.running_code().as_str();
            },
            _ => {
                result += self.data.skin.running_code().as_str();
            },
        }

        result
    }
}
impl TShaderBindCode for BindUseSkinValue {
    fn vs_define_code(&self, set: u32) -> String {
        let mut result = String::from("");
        match self.data.skin {
            ESkinCode::None => {},
            ESkinCode::UBO(_, bone) => {
                result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
                result += " Bone {\r\n";
                result += ShaderSetBind::code_uniform_array("mat4", ShaderVarUniform::BONE_MATRICES, bone.count()).as_str();
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

    fn fs_define_code(&self, set: u32) -> String {
        String::from("")
    }

}
impl TShaderBind for BindUseSkinValue {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        match self.data.skin {
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
                        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindModelAboutSkinValue::TOTAL_SIZE) },
                        count: None,
                    }
                );
            },
            ESkinCode::FramesTexture(_) => {
                entries.push(
                        wgpu::BindGroupLayoutEntry {
                        binding: self.bind,
                        visibility: wgpu::ShaderStages ::VERTEX,
                        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindModelAboutSkinValue::TOTAL_SIZE) },
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
impl TKeyBind for BindUseSkinValue {
    fn key_bind(&self) -> crate::bind_group::bind::KeyBind {
        match self.data.skin {
            ESkinCode::None => {
                crate::bind_group::bind::KeyBind::Buffer(
                    crate::bind_group::bind::KeyBindBuffer {
                        bind: self.bind,
                        id_buffer: self.data.data.clone(),
                        entry: wgpu::BindGroupLayoutEntry {
                            binding: self.bind,
                            visibility: wgpu::ShaderStages ::VERTEX,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindModelAboutSkinValue::TOTAL_SIZE) },
                            count: None,
                        },
                    }
                )
            },
            ESkinCode::UBO(_, bones) => {
                crate::bind_group::bind::KeyBind::Buffer(
                    crate::bind_group::bind::KeyBindBuffer {
                        bind: self.bind,
                        id_buffer: self.data.data.clone(),
                        entry: wgpu::BindGroupLayoutEntry {
                            binding: self.bind,
                            visibility: wgpu::ShaderStages ::VERTEX,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(bones.use_bytes() as wgpu::BufferAddress) },
                            count: None,
                        }
                    }
                )
            },
            _ => {
                crate::bind_group::bind::KeyBind::Buffer(
                    crate::bind_group::bind::KeyBindBuffer {
                        bind: self.bind,
                        id_buffer: self.data.data.clone(),
                        entry: wgpu::BindGroupLayoutEntry {
                            binding: self.bind,
                            visibility: wgpu::ShaderStages ::VERTEX,
                            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindModelAboutSkinValue::TOTAL_SIZE) },
                            count: None,
                        },
                    }
                )
            },
        }
    }
}
impl TRenderBindBufferData for BindUseSkinValue {
    fn buffer(&self) -> &render_core::rhi::buffer::Buffer {
        self.data.data.buffer()
    }

    fn dyn_offset(&self) -> wgpu::DynamicOffset {
        self.data.data.offset() as wgpu::DynamicOffset
    }
}
