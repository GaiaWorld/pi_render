use std::num::NonZeroU64;

use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_atom::Atom;
use render_core::rhi::{buffer::BufferId, asset::TextureRes};
use render_shader::texture_sampler_code::SamplerDesc;

use crate::{buffer::dyn_mergy_buffer::DynMergyBufferRange, sampler::{AssetSampler, AssetMgrSampler}};

pub trait TBindValue {
    fn buffer_info(&self) -> &DynMergyBufferRange;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyBindBuffer {
    pub(crate) bind: u32,
    pub(crate) id_buffer: DynMergyBufferRange,
    pub(crate) entry: wgpu::BindGroupLayoutEntry,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyBindTexture {
    pub(crate) bind: u32,
    pub(crate) id_texture: Atom,
    pub(crate) entry: wgpu::BindGroupLayoutEntry,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyBindSampler {
    pub(crate) bind: u32,
    pub(crate) id_sampler: SamplerDesc,
    pub(crate) entry: wgpu::BindGroupLayoutEntry,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyBind {
    Buffer(KeyBindBuffer),
    Texture(KeyBindTexture),
    Sampler(KeyBindSampler),
}
impl KeyBind {
    pub fn layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
        match self {
            KeyBind::Buffer(val) => val.entry.clone(),
            KeyBind::Texture(val) => val.entry.clone(),
            KeyBind::Sampler(val) => val.entry.clone(),
        }
    }
}
pub trait TKeyBind {
    fn key_bind(&self) -> KeyBind;
}

#[derive(Clone)]
pub enum ERenderBind {
    Buffer(u32, DynMergyBufferRange),
    Texture(u32, Handle<TextureRes>),
    Sampler(u32, Handle<AssetSampler>),
}