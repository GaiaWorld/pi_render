use std::hash::Hash;

use pi_assets::asset::Handle;
use pi_atom::Atom;
use pi_slotmap::DefaultKey;

use crate::{asset::TAssetKeyU64, components::view::target_alloc::ShareTargetView, rhi::asset::{AssetWithId, TextureRes}};

use super::{image_texture_view::{EImageTextureViewUsage, ImageTextureView}, ImageTextureViewFrame, KeyImageTexture, KeyImageTextureFrame, KeyImageTextureView, KeyImageTextureViewFrame, KeyTexture, TextureRect, TextureViewDesc};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum KeyTextureViewUsage {
    Tex(u64, TextureRect),
    Image(u64, TextureRect),
    Render(u64, TextureRect),
    ImageFrame(u64),
    SRT(DefaultKey, DefaultKey, TextureRect),
    Temp(u64, TextureRect),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, )]
pub enum EKeyTexture {
    Tex(KeyTexture),
    Image(KeyImageTextureView),
    ImageFrame(KeyImageTextureViewFrame),
    SRT(u64),
}
impl Default for EKeyTexture{
    fn default() -> Self {
        Self::Tex(Default::default())
    }
}
impl EKeyTexture {
    pub fn image(value: &str) -> Self {
        EKeyTexture::Image(KeyImageTextureView { tex: KeyImageTexture { url: Atom::from(value), file: true, ..Default::default() }, desc: TextureViewDesc::default() })
    }
    pub fn combine(value: &str, compressed: bool) -> Self {
        // Self::image(value)
        Self::ImageFrame(KeyImageTextureViewFrame {
            tex: KeyImageTextureFrame {
                url: Atom::from(value),
                file: true,
                compressed,
                cancombine: true,
            },
            desc: TextureViewDesc::default(),
        })
        // EKeyTexture::Tex(KeyTexture::from(value))
    }
}
impl From<&str> for EKeyTexture {
    fn from(value: &str) -> Self {
        // Self::image(value)
        Self::ImageFrame(KeyImageTextureViewFrame {
            tex: KeyImageTextureFrame {
                url: Atom::from(value),
                file: true,
                compressed: false,
                cancombine: false,
            },
            desc: TextureViewDesc::default(),
        })
        // EKeyTexture::Tex(KeyTexture::from(value))
    }
}

#[derive(Clone)]
pub enum ETextureViewUsage {
    Tex(Handle<TextureRes>),
    TexWithId(Handle<AssetWithId<TextureRes>>),
    Image(EImageTextureViewUsage),
    ImageFrame(Handle<ImageTextureViewFrame>),
    // Render(ERenderTargetViewUsage),
    SRT(ShareTargetView),
    // Temp(Arc<wgpu::TextureView>, u64),
}
impl ETextureViewUsage {
    pub fn key(&self) -> KeyTextureViewUsage {
        match self {
            ETextureViewUsage::Image(val) => {
                KeyTextureViewUsage::Image(*val.key(), TextureRect { x: 0, y: 0, w: val.texture.width() as u16, h: val.texture.height() as u16 }) 
            },
            // ETextureViewUsage::Render(val) => {
            //     KeyTextureViewUsage::Render(val.key(), TextureRect { x: 0, y: 0, w: 1, h: 1 }) 
            // },
            ETextureViewUsage::Tex(val) => {
                KeyTextureViewUsage::Tex(*val.key(), TextureRect { x: 0, y: 0, w: val.width as u16, h: val.height as u16 })
            },
            ETextureViewUsage::TexWithId(val) => {
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
            ETextureViewUsage::ImageFrame(val) => {
                KeyTextureViewUsage::ImageFrame(*val.key())
            },
            // ETextureViewUsage::Temp(_, key) => {
            //     KeyTextureViewUsage::Temp(*key, TextureRect::default()) 
            // },
        }
    }
    pub fn view(&self) -> &wgpu::TextureView {
        match self {
            ETextureViewUsage::Image(val) => &val.view,
            // ETextureViewUsage::Render(val) => val.view(),
            ETextureViewUsage::Tex(val) => &val.texture_view,
            ETextureViewUsage::TexWithId(val) => &val.texture_view,
            ETextureViewUsage::SRT(val) => &val.target().colors[0].0,
            ETextureViewUsage::ImageFrame(val) => &val.view,
            // ETextureViewUsage::Temp(val, _) => val,
        }
    }
    pub fn view_dimension(&self) -> wgpu::TextureViewDimension {
        match self {
            ETextureViewUsage::Image(val) => val.texture.image().view_dimension,
            // ETextureViewUsage::Render(val) => val.view(),
            ETextureViewUsage::Tex(val) => wgpu::TextureViewDimension::D2,
            ETextureViewUsage::TexWithId(val) => wgpu::TextureViewDimension::D2,
            ETextureViewUsage::SRT(val) => wgpu::TextureViewDimension::D2,
            ETextureViewUsage::ImageFrame(val) => val.texture.texture().view_dimension,
            // ETextureViewUsage::Temp(val, _) => val,
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
            // (Self::Render(l0), Self::Render(r0)) => l0 == r0,
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
// impl From<ERenderTargetViewUsage> for ETextureViewUsage {
//     fn from(value: ERenderTargetViewUsage) -> Self {
//         Self::Render(value)
//     }
// }
impl TAssetKeyU64 for ETextureViewUsage {}