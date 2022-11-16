
use std::num::NonZeroU8;

use pi_hash::XHashMap;
use render_core::rhi::{texture::Sampler, device::RenderDevice};

pub type SamplerKey = u64;

#[derive(Debug, Default)]
pub struct SamplerDesc {
    /// How to deal with out of bounds accesses in the u (i.e. x) direction
    pub address_mode_u: AddressMode,
    /// How to deal with out of bounds accesses in the v (i.e. y) direction
    pub address_mode_v: AddressMode,
    /// How to deal with out of bounds accesses in the w (i.e. z) direction
    pub address_mode_w: AddressMode,
    /// How to filter the texture when it needs to be magnified (made larger)
    pub mag_filter: FilterMode,
    /// How to filter the texture when it needs to be minified (made smaller)
    pub min_filter: FilterMode,
    /// How to filter between mip map levels
    pub mipmap_filter: FilterMode,
    /// If this is enabled, this is a comparison sampler using the given comparison function.
    pub compare: Option<CompareFunction>,
    /// Valid values: 1, 2, 4, 8, and 16.
    pub anisotropy_clamp: EAnisotropyClamp,
    /// Border color to use when address_mode is [`AddressMode::ClampToBorder`]
    pub border_color: Option<SamplerBorderColor>,
}
impl SamplerDesc {
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
            EAnisotropyClamp::One       => Some(1),
            EAnisotropyClamp::Two       => Some(2),
            EAnisotropyClamp::Four      => Some(4),
            EAnisotropyClamp::Eight     => Some(8),
            EAnisotropyClamp::Sixteen   => Some(16),
        }
    }
}

const BYTE_ADDRESS_MODE: u8 = 2;
const BYTE_FILTER_MODE: u8 = 2;
const BYTE_COMPARE: u8 = 4;
const BYTE_ANISOTROPY: u8 = 3;
const BYTE_BORDER_COLOR: u8 = 3;

struct KeyCalcolator {
    pub key: SamplerKey,
    pub use_bytes: u8,
}

impl KeyCalcolator {
    pub fn new() -> Self {
        Self { key: 0, use_bytes: 0 }
    }
}

pub enum EAnisotropyClamp {
    None,
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}

#[derive(Debug, Default)]
pub struct SamplerPool {
    map: XHashMap<SamplerKey, Sampler>,
}
impl SamplerPool {
    pub fn create(
        &mut self,
        value: &SamplerDesc,
        device: &RenderDevice,
    ) {
        let key = Self::cacl_key(value);

        if self.map.contains_key(&key) == false {
            let sampler = device.create_sampler(
                &value.to_sampler_description()
            );

            self.map.insert(key, Sampler::from(sampler));
        }
    }
    pub fn get(
        &self,
        key: SamplerKey,
    ) -> Option<&Sampler> {
        self.map.get(&key)
    }
    pub fn cacl_key(
        value: &SamplerDesc,
    ) -> SamplerKey {
        let mut calcolator = KeyCalcolator::new();

        Self::cacl_address_mode(&mut calcolator, value.address_mode_u, BYTE_ADDRESS_MODE);
        Self::cacl_address_mode(&mut calcolator, value.address_mode_v, BYTE_ADDRESS_MODE);
        Self::cacl_address_mode(&mut calcolator, value.address_mode_w, BYTE_ADDRESS_MODE);
        
        Self::cacl_filter_mode(&mut calcolator, value.mag_filter, BYTE_FILTER_MODE);
        Self::cacl_filter_mode(&mut calcolator, value.min_filter, BYTE_FILTER_MODE);
        Self::cacl_filter_mode(&mut calcolator, value.mipmap_filter, BYTE_FILTER_MODE);
        
        Self::cacl_compare(&mut calcolator, value.compare, BYTE_COMPARE);
        
        Self::cacl_address_mode(&mut calcolator, value.anisotropy_clamp, BYTE_ANISOTROPY);
        
        Self::cacl_address_mode(&mut calcolator, value.border_color, BYTE_BORDER_COLOR);

        return calcolator.key;
    }
    fn cacl_address_mode(
        calcolator: &mut KeyCalcolator,
        value: wgpu::AddressMode,
        use_byte: u8,
    ) {
        let diff = SamplerKey::pow(2, calcolator.use_bytes as u32);
        calcolator.key += match value {
            wgpu::AddressMode::ClampToEdge      => 0 * diff,
            wgpu::AddressMode::Repeat           => 1 * diff,
            wgpu::AddressMode::MirrorRepeat     => 2 * diff,
            wgpu::AddressMode::ClampToBorder    => 3 * diff,
        };
    
        calcolator.use_bytes += use_byte;
    }
    fn cacl_filter_mode(
        calcolator: &mut KeyCalcolator,
        value: wgpu::FilterMode,
        use_byte: u8,
    ) {
        let diff = SamplerKey::pow(2, calcolator.use_bytes as u32);
        calcolator.key += match value {
            wgpu::FilterMode::Nearest       => 0 * diff,
            wgpu::FilterMode::Linear        => 1 * diff,
        };
    
        calcolator.use_bytes += use_byte;
    }
    fn cacl_compare(
        calcolator: &mut KeyCalcolator,
        value: Option<wgpu::CompareFunction>,
        use_byte: u8,
    ) {
        let diff = SamplerKey::pow(2, calcolator.use_bytes as u32);
        calcolator.key += match value {
            Some(value) => {
                match value {
                    wgpu::CompareFunction::Never        => 1 * diff,
                    wgpu::CompareFunction::Less         => 2 * diff,
                    wgpu::CompareFunction::Equal        => 3 * diff,
                    wgpu::CompareFunction::LessEqual    => 4 * diff,
                    wgpu::CompareFunction::Greater      => 5 * diff,
                    wgpu::CompareFunction::NotEqual     => 6 * diff,
                    wgpu::CompareFunction::GreaterEqual => 7 * diff,
                    wgpu::CompareFunction::Always       => 8 * diff,
                }
            },
            _ => 0,
        };
    
        calcolator.use_bytes += use_byte;
    }
    fn cacl_anisotropy(
        calcolator: &mut KeyCalcolator,
        value: EAnisotropyClamp,
        use_byte: u8,
    ) {
        let diff = SamplerKey::pow(2, calcolator.use_bytes as u32);
        calcolator.key += match value {
            EAnisotropyClamp::None      => 0 * diff,
            EAnisotropyClamp::One       => 1 * diff,
            EAnisotropyClamp::Two       => 2 * diff,
            EAnisotropyClamp::Four      => 3 * diff,
            EAnisotropyClamp::Eight     => 4 * diff,
            EAnisotropyClamp::Sixteen   => 5 * diff,
        };
    
        calcolator.use_bytes += use_byte;
    }
    fn cacl_border(
        calcolator: &mut KeyCalcolator,
        value: Option<wgpu::SamplerBorderColor>,
        use_byte: u8,
    ) {
        let diff = SamplerKey::pow(2, calcolator.use_bytes as u32);
        calcolator.key += match value {
            Some(value) => {
                match value {
                    wgpu::SamplerBorderColor::TransparentBlack => 1 * diff,
                    wgpu::SamplerBorderColor::OpaqueBlack => 2 * diff,
                    wgpu::SamplerBorderColor::OpaqueWhite => 3 * diff,
                    wgpu::SamplerBorderColor::Zero => 4 * diff,
                }
            },
            _ => 0,
        };
    
        calcolator.use_bytes += use_byte;
    }
}