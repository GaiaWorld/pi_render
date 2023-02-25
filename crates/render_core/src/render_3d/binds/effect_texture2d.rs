use std::sync::Arc;

use derive_deref_rs::Deref;
use pi_assets::asset::Handle;

use crate::{
    renderer::{
        texture::BindDataTexture2D,
        bind::{KeyBindLayoutTexture2D, EKeyBind, KeyBindTexture2D},
    },
    rhi::{asset::TextureRes},
    render_3d::shader::{shader_effect_meta::ShaderEffectMeta, TBindDescToShaderCode}};


pub trait TEffectBindTexture2D {
    const INDEX: usize;
    fn vs_define_code(&self, meta: &ShaderEffectMeta, set: u32, bind: u32) -> String { vs_define(meta, Self::INDEX, set, bind) }
    fn fs_define_code(&self, meta: &ShaderEffectMeta, set: u32, bind: u32) -> String { fs_define(meta, Self::INDEX, set, bind) }
}

pub trait TEffectBindTexture2DData: TEffectBindTexture2D {
    fn data(&self) -> &BindDataTexture2D;
    fn key_bind(&self, meta: &ShaderEffectMeta, binding: u16) -> Option<EKeyBind> {
        if let Some(layout) = self.key_layout(meta, binding) { Some(EKeyBind::Texture2D(KeyBindTexture2D { data: self.data().clone(), layout: Arc::new(layout)  })) } else { None }
    }
    fn key_layout(&self, meta: &ShaderEffectMeta, binding: u16) -> Option<KeyBindLayoutTexture2D> {
        if let Some(desc) = meta.textures.get(Self::INDEX) { Some(KeyBindLayoutTexture2D { binding, visibility: desc.stage, texture_sample_type: desc.tex_sampler_type, }) } else { None }
    }
}

#[derive(Debug, Clone, Deref)]
pub struct EffectBindTexture2D01(pub BindDataTexture2D);
impl From<Handle<TextureRes>> for EffectBindTexture2D01 {
    fn from(value: Handle<TextureRes>) -> Self { Self( BindDataTexture2D(value) ) }
}
impl TEffectBindTexture2D for EffectBindTexture2D01 {
    const INDEX: usize = 1 - 1;
}
impl TEffectBindTexture2DData for EffectBindTexture2D01 {
    fn data(&self) -> &BindDataTexture2D { &self }
}

#[derive(Debug, Clone, Deref)]
pub struct EffectBindTexture2D02(pub BindDataTexture2D);
impl From<Handle<TextureRes>> for EffectBindTexture2D02 {
    fn from(value: Handle<TextureRes>) -> Self { Self( BindDataTexture2D(value) ) }
}
impl TEffectBindTexture2D for EffectBindTexture2D02 {
    const INDEX: usize = 2 - 1;
}
impl TEffectBindTexture2DData for EffectBindTexture2D02 {
    fn data(&self) -> &BindDataTexture2D { &self }
}


#[derive(Debug, Clone, Deref)]
pub struct EffectBindTexture2D03(pub BindDataTexture2D);
impl From<Handle<TextureRes>> for EffectBindTexture2D03 {
    fn from(value: Handle<TextureRes>) -> Self { Self( BindDataTexture2D(value) ) }
}
impl TEffectBindTexture2D for EffectBindTexture2D03 {
    const INDEX: usize = 3 - 1;
}
impl TEffectBindTexture2DData for EffectBindTexture2D03 {
    fn data(&self) -> &BindDataTexture2D { &self }
}


#[derive(Debug, Clone, Deref)]
pub struct EffectBindTexture2D04(pub BindDataTexture2D);
impl From<Handle<TextureRes>> for EffectBindTexture2D04 {
    fn from(value: Handle<TextureRes>) -> Self { Self( BindDataTexture2D(value) ) }
}
impl TEffectBindTexture2D for EffectBindTexture2D04 {
    const INDEX: usize = 4 - 1;
}
impl TEffectBindTexture2DData for EffectBindTexture2D04 {
    fn data(&self) -> &BindDataTexture2D { &self }
}


#[derive(Debug, Clone, Deref)]
pub struct EffectBindTexture2D05(pub BindDataTexture2D);
impl From<Handle<TextureRes>> for EffectBindTexture2D05 {
    fn from(value: Handle<TextureRes>) -> Self { Self( BindDataTexture2D(value) ) }
}
impl TEffectBindTexture2D for EffectBindTexture2D05 {
    const INDEX: usize = 5 - 1;
}
impl TEffectBindTexture2DData for EffectBindTexture2D05 {
    fn data(&self) -> &BindDataTexture2D { &self }
}


#[derive(Debug, Clone, Deref)]
pub struct EffectBindTexture2D06(pub BindDataTexture2D);
impl From<Handle<TextureRes>> for EffectBindTexture2D06 {
    fn from(value: Handle<TextureRes>) -> Self { Self( BindDataTexture2D(value) ) }
}
impl TEffectBindTexture2D for EffectBindTexture2D06 {
    const INDEX: usize = 6 - 1;
}
impl TEffectBindTexture2DData for EffectBindTexture2D06 {
    fn data(&self) -> &BindDataTexture2D { &self }
}
    
fn vs_define(meta: &ShaderEffectMeta, index: usize, set: u32, bind: u32) -> String {
    let mut result = String::from("");
    
    if let Some(desc) = meta.textures.get(index) {
        if  desc.stage.mode() & wgpu::ShaderStages::VERTEX == wgpu::ShaderStages::VERTEX {
            result += desc.vs_code(set, bind).as_str();
        }
    }

    result
}
fn fs_define(meta: &ShaderEffectMeta, index: usize, set: u32, bind: u32) -> String {
    let mut result = String::from("");

    if let Some(desc) = meta.textures.get(index) {
        if  desc.stage.mode() & wgpu::ShaderStages::FRAGMENT == wgpu::ShaderStages::FRAGMENT {
            result += desc.fs_code(set, bind).as_str();
        }
    }

    result
}