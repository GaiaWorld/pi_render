

use std::hash::Hash;

use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_share::Share;

use crate::{
    renderer::{
        bind_group::{BindGroupUsage, BindGroupLayout, KeyBindGroup, BindGroup},
        shader::TShaderSetBlock,
    },
    render_3d::{
        binds::{
            effect_texture2d::{EffectBindTexture2D01, EffectBindTexture2D02, EffectBindTexture2D03, EffectBindTexture2D04, EffectBindTexture2D05, EffectBindTexture2D06, TEffectBindTexture2D, TEffectBindTexture2DData},
            effect_sampler2d::{EffectBindSampler2D01, EffectBindSampler2D02, EffectBindSampler2D03, EffectBindSampler2D04, EffectBindSampler2D05, EffectBindSampler2D06, TEffectBindSampler2D, TEffectBindSampler2DData}
        },
        shader::shader_effect_meta::{ShaderEffectMeta}
    },
    rhi::{device::RenderDevice}
};


#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderSetTextureSamplers {

}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct EffectTextureSamplers {
    pub textures: (
        Option<EffectBindTexture2D01>, Option<EffectBindTexture2D02>, Option<EffectBindTexture2D03>,
        Option<EffectBindTexture2D04>, Option<EffectBindTexture2D05>, Option<EffectBindTexture2D06>,
    ),
    pub samplers: (
        Option<EffectBindSampler2D01>, Option<EffectBindSampler2D02>, Option<EffectBindSampler2D03>,
        Option<EffectBindSampler2D04>, Option<EffectBindSampler2D05>, Option<EffectBindSampler2D06>,
    ),
}

#[derive(Debug, Clone)]
pub struct KeyBindGroupTextureSamplers {
    pub key: KeyShaderSetTextureSamplers,
    pub effect_texture_samplers: EffectTextureSamplers,
    pub meta: Handle<ShaderEffectMeta>,
}
impl KeyBindGroupTextureSamplers {
    pub fn new(
        key: KeyShaderSetTextureSamplers,
        effect_texture_samplers: EffectTextureSamplers,
        meta: Handle<ShaderEffectMeta>,
    ) -> Self {
        Self { key, effect_texture_samplers, meta  }
    }
    pub fn key_bind_group(&self) -> Option<KeyBindGroup> {
        let mut binds = BindGroupUsage::none_binds();

        let mut binding = 0;

        if let Some(val) = &self.effect_texture_samplers.textures.0 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &self.effect_texture_samplers.textures.1 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &self.effect_texture_samplers.textures.2 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &self.effect_texture_samplers.textures.3 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &self.effect_texture_samplers.textures.4 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &self.effect_texture_samplers.textures.5 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        
        if let Some(val) = &self.effect_texture_samplers.samplers.0 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &self.effect_texture_samplers.samplers.1 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &self.effect_texture_samplers.samplers.2 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &self.effect_texture_samplers.samplers.3 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &self.effect_texture_samplers.samplers.4 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &self.effect_texture_samplers.samplers.5 {
            if let Some(layout) = val.key_bind(&self.meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }

        if binding == 0 {
            None
        } else {
            Some(KeyBindGroup(binds))
        }
    }
}
impl Hash for KeyBindGroupTextureSamplers {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
        self.effect_texture_samplers.hash(state);
        self.meta.key().hash(state);
    }
}
impl PartialEq for KeyBindGroupTextureSamplers {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.effect_texture_samplers == other.effect_texture_samplers && self.meta.key() == other.meta.key()
    }
}
impl Eq for KeyBindGroupTextureSamplers {
    fn assert_receiver_is_total_eq(&self) {}
}

#[derive(Debug, Clone)]
pub struct BindGroupTextureSamplers {
    pub(crate) bind_group: BindGroupUsage,
    pub(crate) key: KeyBindGroupTextureSamplers,
}

impl BindGroupTextureSamplers {
    pub fn new(
        key: KeyBindGroupTextureSamplers,
        bind_group: BindGroupUsage,
    ) -> Self {
        Self { bind_group, key }
    }
    pub fn key(&self) -> &KeyBindGroupTextureSamplers { &self.key }
    pub fn bind_group(&self) -> &BindGroupUsage { &self.bind_group }
}

impl TShaderSetBlock for BindGroupTextureSamplers {
    fn vs_define_code(&self) -> String {
        let mut binding = 0;

        let mut result = String::from("");

        if let Some(val) = &self.key.effect_texture_samplers.textures.0 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.1 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.2 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.3 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.4 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.5 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        
        if let Some(val) = &self.key.effect_texture_samplers.samplers.0 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.1 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.2 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.3 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.4 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.5 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); //binding += 1;
        }

        result
    }

    fn fs_define_code(&self) -> String {
        let mut binding = 0;

        let mut result = String::from("");

        if let Some(val) = &self.key.effect_texture_samplers.textures.0 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.1 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.2 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.3 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.4 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.5 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        
        if let Some(val) = &self.key.effect_texture_samplers.samplers.0 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.1 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.2 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.3 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.4 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.5 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); //binding += 1;
        }

        result
    }

    fn vs_running_code(&self) -> String {
        String::from("")
    }

    fn fs_running_code(&self) -> String {
        String::from("")
    }
}