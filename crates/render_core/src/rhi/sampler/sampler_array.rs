use std::sync::Arc;

use super::KeySampler;


pub type KeySamplerArray = EKeySamplerArray;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum EKeySamplerArray {
    N16(Arc<[KeySampler;16]>),
    N32(Arc<[KeySampler;32]>),
    N64(Arc<[KeySampler;64]>),
    N128(Arc<[KeySampler;128]>),
}
