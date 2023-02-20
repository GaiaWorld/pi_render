
use std::num::NonZeroU8;

use pi_assets::{asset::{Handle, Asset}, mgr::AssetMgr};
use pi_hash::XHashMap;
use render_core::rhi::{texture::Sampler, device::RenderDevice};
use render_shader::texture_sampler_code::SamplerDesc;

// #[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
// pub struct AssetKeySampler(pub SamplerDesc);

pub struct AssetSampler(pub Sampler);
impl Asset for AssetSampler {
    type Key = SamplerDesc;
    fn size(&self) -> usize {
        8
    }
}

pub type AssetMgrSampler = AssetMgr<AssetSampler>;

// #[derive(Debug, Default)]
// pub struct SamplerPool {
//     map: XHashMap<SamplerDesc, Sampler>,
// }
// impl SamplerPool {
//     pub fn create(
//         &mut self,
//         key: &SamplerDesc,
//         device: &RenderDevice,
//     ) {
//         if self.map.contains_key(&key) == false {
//             let sampler = device.create_sampler(
//                 &key.to_sampler_description()
//             );

//             self.map.insert(key.clone(), Sampler::from(sampler));
//         }
//     }
//     pub fn get(
//         &self,
//         key: SamplerDesc,
//     ) -> Option<Sampler> {
//         match self.map.get(&key) {
//             Some(value) => Some(value.clone()),
//             None => None,
//         }
//     }
//     // pub fn cacl_key(
//     //     value: &SamplerDesc,
//     // ) -> SamplerAssetKey {
//     //     let mut calcolator = KeyCalcolator::new();

//     //     Self::cacl_address_mode(&mut calcolator, value.address_mode_u, BYTE_ADDRESS_MODE);
//     //     Self::cacl_address_mode(&mut calcolator, value.address_mode_v, BYTE_ADDRESS_MODE);
//     //     Self::cacl_address_mode(&mut calcolator, value.address_mode_w, BYTE_ADDRESS_MODE);
        
//     //     Self::cacl_filter_mode(&mut calcolator, value.mag_filter, BYTE_FILTER_MODE);
//     //     Self::cacl_filter_mode(&mut calcolator, value.min_filter, BYTE_FILTER_MODE);
//     //     Self::cacl_filter_mode(&mut calcolator, value.mipmap_filter, BYTE_FILTER_MODE);
        
//     //     Self::cacl_compare(&mut calcolator, value.compare, BYTE_COMPARE);
        
//     //     Self::cacl_anisotropy(&mut calcolator, value.anisotropy_clamp, BYTE_ANISOTROPY);
        
//     //     Self::cacl_border(&mut calcolator, value.border_color, BYTE_BORDER_COLOR);

//     //     return calcolator.key;
//     // }
//     // fn cacl_address_mode(
//     //     calcolator: &mut KeyCalcolator,
//     //     value: wgpu::AddressMode,
//     //     use_byte: u8,
//     // ) {
//     //     let diff = SamplerAssetKey::pow(2, calcolator.use_bytes as u32);
//     //     calcolator.key += match value {
//     //         wgpu::AddressMode::ClampToEdge      => 0 * diff,
//     //         wgpu::AddressMode::Repeat           => 1 * diff,
//     //         wgpu::AddressMode::MirrorRepeat     => 2 * diff,
//     //         wgpu::AddressMode::ClampToBorder    => 3 * diff,
//     //     };
    
//     //     calcolator.use_bytes += use_byte;
//     // }
//     // fn cacl_filter_mode(
//     //     calcolator: &mut KeyCalcolator,
//     //     value: wgpu::FilterMode,
//     //     use_byte: u8,
//     // ) {
//     //     let diff = SamplerAssetKey::pow(2, calcolator.use_bytes as u32);
//     //     calcolator.key += match value {
//     //         wgpu::FilterMode::Nearest       => 0 * diff,
//     //         wgpu::FilterMode::Linear        => 1 * diff,
//     //     };
    
//     //     calcolator.use_bytes += use_byte;
//     // }
//     // fn cacl_compare(
//     //     calcolator: &mut KeyCalcolator,
//     //     value: Option<wgpu::CompareFunction>,
//     //     use_byte: u8,
//     // ) {
//     //     let diff = SamplerAssetKey::pow(2, calcolator.use_bytes as u32);
//     //     calcolator.key += match value {
//     //         Some(value) => {
//     //             match value {
//     //                 wgpu::CompareFunction::Never        => 1 * diff,
//     //                 wgpu::CompareFunction::Less         => 2 * diff,
//     //                 wgpu::CompareFunction::Equal        => 3 * diff,
//     //                 wgpu::CompareFunction::LessEqual    => 4 * diff,
//     //                 wgpu::CompareFunction::Greater      => 5 * diff,
//     //                 wgpu::CompareFunction::NotEqual     => 6 * diff,
//     //                 wgpu::CompareFunction::GreaterEqual => 7 * diff,
//     //                 wgpu::CompareFunction::Always       => 8 * diff,
//     //             }
//     //         },
//     //         _ => 0,
//     //     };
    
//     //     calcolator.use_bytes += use_byte;
//     // }
//     // fn cacl_anisotropy(
//     //     calcolator: &mut KeyCalcolator,
//     //     value: EAnisotropyClamp,
//     //     use_byte: u8,
//     // ) {
//     //     let diff = SamplerAssetKey::pow(2, calcolator.use_bytes as u32);
//     //     calcolator.key += match value {
//     //         EAnisotropyClamp::None      => 0 * diff,
//     //         EAnisotropyClamp::One       => 1 * diff,
//     //         EAnisotropyClamp::Two       => 2 * diff,
//     //         EAnisotropyClamp::Four      => 3 * diff,
//     //         EAnisotropyClamp::Eight     => 4 * diff,
//     //         EAnisotropyClamp::Sixteen   => 5 * diff,
//     //     };
    
//     //     calcolator.use_bytes += use_byte;
//     // }
//     // fn cacl_border(
//     //     calcolator: &mut KeyCalcolator,
//     //     value: Option<wgpu::SamplerBorderColor>,
//     //     use_byte: u8,
//     // ) {
//     //     let diff = SamplerAssetKey::pow(2, calcolator.use_bytes as u32);
//     //     calcolator.key += match value {
//     //         Some(value) => {
//     //             match value {
//     //                 wgpu::SamplerBorderColor::TransparentBlack => 1 * diff,
//     //                 wgpu::SamplerBorderColor::OpaqueBlack => 2 * diff,
//     //                 wgpu::SamplerBorderColor::OpaqueWhite => 3 * diff,
//     //                 wgpu::SamplerBorderColor::Zero => 4 * diff,
//     //             }
//     //         },
//     //         _ => 0,
//     //     };
    
//     //     calcolator.use_bytes += use_byte;
//     // }
// }

// pub struct AssetSampler(pub wgpu::Sampler);

// impl Asset for AssetSampler {
//     type Key = SamplerAssetKey;

//     fn size(&self) -> usize {
//         128
//     }
// }
