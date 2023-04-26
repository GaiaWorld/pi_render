use std::{hash::Hash, sync::Arc};

use pi_assets::asset::Handle;
use pi_slotmap::DefaultKey;

use crate::{rhi::{texture::TextureView, asset::{TextureRes, RenderRes}}, components::view::target_alloc::ShareTargetView, asset::TAssetKeyU64};

use super::{image_texture_view::{ImageTextureView, EImageTextureViewUsage}, render_target::{ERenderTargetViewUsage}, TextureRect};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum KeyTextureViewUsage {
    Tex(u64, TextureRect),
    Image(u64, TextureRect),
    Render(u64, TextureRect),
    SRT(DefaultKey, DefaultKey, TextureRect),
    Temp(u64, TextureRect),
}

#[derive(Clone)]
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
                KeyTextureViewUsage::Image(*val.key(), TextureRect::default()) 
            },
            ETextureViewUsage::Render(val) => {
                KeyTextureViewUsage::Render(val.key(), TextureRect::default()) 
            },
            ETextureViewUsage::Tex(val) => {
                KeyTextureViewUsage::Tex(*val.key(), TextureRect::default()) 
            },
            ETextureViewUsage::SRT(val) => {
                let rect = val.rect();
                let x = rect.min.x as u32;
                let y = rect.min.y as u32;
                let w = (rect.max.x - rect.min.x) as u32;
                let h = (rect.max.y - rect.min.y) as u32;
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