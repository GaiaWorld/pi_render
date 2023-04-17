mod bind_texture;
mod texture_format;
mod image_texture;
mod image_texture_view;
mod render_target;
mod texture_view;
mod texture_view_array;

use std::num::NonZeroU32;

pub use bind_texture::*;
pub use texture_format::*;
pub use image_texture::*;
pub use image_texture_view::*;
pub use render_target::*;
pub use texture_view::*;
pub use texture_view_array::*;


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TextureViewDesc { 
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TextureRect {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) w: u32,
    pub(crate) h: u32,
}
impl Default for TextureRect {
    fn default() -> Self {
        Self { x: 0, y: 0, w: 1, h: 1 }
    }
}
