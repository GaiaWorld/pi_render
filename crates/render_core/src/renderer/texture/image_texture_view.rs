use std::{sync::Arc, ops::Deref};

use pi_assets::{asset::{Handle, Asset, Size, Garbageer}, mgr::LoadResult};
use pi_futures::BoxFuture;
use pi_share::Share;
use wgpu::TextureView;

use crate::asset::TAssetKeyU64;

use super::{image_texture::{KeyImageTexture, ResImageTexture}, TextureViewDesc};

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

pub struct ImageTextureView {
    pub(crate) texture: Handle<ResImageTexture>,
    pub(crate) view: Share<TextureView>,
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
        texture: Handle<ResImageTexture>,
    ) -> Self {
        let view = texture.data.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(key.url().deref()),
            format: Some(texture.data.format.clone()),
            dimension: Some(texture.data.view_dimension.clone()),
            aspect: wgpu::TextureAspect::All, // key.desc.aspect,
            base_mip_level: key.desc.base_mip_level as u32,
            mip_level_count: key.desc.mip_level_count(),
            base_array_layer: key.desc.base_array_layer as u32,
            array_layer_count: key.desc.array_layer_count(),
        });

        Self {
            texture,
            view: Share::new(view) ,
        }
    }
    pub fn texture(&self) -> &Handle<ResImageTexture> {
        &self.texture
    }
    pub fn async_load<'a, G: Garbageer<Self>>(image: Handle<ResImageTexture>, key: KeyImageTextureView, result: LoadResult<'a, Self, G>) -> BoxFuture<'a, Result<Handle<Self>, ()>> {
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
