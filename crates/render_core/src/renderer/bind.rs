use std::{num::{NonZeroU64, NonZeroU32}, fmt::Debug, hash::Hash};

use super::{
    texture::{BindDataTextureArray, BindDataTexture2D},
    sampler::BindDataSampler,
    shader_stage::EShaderStage,
    bind_buffer::BindBufferRange
};

pub type KeyBindLayoutBindingType = u8;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindLayoutBuffer {
    pub visibility: EShaderStage,
    pub dynamic: bool,
    pub min_binding_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindLayoutTexture2D {
    pub visibility: EShaderStage,
    pub texture_sample_type: wgpu::TextureSampleType,
    pub view_dimension: wgpu::TextureViewDimension,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindLayoutTexture2DArray {
    pub count: u8,
    pub visibility: EShaderStage,
    pub texture_sample_type: wgpu::TextureSampleType,
    pub view_dimension: wgpu::TextureViewDimension,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindLayoutSampler {
    pub visibility: EShaderStage,
    pub binding_type: wgpu::SamplerBindingType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyBindLayout {
    Buffer(KeyBindLayoutBuffer),
    Texture2D(KeyBindLayoutTexture2D),
    Sampler(KeyBindLayoutSampler),
    Texture2DArray(KeyBindLayoutTexture2DArray),
    // SamplerArray(KeyBindSamplerArray),
}
impl KeyBindLayout {
    pub fn layout_entry(&self, binding: u32) -> wgpu::BindGroupLayoutEntry {
        match self {
            KeyBindLayout::Buffer(val) => {
                wgpu::BindGroupLayoutEntry {
                    binding: binding as u32,
                    visibility: val.visibility.mode(),
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(val.min_binding_size as u64)  },
                    count: None,
                }
            },
            KeyBindLayout::Texture2D(val) => {
                wgpu::BindGroupLayoutEntry {
                    binding: binding as u32,
                    visibility: val.visibility.mode(),
                    ty: wgpu::BindingType::Texture { sample_type: val.texture_sample_type, view_dimension: val.view_dimension, multisampled: false },
                    count: None,
                }
            },
            KeyBindLayout::Sampler(val) => {
                wgpu::BindGroupLayoutEntry {
                    binding: binding as u32,
                    visibility: val.visibility.mode(),
                    ty: wgpu::BindingType::Sampler(val.binding_type),
                    count: None,
                }
            },
            KeyBindLayout::Texture2DArray(val) => {
                wgpu::BindGroupLayoutEntry {
                    binding: binding as u32,
                    visibility: val.visibility.mode(),
                    ty: wgpu::BindingType::Texture { sample_type: val.texture_sample_type, view_dimension: val.view_dimension, multisampled: false },
                    count: NonZeroU32::new(val.count as u32),
                }
            },
            // KeyBind::SamplerArray(val) => val.entry.clone(),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindBuffer {
    pub data: BindBufferRange,
    pub layout: KeyBindLayoutBuffer,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindTexture2D {
    pub data: BindDataTexture2D,
    pub layout: KeyBindLayoutTexture2D,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindTexture2DArray {
    pub data: BindDataTextureArray,
    pub layout: KeyBindLayoutTexture2DArray,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindSampler {
    pub data: BindDataSampler,
    pub layout: KeyBindLayoutSampler,
}

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct KeyBindSamplerArray {
//     pub(crate) id_sampler: KeySamplerArray,
//     pub(crate) binding: u16,
//     pub(crate) visibility: EShaderStage,
//     pub(crate) binding_type: wgpu::SamplerBindingType,
// }



/// * Bind 数据的 Key
/// ** 包含了 数据的Key 和 layout的key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EKeyBind {
    Buffer(KeyBindBuffer),
    Texture2D(KeyBindTexture2D),
    Sampler(KeyBindSampler),
    // Texture2DArray(KeyBindTexture2DArray),
    // SamplerArray(KeyBindSamplerArray),
}
impl EKeyBind {
    pub(crate) fn bind_source<'a>(
        &'a self,
    ) -> wgpu::BindingResource<'a> {
        match self {
            Self::Buffer(val) => wgpu::BindingResource::Buffer(
                wgpu::BufferBinding {
                    buffer: &val.data.buffer(),
                    offset: 0,
                    size: NonZeroU64::new(val.data.size() as u64),
                }
            ),
            Self::Texture2D(val) => {
                wgpu::BindingResource::TextureView(val.data.view())
            },
            Self::Sampler(val) => {
                wgpu::BindingResource::Sampler(&val.data.0.0)
            },
            // Self::Texture2DArray(val) => {
            //     wgpu::BindingResource::TextureViewArray(&val.data.array())
            // }
        }
    }
    pub fn key_bind_layout(&self) -> KeyBindLayout {
        match self {
            EKeyBind::Buffer(val) => KeyBindLayout::Buffer(val.layout.clone()),
            EKeyBind::Texture2D(val) => KeyBindLayout::Texture2D(val.layout.clone()),
            EKeyBind::Sampler(val) => KeyBindLayout::Sampler(val.layout.clone()),
            // EKeyBind::Texture2DArray(val) => KeyBindLayout::Texture2DArray(val.layout.clone()),
        }
    }
}
pub trait TKeyBind {
    /// * 获取 Bind 数据的 Key
    fn key_bind(&self) -> Option<EKeyBind>;
}
