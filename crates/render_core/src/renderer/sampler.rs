use std::{hash::Hash, fmt::Debug};

use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_share::Share;

use crate::rhi::{device::RenderDevice, sampler::{Sampler, sampler_array::EKeySamplerArray, SamplerDesc}};

pub type KeySampler = SamplerDesc;

pub type KeySamplerArray = EKeySamplerArray;

pub type SamplerRes = Sampler;

#[derive(Clone)]
pub struct BindDataSampler(pub Handle<SamplerRes>);
impl BindDataSampler {
    pub fn create(key: KeySampler, device: &RenderDevice, asset: &Share<AssetMgr<SamplerRes>>) -> Option<Self> {
        if let Some(val) = asset.get(&key) {
            Some(Self(val))
        } else {
            let samp = Sampler::new(device, &key);
            if let Ok(samp) = asset.insert(key, samp) {
                Some(Self(samp))
            } else {
                None
            }
        }
    }
}
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
