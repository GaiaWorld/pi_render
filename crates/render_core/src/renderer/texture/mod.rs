mod bind_texture;
mod texture_format;
mod image_texture;
mod image_texture_view;
mod render_target;
pub mod texture_view;
mod texture_view_array;

pub use bind_texture::*;
pub use texture_format::*;
pub use image_texture::*;
pub use image_texture_view::*;
pub use render_target::*;
pub use texture_view::*;
pub use texture_view_array::*;


#[derive(Debug, Clone, Default, Hash, PartialEq, Eq)]
pub struct TextureViewDesc { 
    // /// Aspect of the texture. Color textures must be [`TextureAspect::All`].
    // pub aspect: wgpu::TextureAspect,
    /// Base mip level.
    pub base_mip_level: u8,
    /// Mip level count.
    /// If `Some(count)`, `base_mip_level + count` must be less or equal to underlying texture mip count.
    /// If `None`, considered to include the rest of the mipmap levels, but at least 1 in total.
    pub mip_level_count: Option<u8>,
    /// Base array layer.
    pub base_array_layer: u8,
    /// Layer count.
    /// If `Some(count)`, `base_array_layer + count` must be less or equal to the underlying array count.
    /// If `None`, considered to include the rest of the array layers, but at least 1 in total.
    pub array_layer_count: Option<u8>,
}
impl TextureViewDesc {
    pub fn mip_level_count(&self) -> Option<u32> {
        if let Some(v) = self.mip_level_count {
            Some(v as u32)
        } else { None }
    }
    pub fn array_layer_count(&self) -> Option<u32> {
        if let Some(v) = self.array_layer_count {
            Some(v as u32)
        } else { None }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TextureRect {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) w: u16,
    pub(crate) h: u16,
}
impl Default for TextureRect {
    fn default() -> Self {
        Self { x: 0, y: 0, w: 1, h: 1 }
    }
}
