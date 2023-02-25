use std::{num::NonZeroU8, sync::Arc, hash::Hash, fmt::Debug};

use pi_assets::asset::{Asset, Handle};

pub type KeySampler = SamplerDesc;

pub type KeySamplerArray = ESamplers;

pub struct SamplerRes(pub wgpu::Sampler);
impl Asset for SamplerRes {
    type Key = KeySampler;
    fn size(&self) -> usize {
        256
    }
}

#[derive(Clone)]
pub struct BindDataSampler(pub Handle<SamplerRes>);
impl Hash for BindDataSampler {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.key().hash(state);
    }
}
impl PartialEq for BindDataSampler {
    fn eq(&self, other: &Self) -> bool {
        self.0.key() == other.0.key()
    }
}
impl Eq for BindDataSampler {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}
impl Debug for BindDataSampler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BindDataSampler").field("key", &self.0.key()).finish()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ESamplers {
    N16(Arc<[KeySampler;16]>),
    N32(Arc<[KeySampler;32]>),
    N64(Arc<[KeySampler;64]>),
    N128(Arc<[KeySampler;128]>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum EAddressMode {
    /// Clamp the value to the edge of the texture
    ///
    /// -0.25 -> 0.0
    /// 1.25  -> 1.0
    ClampToEdge,
    /// Repeat the texture in a tiling fashion
    ///
    /// -0.25 -> 0.75
    /// 1.25 -> 0.25
    Repeat,
    /// Repeat the texture, mirroring it every repeat
    ///
    /// -0.25 -> 0.25
    /// 1.25 -> 0.75
    MirrorRepeat,
    /// Clamp the value to the border of the texture
    /// Requires feature [`Features::ADDRESS_MODE_CLAMP_TO_BORDER`]
    ///
    /// -0.25 -> border
    /// 1.25 -> border
    ClampToBorder,
}

impl Default for EAddressMode {
    fn default() -> Self {
        Self::ClampToEdge
    }
}
impl EAddressMode {
    fn mode(&self) -> wgpu::AddressMode {
        match self {
            EAddressMode::ClampToEdge   => wgpu::AddressMode::ClampToEdge   ,
            EAddressMode::Repeat        => wgpu::AddressMode::Repeat        ,
            EAddressMode::MirrorRepeat  => wgpu::AddressMode::MirrorRepeat  ,
            EAddressMode::ClampToBorder => wgpu::AddressMode::ClampToBorder ,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum EFilterMode {
    Nearest,
    Linear,
}
impl Default for EFilterMode {
    fn default() -> Self {
        Self::Nearest
    }
}
impl EFilterMode {
    fn mode(&self) -> wgpu::FilterMode {
        match self {
            EFilterMode::Nearest    => wgpu::FilterMode::Nearest,
            EFilterMode::Linear     => wgpu::FilterMode::Linear,
        }
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct SamplerDesc {
    /// How to deal with out of bounds accesses in the u (i.e. x) direction
    pub address_mode_u: EAddressMode,
    /// How to deal with out of bounds accesses in the v (i.e. y) direction
    pub address_mode_v: EAddressMode,
    /// How to deal with out of bounds accesses in the w (i.e. z) direction
    pub address_mode_w: EAddressMode,
    /// How to filter the texture when it needs to be magnified (made larger)
    pub mag_filter: EFilterMode,
    /// How to filter the texture when it needs to be minified (made smaller)
    pub min_filter: EFilterMode,
    /// How to filter between mip map levels
    pub mipmap_filter: EFilterMode,
    /// If this is enabled, this is a comparison sampler using the given comparison function.
    pub compare: Option<wgpu::CompareFunction>,
    /// Valid values: 1, 2, 4, 8, and 16.
    pub anisotropy_clamp: EAnisotropyClamp,
    /// Border color to use when address_mode is [`AddressMode::ClampToBorder`]
    pub border_color: Option<wgpu::SamplerBorderColor>,
}
impl SamplerDesc {
    pub fn size(&self) -> usize {
        20
    }
    pub fn to_sampler_description(&self) -> wgpu::SamplerDescriptor {
        wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: self.address_mode_u.mode(),
            address_mode_v: self.address_mode_v.mode(),
            address_mode_w: self.address_mode_w.mode(),
            mag_filter: self.mag_filter.mode(),
            min_filter: self.min_filter.mode(),
            mipmap_filter: self.mipmap_filter.mode(),
            compare: self.compare,
            anisotropy_clamp: self.anisotropy_clamp(),
            border_color: self.border_color,
            ..Default::default()
        }
    }
    pub fn anisotropy_clamp(&self) -> Option<NonZeroU8> {
        match self.anisotropy_clamp {
            EAnisotropyClamp::None      => None,
            EAnisotropyClamp::One       => NonZeroU8::new(1 ),
            EAnisotropyClamp::Two       => NonZeroU8::new(2 ),
            EAnisotropyClamp::Four      => NonZeroU8::new(4 ),
            EAnisotropyClamp::Eight     => NonZeroU8::new(8 ),
            EAnisotropyClamp::Sixteen   => NonZeroU8::new(16),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EAnisotropyClamp {
    None,
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}
impl Default for EAnisotropyClamp {
    fn default() -> Self {
        Self::None
    }
}
