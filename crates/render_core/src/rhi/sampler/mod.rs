use std::{hash::Hash, fmt::Debug};

use pi_assets::asset::{Asset, Size};

use super::device::RenderDevice;

pub mod sampler_array;

pub type KeySampler = SamplerDesc;

pub struct Sampler(pub crate::rhi::texture::Sampler);
impl Asset for Sampler {
    type Key = KeySampler;
    // const TYPE: &'static str = "Sampler";
}

impl Size for Sampler {
    fn size(&self) -> usize {
        256
    }
}

impl Sampler {
    pub fn new(device: &RenderDevice, key: &KeySampler) -> Self {
        Self(
            device.create_sampler(
                &key.to_sampler_description()
            )
        )
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
    pub fn mode(&self) -> wgpu::AddressMode {
        match self {
            EAddressMode::ClampToEdge   => wgpu::AddressMode::ClampToEdge   ,
            EAddressMode::Repeat        => wgpu::AddressMode::Repeat        ,
            EAddressMode::MirrorRepeat  => wgpu::AddressMode::MirrorRepeat  ,
            EAddressMode::ClampToBorder => wgpu::AddressMode::ClampToBorder ,
        }
    }
    pub fn to_u8(&self) -> u8 {
        match self {
            EAddressMode::ClampToEdge   => 0 ,
            EAddressMode::Repeat        => 1 ,
            EAddressMode::MirrorRepeat  => 2 ,
            EAddressMode::ClampToBorder => 3 ,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
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
        32
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
    pub fn anisotropy_clamp(&self) -> u16 {
        match self.anisotropy_clamp {
            EAnisotropyClamp::None      => 1,
            EAnisotropyClamp::One       => 1,
            EAnisotropyClamp::Two       => 2,
            EAnisotropyClamp::Four      => 4,
            EAnisotropyClamp::Eight     => 8,
            EAnisotropyClamp::Sixteen   => 16,
        }
    }
    pub fn linear_clamp() -> Self {
        Self {
            address_mode_u: EAddressMode::ClampToEdge,
            address_mode_v: EAddressMode::ClampToEdge,
            address_mode_w: EAddressMode::ClampToEdge,
            mag_filter: EFilterMode::Linear,
            min_filter: EFilterMode::Linear,
            mipmap_filter: EFilterMode::Nearest,
            compare: None,
            anisotropy_clamp: EAnisotropyClamp::One,
            border_color: None,
        }
    }
    pub fn linear_repeat() -> Self {
        Self {
            address_mode_u: EAddressMode::Repeat,
            address_mode_v: EAddressMode::Repeat,
            address_mode_w: EAddressMode::Repeat,
            mag_filter: EFilterMode::Linear,
            min_filter: EFilterMode::Linear,
            mipmap_filter: EFilterMode::Nearest,
            compare: None,
            anisotropy_clamp: EAnisotropyClamp::One,
            border_color: None,
        }
    }
    pub fn nearest_clamp() -> Self {
        Self {
            address_mode_u: EAddressMode::ClampToEdge,
            address_mode_v: EAddressMode::ClampToEdge,
            address_mode_w: EAddressMode::ClampToEdge,
            mag_filter: EFilterMode::Nearest,
            min_filter: EFilterMode::Nearest,
            mipmap_filter: EFilterMode::Nearest,
            compare: None,
            anisotropy_clamp: EAnisotropyClamp::None,
            border_color: None,
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
