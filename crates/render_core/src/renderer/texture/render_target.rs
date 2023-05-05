use std::{sync::Arc, num::NonZeroU32, hash::Hash};

use pi_assets::asset::{Handle, Asset};
use pi_share::Share;

use crate::{rhi::{device::RenderDevice, texture::{Texture, TextureView}}, renderer::texture::{texture_format::TTextureFormatPixelByte}, asset::{ASSET_SIZE_FOR_UNKOWN, TAssetKeyU64}};

use super::TextureViewDesc;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EDepthStencilFormat {
    // Depth and stencil formats
    /// Special depth format with 32 bit floating point depth.
    // #[cfg_attr(feature = "serde", serde(rename = "depth32float"))]
    Depth32Float,
    /// Special depth/stencil format with 32 bit floating point depth and 8 bits integer stencil.
    // #[cfg_attr(feature = "serde", serde(rename = "depth32float-stencil8"))]
    Depth32FloatStencil8,
    /// Special depth format with at least 24 bit integer depth.
    // #[cfg_attr(feature = "serde", serde(rename = "depth24plus"))]
    Depth24Plus,
    /// Special depth/stencil format with at least 24 bit integer depth and 8 bits integer stencil.
    // #[cfg_attr(feature = "serde", serde(rename = "depth24plus-stencil8"))]
    Depth24PlusStencil8,
    /// Special depth/stencil format with 24 bit integer depth and 8 bits integer stencil.
    // #[cfg_attr(feature = "serde", serde(rename = "depth24unorm-stencil8"))]
    Depth16Unorm,
}
impl EDepthStencilFormat {
    pub fn format(&self) -> wgpu::TextureFormat {
        match self {
            EDepthStencilFormat::Depth32Float               => wgpu::TextureFormat::Depth32Float        ,
            EDepthStencilFormat::Depth32FloatStencil8       => wgpu::TextureFormat::Depth32FloatStencil8,
            EDepthStencilFormat::Depth24Plus                => wgpu::TextureFormat::Depth24Plus         ,
            EDepthStencilFormat::Depth24PlusStencil8        => wgpu::TextureFormat::Depth24PlusStencil8 ,
            EDepthStencilFormat::Depth16Unorm               => wgpu::TextureFormat::Depth16Unorm,
        }
    }
}

pub struct RenderTargetAllocator {

}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyRenderTexture {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    idx: usize,
}

#[derive(Debug)]
pub struct RenderTexture {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) texture: Texture,
    pub(crate) format: wgpu::TextureFormat,
    size: usize,
}
impl RenderTexture {
    fn new(width: u32, height: u32, format: wgpu::TextureFormat, texture: Texture) -> Self {
        Self {
            width,
            height,
            texture,
            format,
            size: (width * height) as usize * format.pixel_bytes(),
        }
    }
    pub fn color(device: &RenderDevice, width: u32, height: u32, format: wgpu::TextureFormat) -> RenderTexture {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB so we need to reflect that here.
                format,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                label: None,
				view_formats: &[],
            }
        );

        RenderTexture::new(width, height, format, texture)
    }
    pub fn depth(device: &RenderDevice, width: u32, height: u32, format: EDepthStencilFormat) -> RenderTexture {
        let format = format.format();
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                size,
                mip_level_count: 1, // We'll talk about this a little later
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB so we need to reflect that here.
                format,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                label: None,
				view_formats: &[],
            }
        );

        RenderTexture::new(width, height, format, texture)
    }
}

impl Asset for RenderTexture {
    type Key = KeyRenderTexture;
    fn size(&self) -> usize {
        self.size
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyRenderTargetView {
    pub texture: KeyRenderTexture,
    pub desc: TextureViewDesc,
}
impl TAssetKeyU64 for KeyRenderTargetView {}
pub type KeyRenderTargetViewU64 = u64;

pub struct RenderTargetView {
    pub(crate) texture: Handle<RenderTexture>,
    pub(crate) view: TextureView,
}
impl RenderTargetView {
    pub fn view(&self) -> &TextureView {
        &self.view
    }
}
impl Asset for RenderTargetView {
    type Key = u64;
    fn size(&self) -> usize {
        ASSET_SIZE_FOR_UNKOWN
    }
}

// pub enum ERenderTargetView {
//     Color(Handle<RenderTexture>, TextureView),
//     ColorDepth(Handle<RenderTexture>, Handle<RenderTexture>, TextureView, TextureView),
//     Depth(Handle<RenderTexture>, TextureView),
// }

// impl ERenderTargetView {
//     pub fn new(
//         device: &RenderDevice,
//         color: Option<(Handle<RenderTexture>, wgpu::TextureViewDescriptor)>,
//         depth_stencil: Option<(Handle<RenderTexture>, wgpu::TextureViewDescriptor)>,
//     ) -> Option<Self> {
//         match (color, depth_stencil) {
//             (None, None) => None,
//             (None, Some((tex, desc))) => {
//                 let view = tex.texture.create_view(&desc);
//                 Some(
//                     Self::Depth(tex, view)
//                 )
//             },
//             (Some((tex, desc)), None) => {
//                 let view = tex.texture.create_view(&desc);
//                 Some(
//                     Self::Color(tex, view)
//                 )
//             },
//             (Some((tex, desc)), Some((tex2, desc2))) => {
//                 let view = tex.texture.create_view(&desc);
//                 let view2 = tex2.texture.create_view(&desc2);
//                 Some(
//                     Self::ColorDepth(tex, tex2, view, view2)
//                 )
//             },
//         }
//     }
//     pub fn color_view(& self) -> Option<& TextureView> {
//         match self {
//             Self::Color(_, val) => Some(&val),
//             Self::ColorDepth(_, _, val, _) => Some(&val),
//             Self::Depth(_, _) => None,
//         }
//     }
//     pub fn depth_view(& self) -> Option<& TextureView> {
//         match self {
//             Self::Color(_, _) => None,
//             Self::ColorDepth(_, _, _, val) => Some(&val),
//             Self::Depth(_, val) => Some(&val),
//         }
//     }
// }

// impl Asset for ERenderTargetView {
//     type Key = u64;
//     fn size(&self) -> usize {
//         ASSET_SIZE_FOR_UNKOWN
//     }
// }

#[derive(Clone)]
pub enum ERenderTargetViewUsage {
    Handle(Handle<RenderTargetView>),
    Arc(Arc<RenderTargetView>, KeyRenderTargetViewU64),
}
impl ERenderTargetViewUsage {
    pub fn key(&self) -> u64 {
        match self {
            ERenderTargetViewUsage::Handle(val) => *val.key(),
            ERenderTargetViewUsage::Arc(val, key) => *key,
        }
    }
    pub fn view(&self) -> &TextureView {
        match self {
            ERenderTargetViewUsage::Handle(val) => val.view(),
            ERenderTargetViewUsage::Arc(val, _) => val.view(),
        }
    }
}
impl PartialEq for ERenderTargetViewUsage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Handle(l0), Self::Handle(r0)) => l0.key() == r0.key(),
            (Self::Arc(l0, l1), Self::Arc(r0, r1)) => l1 == r1,
            _ => false,
        }
    }
}
impl Eq for ERenderTargetViewUsage {}
