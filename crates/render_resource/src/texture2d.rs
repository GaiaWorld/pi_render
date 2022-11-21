use pi_assets::asset::{Asset, Handle};
use render_core::rhi::{texture::Sampler, asset::TextureRes};


pub struct Texture2D {
    pub texture: Handle<TextureRes>,
    pub sampler: Sampler,
}

impl Asset for Texture2D {
    type Key = TextureAssetKey;

    fn size(&self) -> usize {
        1
    }
}