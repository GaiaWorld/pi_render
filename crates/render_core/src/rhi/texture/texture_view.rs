use std::num::NonZeroU32;

use pi_assets::asset::{Handle, Asset};

use super::{ETexture, EKeyTexture};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextureViewDesc {    /// Format of the texture view. At this time, it must be the same as the underlying format of the texture.
    pub format: Option<wgpu::TextureFormat>,
    /// The dimension of the texture view. For 1D textures, this must be `1D`. For 2D textures it must be one of
    /// `D2`, `D2Array`, `Cube`, and `CubeArray`. For 3D textures it must be `3D`
    pub dimension: Option<wgpu::TextureViewDimension>,
    /// Aspect of the texture. Color textures must be [`TextureAspect::All`].
    pub aspect: wgpu::TextureAspect,
    /// Base mip level.
    pub base_mip_level: u32,
    /// Mip level count.
    /// If `Some(count)`, `base_mip_level + count` must be less or equal to underlying texture mip count.
    /// If `None`, considered to include the rest of the mipmap levels, but at least 1 in total.
    pub mip_level_count: Option<NonZeroU32>,
    /// Base array layer.
    pub base_array_layer: u32,
    /// Layer count.
    /// If `Some(count)`, `base_array_layer + count` must be less or equal to the underlying array count.
    /// If `None`, considered to include the rest of the array layers, but at least 1 in total.
    pub array_layer_count: Option<NonZeroU32>,
}
impl TextureViewDesc {
    pub fn desc(&self) -> wgpu::TextureViewDescriptor {
        wgpu::TextureViewDescriptor {
            label:              None,
            format:             self.format,
            dimension:          self.dimension,
            aspect:             self.aspect,
            base_mip_level:     self.base_mip_level,
            mip_level_count:    self.mip_level_count,
            base_array_layer:   self.base_array_layer,
            array_layer_count:  self.array_layer_count,
        }
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyTextureView(pub EKeyTexture, pub TextureViewDesc);

pub struct TextureView {
    texture: ETexture,
    view: wgpu::TextureView,
}
impl Asset for TextureView {
    type Key = KeyTextureView;
    fn size(&self) -> usize {
        8 + 64
    }
}
impl TextureView {
    pub fn new(
        key: &KeyTextureView,
        texture: ETexture,
    ) -> Self {
        let view = texture.create_view(&key.1.desc());

        Self {
            texture,
            view,
        }
    }
}
