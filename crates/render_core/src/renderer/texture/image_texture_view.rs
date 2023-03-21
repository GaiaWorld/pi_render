use std::{num::NonZeroU32, sync::Arc};

use pi_assets::asset::{Handle, Asset};

use crate::{asset::TAssetKeyU64, rhi::texture::TextureView};

use super::{image_texture::{KeyImageTexture, ImageTexture}, TextureViewDesc};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyImageTextureView {
    pub(crate) tex: KeyImageTexture,
    pub(crate) desc: TextureViewDesc,
}
impl TAssetKeyU64 for KeyImageTextureView {}

pub struct ImageTextureView {
    pub(crate) texture: Handle<ImageTexture>,
    pub(crate) view: TextureView,
}
impl Asset for ImageTextureView {
    type Key = u64;
    fn size(&self) -> usize {
        8 + 64
    }
}
impl ImageTextureView {
    pub fn new(
        key: &KeyImageTextureView,
        texture: Handle<ImageTexture>,
    ) -> Self {
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(texture.format.clone()),
            dimension: Some(texture.dimension.clone()),
            aspect: key.desc.aspect,
            base_mip_level: key.desc.base_mip_level,
            mip_level_count: key.desc.mip_level_count,
            base_array_layer: key.desc.base_array_layer,
            array_layer_count: key.desc.array_layer_count,
        });

        Self {
            texture,
            view: TextureView::with_texture(Arc::new(view)) ,
        }
    }
}

pub type EImageTextureViewUsage = Handle<ImageTextureView>;
// #[derive(Clone)]
// pub enum EImageTextureViewUsage {
//     Handle(Handle<ImageTextureView>),
//     Arc(Arc<ImageTextureView>),
// }
