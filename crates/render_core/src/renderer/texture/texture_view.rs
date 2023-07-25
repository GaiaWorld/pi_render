use std::{hash::Hash, sync::Arc};

use pi_assets::asset::Handle;
use pi_atom::Atom;
use pi_slotmap::DefaultKey;

use crate::{rhi::{asset::TextureRes}, components::view::target_alloc::ShareTargetView, asset::TAssetKeyU64};

use super::{image_texture_view::{ImageTextureView, EImageTextureViewUsage}, render_target::{ERenderTargetViewUsage}, TextureRect, KeyTexture, KeyImageTexture, KeyImageTextureView, TextureViewDesc, ImageTexture};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum KeyTextureViewUsage {
    Tex(u64, TextureRect),
    Image(u64, TextureRect),
    Render(u64, TextureRect),
    SRT(DefaultKey, DefaultKey, TextureRect),
    Temp(u64, TextureRect),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum EKeyTexture {
    Tex(KeyTexture),
    Image(KeyImageTextureView),
    SRT(u64),
}
impl EKeyTexture {
    pub fn image(value: &str) -> Self {
        EKeyTexture::Image(KeyImageTextureView { tex: KeyImageTexture::File(Atom::from(value), false), desc: TextureViewDesc::default() })
    }
}
impl From<&str> for EKeyTexture {
    fn from(value: &str) -> Self {
        Self::image(value)
        // EKeyTexture::Tex(KeyTexture::from(value))
    }
}

#[derive(Clone, Debug)]
pub enum ETextureViewUsage {
    Tex(Handle<TextureRes>),
    Image(EImageTextureViewUsage),
    Render(ERenderTargetViewUsage),
    SRT(ShareTargetView),
    Temp(Arc<wgpu::TextureView>, u64),
}
impl ETextureViewUsage {
    pub fn key(&self) -> KeyTextureViewUsage {
        match self {
            ETextureViewUsage::Image(val) => {
                KeyTextureViewUsage::Image(*val.key(), TextureRect { x: 0, y: 0, w: val.texture.width as u16, h: val.texture.height as u16 }) 
            },
            ETextureViewUsage::Render(val) => {
                KeyTextureViewUsage::Render(val.key(), TextureRect { x: 0, y: 0, w: 1, h: 1 }) 
            },
            ETextureViewUsage::Tex(val) => {
                KeyTextureViewUsage::Tex(*val.key(), TextureRect { x: 0, y: 0, w: val.width as u16, h: val.height as u16 })
            },
            ETextureViewUsage::SRT(val) => {
                let rect = val.rect();
                let x = rect.min.x as u16;
                let y = rect.min.y as u16;
                let w = (rect.max.x - rect.min.x) as u16;
                let h = (rect.max.y - rect.min.y) as u16;
                KeyTextureViewUsage::SRT(val.ty_index(), val.target_index(), TextureRect { x, y, w, h })
            },
            ETextureViewUsage::Temp(_, key) => {
                KeyTextureViewUsage::Temp(*key, TextureRect::default()) 
            },
        }
    }
    pub fn view(&self) -> &wgpu::TextureView {
        match self {
            ETextureViewUsage::Image(val) => &val.view,
            ETextureViewUsage::Render(val) => val.view(),
            ETextureViewUsage::Tex(val) => &val.texture_view,
            ETextureViewUsage::SRT(val) => &val.target().colors[0].0,
            ETextureViewUsage::Temp(val, _) => val,
        }
    }
}
impl Hash for ETextureViewUsage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key().hash(state)
    }
}
impl PartialEq for ETextureViewUsage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Image(l0), Self::Image(r0)) => l0.key() == r0.key(),
            (Self::Render(l0), Self::Render(r0)) => l0 == r0,
            _ => false,
        }
    }
}
impl Eq for ETextureViewUsage {}
impl From<ShareTargetView> for ETextureViewUsage {
    fn from(value: ShareTargetView) -> Self {
        Self::SRT(value)
    }
}
impl From<Handle<TextureRes>> for ETextureViewUsage {
    fn from(value: Handle<TextureRes>) -> Self {
        Self::Tex(value)
    }
}
impl From<Handle<ImageTextureView>> for ETextureViewUsage {
    fn from(value: Handle<ImageTextureView>) -> Self {
        Self::Image(value)
    }
}
impl From<ERenderTargetViewUsage> for ETextureViewUsage {
    fn from(value: ERenderTargetViewUsage) -> Self {
        Self::Render(value)
    }
}
impl TAssetKeyU64 for ETextureViewUsage {}