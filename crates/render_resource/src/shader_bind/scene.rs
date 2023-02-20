use std::{num::NonZeroU64, sync::Arc};

use render_shader::{set_bind::ShaderSetBind, buildin_var::ShaderVarUniform, shader::TShaderBindCode};

use crate::{buffer::dyn_mergy_buffer::DynMergyBufferRange, bind_group::bind::TKeyBind};

use super::{TShaderBind, TRenderBindBufferData};


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindSceneAboutCamera {
    pub(crate) data: DynMergyBufferRange,
}
impl ShaderBindSceneAboutCamera {

    pub const OFFSET_VIEW_MATRIX:           wgpu::BufferAddress = 0;
    pub const OFFSET_PROJECT_MATRIX:        wgpu::BufferAddress = 16 * 4;
    pub const OFFSET_VIEW_PROJECT_MATRIX:   wgpu::BufferAddress = 16 * 4 + 16 * 4;
    pub const OFFSET_CAMERA_POSITION:       wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4;
    pub const OFFSET_CAMERA_DIRECTION:      wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4 + 4 * 4;
    
    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4 + 4 * 4 + 4 * 4;
    pub fn new(data: DynMergyBufferRange) -> Self {
        Self { data }
    }
    pub fn data(&self) -> &DynMergyBufferRange {
        &self.data
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseSceneAboutCamera {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindSceneAboutCamera>,
}
impl BindUseSceneAboutCamera {
    pub fn new(bind: u32, data: Arc<ShaderBindSceneAboutCamera>) -> Self {
        Self { bind, data }
    }
}
impl TShaderBindCode for BindUseSceneAboutCamera {
    fn vs_define_code(&self, set: u32) -> String {
        let mut result = String::from("");
        result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
        result += " Camera {\r\n";
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::VIEW_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::PROJECT_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::VIEW_PROJECT_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::CAMERA_POSITION).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::CAMERA_DIRECTION).as_str();
        result += "};\r\n";
        result
    }
    fn fs_define_code(&self, set: u32) -> String {
        self.vs_define_code(set)
    }
}
impl TShaderBind for BindUseSceneAboutCamera {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.bind,
                visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindSceneAboutCamera::TOTAL_SIZE) },
                count: None,
            }
        )
    }
    fn bind(&self) -> u32 {
        self.bind
    }
}
impl TKeyBind for BindUseSceneAboutCamera {
    fn key_bind(&self) -> crate::bind_group::bind::KeyBind {
        crate::bind_group::bind::KeyBind::Buffer(
            crate::bind_group::bind::KeyBindBuffer {
                bind: self.bind,
                id_buffer: self.data.data.clone(),
                entry: wgpu::BindGroupLayoutEntry {
                    binding: self.bind,
                    visibility: wgpu::ShaderStages ::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindSceneAboutCamera::TOTAL_SIZE) },
                    count: None,
                },
            }
        )
    }
}
impl TRenderBindBufferData for BindUseSceneAboutCamera {
    fn buffer(&self) -> &render_core::rhi::buffer::Buffer {
        self.data.data.buffer()
    }

    fn dyn_offset(&self) -> wgpu::DynamicOffset {
        self.data.data.start() as wgpu::DynamicOffset
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindSceneAboutTime {
    pub(crate) data: DynMergyBufferRange,
}
impl ShaderBindSceneAboutTime {

    pub const OFFSET_TIME:                  wgpu::BufferAddress = 0;
    pub const OFFSET_DELTA_TIME:            wgpu::BufferAddress = 4 * 4;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 4 * 4 + 4 * 4;

    pub fn new(data: DynMergyBufferRange) -> Self {
        Self { data }
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseSceneAboutTime {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindSceneAboutTime>,
}
impl BindUseSceneAboutTime {
    pub fn new(bind: u32, data: Arc<ShaderBindSceneAboutTime>) -> Self {
        Self { bind, data }
    }
}
impl TShaderBindCode for BindUseSceneAboutTime {
    fn vs_define_code(&self, set: u32) -> String {
        let mut result = String::from("");
        result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
        result += " Time {\r\n";
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::TIME).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::DELTA_TIME).as_str();
        result += "};\r\n";
        result
    }
    fn fs_define_code(&self, set: u32) -> String {
        self.vs_define_code(set)
    }
}
impl TShaderBind for BindUseSceneAboutTime {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.bind,
                visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindSceneAboutTime::TOTAL_SIZE) },
                count: None,
            }
        );
    }
    fn bind(&self) -> u32 {
        self.bind
    }
}
impl TKeyBind for BindUseSceneAboutTime {
    fn key_bind(&self) -> crate::bind_group::bind::KeyBind {
        crate::bind_group::bind::KeyBind::Buffer(
            crate::bind_group::bind::KeyBindBuffer {
                bind: self.bind,
                id_buffer: self.data.data.clone(),
                entry: wgpu::BindGroupLayoutEntry {
                    binding: self.bind,
                    visibility: wgpu::ShaderStages ::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindSceneAboutTime::TOTAL_SIZE) },
                    count: None,
                },
            }
        )
    }
}
impl TRenderBindBufferData for BindUseSceneAboutTime {
    fn buffer(&self) -> &render_core::rhi::buffer::Buffer {
        self.data.data.buffer()
    }

    fn dyn_offset(&self) -> wgpu::DynamicOffset {
        self.data.data.start() as wgpu::DynamicOffset
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindSceneAboutFog {
    pub(crate) data: DynMergyBufferRange,
}
impl ShaderBindSceneAboutFog {

    pub const OFFSET_FOG_INFO:              wgpu::BufferAddress = 0;
    pub const OFFSET_FOG_PARAM:             wgpu::BufferAddress = 4 * 4;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 4 * 4 + 4 * 4;
    pub fn new(data: DynMergyBufferRange) -> Self {
        Self { data }
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseSceneAboutFog {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindSceneAboutFog>,
}
impl BindUseSceneAboutFog {
    pub fn new(bind: u32, data: Arc<ShaderBindSceneAboutFog>) -> Self {
        Self { bind, data }
    }
}
impl TShaderBindCode for BindUseSceneAboutFog {
    fn vs_define_code(&self, set: u32) -> String {
        let mut result = String::from("");
        result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
        result += " Fog {\r\n";
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::FOG_INFO).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::FOG_PARAM).as_str();
        result += "};\r\n";
        result
    }
    fn fs_define_code(&self, set: u32) -> String {
        self.vs_define_code(set)
    }
}
impl TShaderBind for BindUseSceneAboutFog {

    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.bind,
                visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindSceneAboutFog::TOTAL_SIZE) },
                count: None,
            }
        );
    }
    fn bind(&self) -> u32 {
        self.bind
    }
}
impl TKeyBind for BindUseSceneAboutFog {
        fn key_bind(&self) -> crate::bind_group::bind::KeyBind {
        crate::bind_group::bind::KeyBind::Buffer(
            crate::bind_group::bind::KeyBindBuffer {
                bind: self.bind,
                id_buffer: self.data.data.clone(),
                entry: wgpu::BindGroupLayoutEntry {
                    binding: self.bind,
                    visibility: wgpu::ShaderStages ::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindSceneAboutFog::TOTAL_SIZE) },
                    count: None,
                },
            }
        )
    }
}
impl TRenderBindBufferData for BindUseSceneAboutFog {
    fn buffer(&self) -> &render_core::rhi::buffer::Buffer {
        self.data.data.buffer()
    }

    fn dyn_offset(&self) -> wgpu::DynamicOffset {
        self.data.data.start() as wgpu::DynamicOffset
    }
}



#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindSceneAboutAmbient {
    pub(crate) data: DynMergyBufferRange,
}
impl ShaderBindSceneAboutAmbient {

    pub const OFFSET_AMBIENT:               wgpu::BufferAddress = 0;
    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 4 * 4;

    pub fn new(data: DynMergyBufferRange) -> Self {
        Self { data }
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseSceneAboutAmbient {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindSceneAboutAmbient>,
}
impl BindUseSceneAboutAmbient {
    pub fn new(bind: u32, data: Arc<ShaderBindSceneAboutAmbient>) -> Self {
        Self { bind, data }
    }
}
impl TShaderBindCode for BindUseSceneAboutAmbient {
    fn vs_define_code(&self, set: u32) -> String {
        let mut result = String::from("");
        result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
        result += " Ambient {\r\n";
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::AMBIENT_PARAM).as_str();
        result += "};\r\n";
        result
    }
    fn fs_define_code(&self, set: u32) -> String {
        self.vs_define_code(set)
    }
}
impl TShaderBind for BindUseSceneAboutAmbient {

    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.bind,
                visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindSceneAboutAmbient::TOTAL_SIZE) },
                count: None,
            }
        );
    }
    fn bind(&self) -> u32 {
        self.bind
    }
}
impl TKeyBind for BindUseSceneAboutAmbient {
        fn key_bind(&self) -> crate::bind_group::bind::KeyBind {
        crate::bind_group::bind::KeyBind::Buffer(
            crate::bind_group::bind::KeyBindBuffer {
                bind: self.bind,
                id_buffer: self.data.data.clone(),
                entry: wgpu::BindGroupLayoutEntry {
                    binding: self.bind,
                    visibility: wgpu::ShaderStages ::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindSceneAboutAmbient::TOTAL_SIZE) },
                    count: None,
                },
            }
        )
    }
}
impl TRenderBindBufferData for BindUseSceneAboutAmbient {
    fn buffer(&self) -> &render_core::rhi::buffer::Buffer {
        self.data.data.buffer()
    }

    fn dyn_offset(&self) -> wgpu::DynamicOffset {
        self.data.data.start() as wgpu::DynamicOffset
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindSceneAboutEffect {
    pub(crate) data: DynMergyBufferRange,
}
impl ShaderBindSceneAboutEffect {

    pub const OFFSET_TIME:                  wgpu::BufferAddress = 0;
    pub const SIZE_TIME:                    wgpu::BufferAddress = 4 * 4;
    pub const OFFSET_DELTA_TIME:            wgpu::BufferAddress = Self::OFFSET_TIME + Self::SIZE_TIME;
    pub const SIZE_DELTA_TIME:              wgpu::BufferAddress = 4 * 4;

    pub const OFFSET_FOG_INFO:              wgpu::BufferAddress = Self::OFFSET_DELTA_TIME + Self::SIZE_DELTA_TIME;
    pub const SIZE_FOG_INFO:                wgpu::BufferAddress = 4 * 4;
    pub const OFFSET_FOG_PARAM:             wgpu::BufferAddress = Self::OFFSET_FOG_INFO + Self::SIZE_FOG_INFO;
    pub const SIZE_FOG_PARAM:               wgpu::BufferAddress = 4 * 4;

    pub const OFFSET_AMBIENT:               wgpu::BufferAddress = Self::OFFSET_FOG_PARAM + Self::SIZE_FOG_PARAM;
    pub const SIZE_AMBIENT:                 wgpu::BufferAddress = 4 * 4;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = Self::OFFSET_AMBIENT + Self::SIZE_AMBIENT;

    pub fn new(data: DynMergyBufferRange) -> Self {
        Self { data }
    }
    pub fn data(&self) -> &DynMergyBufferRange {
        &self.data
    }
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseSceneAboutEffect {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindSceneAboutEffect>,
}
impl BindUseSceneAboutEffect {
    pub fn new(bind: u32, data: Arc<ShaderBindSceneAboutEffect>) -> Self {
        Self { bind, data }
    }
}
impl TShaderBindCode for BindUseSceneAboutEffect {
    fn vs_define_code(&self, set: u32) -> String {
        let mut result = String::from("");
        result += ShaderSetBind::code_set_bind_head(set, self.bind).as_str();
        result += " SceneEffect {\r\n";
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::TIME).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::DELTA_TIME).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::FOG_INFO).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::FOG_PARAM).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::AMBIENT_PARAM).as_str();
        result += "};\r\n";
        result
    }
    fn fs_define_code(&self, set: u32) -> String {
        self.vs_define_code(set)
    }
}
impl TShaderBind for BindUseSceneAboutEffect {

    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.bind,
                visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindSceneAboutEffect::TOTAL_SIZE) },
                count: None,
            }
        );
    }
    fn bind(&self) -> u32 {
        self.bind
    }
}
impl TKeyBind for BindUseSceneAboutEffect {
        fn key_bind(&self) -> crate::bind_group::bind::KeyBind {
        crate::bind_group::bind::KeyBind::Buffer(
            crate::bind_group::bind::KeyBindBuffer {
                bind: self.bind,
                id_buffer: self.data.data.clone(),
                entry: wgpu::BindGroupLayoutEntry {
                    binding: self.bind,
                    visibility: wgpu::ShaderStages ::VERTEX,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(ShaderBindSceneAboutEffect::TOTAL_SIZE) },
                    count: None,
                },
            }
        )
    }
}
impl TRenderBindBufferData for BindUseSceneAboutEffect {
    fn buffer(&self) -> &render_core::rhi::buffer::Buffer {
        self.data.data.buffer()
    }

    fn dyn_offset(&self) -> wgpu::DynamicOffset {
        self.data.data.start() as wgpu::DynamicOffset
    }
}
