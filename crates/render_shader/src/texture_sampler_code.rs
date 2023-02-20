use std::num::NonZeroU8;

pub type SamplerAssetKey = SamplerDesc;

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct SamplerDesc {
    /// How to deal with out of bounds accesses in the u (i.e. x) direction
    pub address_mode_u: wgpu::AddressMode,
    /// How to deal with out of bounds accesses in the v (i.e. y) direction
    pub address_mode_v: wgpu::AddressMode,
    /// How to deal with out of bounds accesses in the w (i.e. z) direction
    pub address_mode_w: wgpu::AddressMode,
    /// How to filter the texture when it needs to be magnified (made larger)
    pub mag_filter: wgpu::FilterMode,
    /// How to filter the texture when it needs to be minified (made smaller)
    pub min_filter: wgpu::FilterMode,
    /// How to filter between mip map levels
    pub mipmap_filter: wgpu::FilterMode,
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
            address_mode_u: self.address_mode_u,
            address_mode_v: self.address_mode_v,
            address_mode_w: self.address_mode_w,
            mag_filter: self.mag_filter,
            min_filter: self.min_filter,
            mipmap_filter: self.mipmap_filter,
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
