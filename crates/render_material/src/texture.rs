
use std::hash;

use pi_slotmap::DefaultKey;
pub struct OffsetScaleTexture2D {
    pub u_scale: f32,
    pub v_scale: f32,
    pub u_offset: f32,
    pub v_offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EAnisotropyClamp {
    One = 1,
    Two = 2,
    Four = 4,
    Eight = 8,
    Sixteen = 16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MaterialTextureSampler {
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
    /// Minimum level of detail (i.e. mip level) to use
    pub lod_min_clamp: f32,
    /// Maximum level of detail (i.e. mip level) to use
    pub lod_max_clamp: f32,
    /// If this is enabled, this is a comparison sampler using the given comparison function.
    pub compare: Option<wgpu::CompareFunction>,
    /// Valid values: 1, 2, 4, 8, and 16.
    pub anisotropy_clamp: Option<std::num::NonZeroU8>,
    /// Border color to use when address_mode is [`AddressMode::ClampToBorder`]
    pub border_color: Option<wgpu::SamplerBorderColor>,
}

impl Default for MaterialTextureSampler {
    fn default() -> Self {
        MaterialTextureSampler::new(wgpu::AddressMode::ClampToEdge, wgpu::FilterMode::Linear)
    }
}

impl MaterialTextureSampler {
    pub fn new(address: wgpu::AddressMode, filter: wgpu::FilterMode) -> Self {
        Self {
            address_mode_u: address,
            address_mode_v: address,
            address_mode_w: address,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: filter,
            lod_min_clamp: 0.0,
            lod_max_clamp: std::f32::MAX,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        }
    }
    pub fn from_sampler_desc(sampler: &wgpu::SamplerDescriptor) -> Self {
        Self {
            address_mode_u: sampler.address_mode_u,
            address_mode_v: sampler.address_mode_v,
            address_mode_w: sampler.address_mode_w,
            mag_filter: sampler.mag_filter,
            min_filter: sampler.min_filter,
            mipmap_filter: sampler.mipmap_filter,
            lod_min_clamp: sampler.lod_min_clamp,
            lod_max_clamp: sampler.lod_max_clamp,
            compare: sampler.compare,
            anisotropy_clamp: sampler.anisotropy_clamp,
            border_color: sampler.border_color,
        }
    }
    pub fn to_sampler_resource(
        &self,
        device: & wgpu::Device,
        label: Option<&str>,
    ) -> wgpu::Sampler {
        device.create_sampler(
            &wgpu::SamplerDescriptor {
                label,
                address_mode_u: self.address_mode_u,
                address_mode_v: self.address_mode_v,
                address_mode_w: self.address_mode_w,
                mag_filter: self.mag_filter,
                min_filter: self.min_filter,
                mipmap_filter: self.mipmap_filter,
                lod_min_clamp: self.lod_min_clamp,
                lod_max_clamp: self.lod_max_clamp,
                compare: self.compare,
                anisotropy_clamp: self.anisotropy_clamp.clone(),
                border_color: self.border_color,
            }
        )
    }
    pub fn is_same(
        &self,
        other: &Self
    ) -> bool {
        other.address_mode_u == self.address_mode_u
        && other.address_mode_v == self.address_mode_v
        && other.address_mode_w == self.address_mode_w
        && other.mag_filter == self.mag_filter
        && other.min_filter == self.min_filter
        && other.mipmap_filter == self.mipmap_filter
        && other.lod_min_clamp == self.lod_min_clamp
        && other.lod_max_clamp == self.lod_max_clamp
        && other.compare == self.compare
        && other.anisotropy_clamp == self.anisotropy_clamp.clone()
        && other.border_color == self.border_color
    }
}