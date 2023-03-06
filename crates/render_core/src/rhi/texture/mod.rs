pub mod texture_cache;
mod texture_impl;
pub mod image_texture;
pub mod texture_view;
pub mod texture_view_array;

use pi_assets::asset::Handle;
pub use texture_impl::*;

use self::{image_texture::{ImageTexture2D, KeyImageTexture2D}, texture_view::TextureViewDesc};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum EKeyTexture {
    Image2D(KeyImageTexture2D),
}

pub enum ETexture {
    Image2D(Handle<ImageTexture2D>),

}
impl ETexture {
    pub fn create_view(&self, view_desc: &wgpu::TextureViewDescriptor) -> wgpu::TextureView {
        match self {
            ETexture::Image2D(texture) => {
                texture.texture.create_view(view_desc)
            },
        }
    }
}
