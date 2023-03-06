use std::sync::Arc;

use derive_deref_rs::Deref;
use pi_assets::asset::Handle;

use crate::{
    renderer::{
        sampler::{BindDataSampler, SamplerRes},
        bind::{KeyBindLayoutSampler, EKeyBind, KeyBindSampler}
    },
    render_3d::shader::{shader_effect_meta::ShaderEffectMeta, uniform_sampler::sampler_code}
};

pub trait TEffectBindSampler2D {
    const INDEX: usize;
    fn vs_define_code(&self, meta: &ShaderEffectMeta, set: u32, bind: u32) -> String { vs_define(meta, Self::INDEX, set, bind) }
    fn fs_define_code(&self, meta: &ShaderEffectMeta, set: u32, bind: u32) -> String { fs_define(meta, Self::INDEX, set, bind) }
}
pub trait TEffectBindSampler2DData: TEffectBindSampler2D {
    fn data(&self) -> &BindDataSampler;
    fn key_bind(&self, meta: &ShaderEffectMeta, binding: u16) -> Option<EKeyBind> {
        if let Some(layout) = self.key_layout(meta, binding) { Some(EKeyBind::Sampler(KeyBindSampler { data: self.data().clone(), layout: Arc::new(layout)  })) } else { None }
    }
    fn key_layout(&self, meta: &ShaderEffectMeta, binding: u16) -> Option<KeyBindLayoutSampler> {
        if let Some(desc) = meta.textures.get(Self::INDEX) { Some(KeyBindLayoutSampler { binding, visibility: desc.stage, binding_type: desc.sampler_type() }) } else { None }
    }
}

#[derive(Debug, Clone, Deref, Hash, PartialEq, Eq)]
pub struct EffectBindSampler2D01(pub BindDataSampler);
impl From<Handle<SamplerRes>> for EffectBindSampler2D01 {
    fn from(value: Handle<SamplerRes>) -> Self { Self( BindDataSampler(value) ) }
}
impl TEffectBindSampler2D for EffectBindSampler2D01 {
    const INDEX: usize = 1 - 1;
}
impl TEffectBindSampler2DData for EffectBindSampler2D01 {
    fn data(&self) -> &BindDataSampler { &self }
}

#[derive(Debug, Clone, Deref, Hash, PartialEq, Eq)]
pub struct EffectBindSampler2D02(pub BindDataSampler);
impl From<Handle<SamplerRes>> for EffectBindSampler2D02 {
    fn from(value: Handle<SamplerRes>) -> Self { Self( BindDataSampler(value) ) }
}
impl TEffectBindSampler2D for EffectBindSampler2D02 {
    const INDEX: usize = 2 - 1;
}
impl TEffectBindSampler2DData for EffectBindSampler2D02 {
    fn data(&self) -> &BindDataSampler { &self }
}


#[derive(Debug, Clone, Deref, Hash, PartialEq, Eq)]
pub struct EffectBindSampler2D03(pub BindDataSampler);
impl From<Handle<SamplerRes>> for EffectBindSampler2D03 {
    fn from(value: Handle<SamplerRes>) -> Self { Self( BindDataSampler(value) ) }
}
impl TEffectBindSampler2D for EffectBindSampler2D03 {
    const INDEX: usize = 3 - 1;
}
impl TEffectBindSampler2DData for EffectBindSampler2D03 {
    fn data(&self) -> &BindDataSampler { &self }
}


#[derive(Debug, Clone, Deref, Hash, PartialEq, Eq)]
pub struct EffectBindSampler2D04(pub BindDataSampler);
impl From<Handle<SamplerRes>> for EffectBindSampler2D04 {
    fn from(value: Handle<SamplerRes>) -> Self { Self( BindDataSampler(value) ) }
}
impl TEffectBindSampler2D for EffectBindSampler2D04 {
    const INDEX: usize = 4 - 1;
}
impl TEffectBindSampler2DData for EffectBindSampler2D04 {
    fn data(&self) -> &BindDataSampler { &self }
}


#[derive(Debug, Clone, Deref, Hash, PartialEq, Eq)]
pub struct EffectBindSampler2D05(pub BindDataSampler);
impl From<Handle<SamplerRes>> for EffectBindSampler2D05 {
    fn from(value: Handle<SamplerRes>) -> Self { Self( BindDataSampler(value) ) }
}
impl TEffectBindSampler2D for EffectBindSampler2D05 {
    const INDEX: usize = 5 - 1;
}
impl TEffectBindSampler2DData for EffectBindSampler2D05 {
    fn data(&self) -> &BindDataSampler { &self }
}


#[derive(Debug, Clone, Deref, Hash, PartialEq, Eq)]
pub struct EffectBindSampler2D06(pub BindDataSampler);
impl From<Handle<SamplerRes>> for EffectBindSampler2D06 {
    fn from(value: Handle<SamplerRes>) -> Self { Self( BindDataSampler(value) ) }
}
impl TEffectBindSampler2D for EffectBindSampler2D06 {
    const INDEX: usize = 6 - 1;
}
impl TEffectBindSampler2DData for EffectBindSampler2D06 {
    fn data(&self) -> &BindDataSampler { &self }
}
    
fn vs_define(meta: &ShaderEffectMeta, index: usize, set: u32, bind: u32) -> String {
    let mut result = String::from("");
    
    if let Some(desc) = meta.textures.get(index) {
        if  desc.stage.mode() & wgpu::ShaderStages::VERTEX == wgpu::ShaderStages::VERTEX {
            result += sampler_code(desc.slotname.as_str(), desc.sampler_type(), set, bind).as_str();
        }
    }

    result
}
fn fs_define(meta: &ShaderEffectMeta, index: usize, set: u32, bind: u32) -> String {
    let mut result = String::from("");

    if let Some(desc) = meta.textures.get(index) {
        if  desc.stage.mode() & wgpu::ShaderStages::FRAGMENT == wgpu::ShaderStages::FRAGMENT {
            result += sampler_code(desc.slotname.as_str(), desc.sampler_type(), set, bind).as_str();
        }
    }

    result
}