use std::{sync::Arc, ops::Deref};

use pi_assets::{asset::{Handle, Asset, Size, Garbageer}, mgr::LoadResult};
use pi_futures::BoxFuture;

use crate::{asset::TAssetKeyU64, rhi::texture::TextureView};

use super::{image_texture::{KeyImageTexture, ImageTexture}, TextureViewDesc};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyImageTextureView {
    pub(crate) tex: KeyImageTexture,
    pub(crate) desc: TextureViewDesc,
}
impl TAssetKeyU64 for KeyImageTextureView {}
impl KeyImageTextureView {
    pub fn new(tex: KeyImageTexture, desc: TextureViewDesc) -> Self {
        Self { tex, desc }
    }
    pub fn url(&self) -> &KeyImageTexture {
        &self.tex
    }
    pub fn view_desc(&self) -> &TextureViewDesc {
        &self.desc
    }
}

#[derive(Debug)]
pub struct ImageTextureView {
    pub(crate) texture: Handle<ImageTexture>,
    pub(crate) view: TextureView,
}
impl Asset for ImageTextureView {
    type Key = u64;
    // const TYPE: &'static str = "ImageTextureView";
}

impl Size for ImageTextureView {
    fn size(&self) -> usize {
        self.texture.size() + 64
    }
}
impl ImageTextureView {
    pub fn new(
        key: &KeyImageTextureView,
        texture: Handle<ImageTexture>,
    ) -> Self {
        let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(key.url().deref()),
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
    pub fn texture(&self) -> &Handle<ImageTexture> {
        &self.texture
    }
    pub fn async_load<'a, G: Garbageer<Self>>(image: Handle<ImageTexture>, key: KeyImageTextureView, result: LoadResult<'a, Self, G>) -> BoxFuture<'a, Result<Handle<Self>, ()>> {
        Box::pin(async move {
            match result {
                LoadResult::Ok(r) => { Ok(r) },
                LoadResult::Wait(f) => {
                    // log::error!("ImageTexture Wait");
                    match f.await {
                        Ok(result) => Ok(result),
                        Err(_) => Err(()),
                    }
                },
                LoadResult::Receiver(recv) => {
                    let texture_view = Self::new(&key, image);
                    let key = key.asset_u64();
                    match recv.receive(key, Ok(texture_view)).await {
                        Ok(result) => { Ok(result) },
                        Err(_) => Err(()),
                    }
                }
            }
        })
    }
}

pub type EImageTextureViewUsage = Handle<ImageTextureView>;
// #[derive(Clone)]
// pub enum EImageTextureViewUsage {
//     Handle(Handle<ImageTextureView>),
//     Arc(Arc<ImageTextureView>),
// }
