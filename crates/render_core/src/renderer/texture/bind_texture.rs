use std::{sync::Arc, fmt::Debug, hash::Hash};
use std::ops::Deref;
use pi_assets::asset::Handle;
use pi_atom::Atom;

use crate::rhi::asset::TextureRes;
use crate::rhi::texture::TextureView;

use super::texture_view::ETextureViewUsage;


pub type KeyTexture = Atom;

pub type KeyTextureArray = ETextureArray;


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ETextureArray {
    N16(Arc<[KeyTexture;16]>),
    N32(Arc<[KeyTexture;32]>),
    N64(Arc<[KeyTexture;64]>),
    N128(Arc<[KeyTexture;128]>),
}
impl ETextureArray {
    pub fn count(&self) -> u32 {
        match self {
            ETextureArray::N16(_)   => 16,
            ETextureArray::N32(_)   => 32,
            ETextureArray::N64(_)   => 64,
            ETextureArray::N128(_)  => 128,
        }
    }
}

// #[derive(Clone, Hash)]
#[derive(Clone)]
pub struct BindDataTexture2D(pub ETextureViewUsage);
impl BindDataTexture2D {
    pub fn view(&self) -> &wgpu::TextureView {
        self.0.view()
    }
}
impl Hash for BindDataTexture2D {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.key().hash(state)
    }
}
impl PartialEq for BindDataTexture2D {
    fn eq(&self, other: &Self) -> bool {
        self.0.key() == other.0.key()
    }
}
impl Eq for BindDataTexture2D {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}
impl Debug for BindDataTexture2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BindDataTexture").field("key", &self.0.key()).finish()
    }
}
impl Deref for BindDataTexture2D {
    type Target = wgpu::TextureView;

    fn deref(&self) -> &Self::Target {
        self.0.view()
    }
}

#[derive(Clone)]
pub struct BindDataTextureArrayN<const N: usize>(pub [BindDataTexture2D;N]);
impl<const N: usize> BindDataTextureArrayN<N> {
    pub fn new(data: [BindDataTexture2D;N]) -> Self {
        // let mut views = vec![];
        // data.iter().for_each(|v| views.push(&v.0.texture_view));
        Self(data)
    }
    pub fn array<'a>(&'a self) -> Vec<&'a wgpu::TextureView> {
        let mut views = vec![];
        self.0.iter().for_each(|v| views.push(v.0.view()) );
        views
    }
}
impl<const N: usize> Hash for BindDataTextureArrayN<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl<const N: usize> PartialEq for BindDataTextureArrayN<N> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<const N: usize> Eq for BindDataTextureArrayN<N> {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}
impl<const N: usize> Debug for BindDataTextureArrayN<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BindDataTextureArrayN").field("Array", &self.0).finish()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BindDataTextureArray {
    N16(Arc<BindDataTextureArrayN<16>>),
    N32(Arc<BindDataTextureArrayN<32>>),
    N64(Arc<BindDataTextureArrayN<64>>),
    N128(Arc<BindDataTextureArrayN<128>>),
}
impl BindDataTextureArray {
    pub fn count(&self) -> u32 {
        match self {
            BindDataTextureArray::N16(_)   => 16,
            BindDataTextureArray::N32(_)   => 32,
            BindDataTextureArray::N64(_)   => 64,
            BindDataTextureArray::N128(_)  => 128,
        }
    }
    pub fn array<'a>(&'a self) -> Vec<&'a wgpu::TextureView> {
        match self {
            BindDataTextureArray::N16(v) => {
                v.array()
            },
            BindDataTextureArray::N32(v) =>  {
                v.array()
            },
            BindDataTextureArray::N64(v) =>  {
                v.array()
            },
            BindDataTextureArray::N128(v) =>  {
                v.array()
            },
        }
    }
}