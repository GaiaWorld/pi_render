use std::{num::{NonZeroU64, NonZeroU32}, fmt::Debug, hash::Hash, sync::Arc};

use wgpu::ShaderStages;

use super::{
    texture::{BindDataTextureArray, BindDataTexture2D},
    sampler::BindDataSampler,
    shader_stage::EShaderStage,
    bind_buffer::BindBufferRange
};

pub type KeyBindLayoutBindingType = u8;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindLayoutBuffer {
    pub binding: KeyBindLayoutBindingType,
    pub visibility: EShaderStage,
    pub min_binding_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindLayoutTexture2D {
    pub binding: KeyBindLayoutBindingType,
    pub visibility: EShaderStage,
    pub texture_sample_type: wgpu::TextureSampleType,
    pub view_dimension: wgpu::TextureViewDimension,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindLayoutTexture2DArray {
    pub binding: KeyBindLayoutBindingType,
    pub count: u8,
    pub visibility: EShaderStage,
    pub texture_sample_type: wgpu::TextureSampleType,
    pub view_dimension: wgpu::TextureViewDimension,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindLayoutSampler {
    pub binding: KeyBindLayoutBindingType,
    pub visibility: EShaderStage,
    pub binding_type: wgpu::SamplerBindingType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyBindLayout {
    Buffer(Arc<KeyBindLayoutBuffer>),
    Texture2D(Arc<KeyBindLayoutTexture2D>),
    Sampler(Arc<KeyBindLayoutSampler>),
    Texture2DArray(Arc<KeyBindLayoutTexture2DArray>),
    // SamplerArray(KeyBindSamplerArray),
}
impl KeyBindLayout {
    pub fn bind(&self) -> u32 {
        match self {
            KeyBindLayout::Buffer(val) => val.binding as u32,
            KeyBindLayout::Texture2D(val) => val.binding as u32,
            KeyBindLayout::Sampler(val) => val.binding as u32,
            KeyBindLayout::Texture2DArray(val) => val.binding as u32,
        }
    }
    pub fn layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
        match self {
            KeyBindLayout::Buffer(val) => {
                wgpu::BindGroupLayoutEntry {
                    binding: val.binding as u32,
                    visibility: val.visibility.mode(),
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(val.min_binding_size as u64)  },
                    count: None,
                }
            },
            KeyBindLayout::Texture2D(val) => {
                wgpu::BindGroupLayoutEntry {
                    binding: val.binding as u32,
                    visibility: val.visibility.mode(),
                    ty: wgpu::BindingType::Texture { sample_type: val.texture_sample_type, view_dimension: val.view_dimension, multisampled: false },
                    count: None,
                }
            },
            KeyBindLayout::Sampler(val) => {
                wgpu::BindGroupLayoutEntry {
                    binding: val.binding as u32,
                    visibility: val.visibility.mode(),
                    ty: wgpu::BindingType::Sampler(val.binding_type),
                    count: None,
                }
            },
            KeyBindLayout::Texture2DArray(val) => {
                wgpu::BindGroupLayoutEntry {
                    binding: val.binding as u32,
                    visibility: val.visibility.mode(),
                    ty: wgpu::BindingType::Texture { sample_type: val.texture_sample_type, view_dimension: val.view_dimension, multisampled: false },
                    count: NonZeroU32::new(val.count as u32),
                }
            },
            // KeyBind::SamplerArray(val) => val.entry.clone(),
        }
    }
}

pub enum EBindResource<'a> {
    Buffer(u32, &'a wgpu::Buffer, u32),
    Texture2D(u32, &'a wgpu::TextureView),
    Sampler(u32, &'a wgpu::Sampler),
    Texture2DArray(u32, Vec<&'a wgpu::TextureView>),
}
impl<'a> EBindResource<'a> {
    pub(crate) fn entry(
        &'a self,
    ) -> wgpu::BindGroupEntry<'a> {
        match self {
            Self::Buffer(binding, buffer, size) => {
                wgpu::BindGroupEntry {
                    binding: *binding,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset: 0,
                        size: NonZeroU64::new(*size as u64),
                    }),
                }
            },
            Self::Texture2D(binding, val) => {
                wgpu::BindGroupEntry {
                    binding: *binding,
                    resource: wgpu::BindingResource::TextureView(val),
                }
            },
            Self::Sampler(binding, val) => {
                wgpu::BindGroupEntry {
                    binding: *binding,
                    resource: wgpu::BindingResource::Sampler(val),
                }
            },
            Self::Texture2DArray(binding, val) => {
                wgpu::BindGroupEntry {
                    binding: *binding,
                    resource: wgpu::BindingResource::TextureViewArray(val),
                }
            },
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EBindData {
    Buffer(BindBufferRange),
    Texture2D(BindDataTexture2D),
    Sampler(BindDataSampler),
    Texture2DArray(BindDataTextureArray),
}
impl EBindData {
    // pub(crate) fn bind_source<'a>(
    //     &'a self,
    //     binding: u32,
    // ) -> EBindResource<'a> {
    //     match self {
    //         EBindData::Buffer(val) => EBindResource::<'a>::Buffer(binding, &val.buffer(), val.size()),
    //         EBindData::Texture2D(val) => EBindResource::<'a>::Texture2D(binding, &val.0.texture_view),
    //         EBindData::Sampler(val) => EBindResource::<'a>::Sampler(binding, &val.0.0),
    //         EBindData::Texture2DArray(val) => {
    //             EBindResource::<'a>::Texture2DArray(binding, val.array())
    //         }
    //     }
    // }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindBuffer {
    pub data: BindBufferRange,
    pub layout: Arc<KeyBindLayoutBuffer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindTexture2D {
    pub data: BindDataTexture2D,
    pub layout: Arc<KeyBindLayoutTexture2D>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindTexture2DArray {
    pub data: BindDataTextureArray,
    pub layout: Arc<KeyBindLayoutTexture2DArray>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindSampler {
    pub data: BindDataSampler,
    pub layout: Arc<KeyBindLayoutSampler>,
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
    Texture2DArray(KeyBindTexture2DArray),
    // SamplerArray(KeyBindSamplerArray),
}
impl EKeyBind {
    pub(crate) fn bind_source<'a>(
        &'a self,
        binding: u32,
    ) -> EBindResource<'a> {
        match self {
            Self::Buffer(val) => EBindResource::<'a>::Buffer(binding, &val.data.buffer(), val.data.size()),
            Self::Texture2D(val) => EBindResource::<'a>::Texture2D(binding, &val.data.view()),
            Self::Sampler(val) => EBindResource::<'a>::Sampler(binding, &val.data.0.0),
            Self::Texture2DArray(val) => {
                EBindResource::<'a>::Texture2DArray(binding, val.data.array())
            }
        }
    }
    pub fn key_bind_layout(&self) -> KeyBindLayout {
        match self {
            EKeyBind::Buffer(val) => KeyBindLayout::Buffer(val.layout.clone()),
            EKeyBind::Texture2D(val) => KeyBindLayout::Texture2D(val.layout.clone()),
            EKeyBind::Sampler(val) => KeyBindLayout::Sampler(val.layout.clone()),
            EKeyBind::Texture2DArray(val) => KeyBindLayout::Texture2DArray(val.layout.clone()),
        }
    }
}
pub trait TKeyBind {
    /// * 获取 Bind 数据的 Key
    fn key_bind(&self) -> Option<EKeyBind>;
}
