use std::sync::Arc;

use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_share::Share;

use crate::{
    renderer::{
        bind_group::{BindGroupUsage, BindGroupLayout},
        bind::{KeyBindTexture2D, EKeyBind}, shader::TShaderSetBlock, texture::BindDataTexture2D
    },
    render_3d::{
        binds::{
            effect_texture2d::{EffectBindTexture2D01, EffectBindTexture2D02, EffectBindTexture2D03, EffectBindTexture2D04, EffectBindTexture2D05, EffectBindTexture2D06, TEffectBindTexture2D, TEffectBindTexture2DData},
            effect_sampler2d::{EffectBindSampler2D01, EffectBindSampler2D02, EffectBindSampler2D03, EffectBindSampler2D04, EffectBindSampler2D05, EffectBindSampler2D06, TEffectBindSampler2D, TEffectBindSampler2DData}
        },
        shader::shader_effect_meta::ShaderEffectMeta
    },
    rhi::{device::RenderDevice, bind_group::BindGroup}
};


#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderSetTextureSamplers {

}

#[derive(Debug, Default, Clone)]
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
pub struct BindGroupTextureSamplers {
    pub bind_group: BindGroupUsage,
    pub key: KeyShaderSetTextureSamplers,
    pub effect_texture_samplers: EffectTextureSamplers,
    pub meta: Handle<ShaderEffectMeta>,
}

impl BindGroupTextureSamplers {
    pub fn new(
        meta: Handle<ShaderEffectMeta>,
        effect_texture_samplers: EffectTextureSamplers,
        device: &RenderDevice,
        asset_mgr_bind_group_layout: &Share<AssetMgr<BindGroupLayout>>,
        asset_mgr_bind_group: &Share<AssetMgr<BindGroup>>,
    ) -> Option<Self> {
        let mut key = KeyShaderSetTextureSamplers::default();
        let mut binds = BindGroupUsage::none_binds();

        let mut binding = 0;

        if let Some(val) = &effect_texture_samplers.textures.0 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &effect_texture_samplers.textures.1 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &effect_texture_samplers.textures.2 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &effect_texture_samplers.textures.3 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &effect_texture_samplers.textures.4 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &effect_texture_samplers.textures.5 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        
        if let Some(val) = &effect_texture_samplers.samplers.0 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &effect_texture_samplers.samplers.1 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &effect_texture_samplers.samplers.2 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &effect_texture_samplers.samplers.3 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &effect_texture_samplers.samplers.4 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }
        if let Some(val) = &effect_texture_samplers.samplers.5 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; };
        }

        if binding == 0 {
            return None;
        }
        if let Some(bind_group) = BindGroupUsage::create(device, binds, asset_mgr_bind_group_layout, asset_mgr_bind_group) {
            Some(
                Self {
                    bind_group,
                    key,
                    meta,
                    effect_texture_samplers,
                }
            )
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindGroupTextureSamplersUsage {
    pub bind_group: BindGroupTextureSamplers,
    pub set: u32,
}
impl TShaderSetBlock for BindGroupTextureSamplersUsage {
    fn vs_define_code(&self) -> String {
        let mut binding = 0;

        let mut result = String::from("");

        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.0 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.1 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.2 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.3 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.4 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.5 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.0 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.1 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.2 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.3 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.4 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.5 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }

        result
    }

    fn fs_define_code(&self) -> String {
        let mut binding = 0;

        let mut result = String::from("");

        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.0 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.1 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.2 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.3 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.4 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.textures.5 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.0 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.1 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.2 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.3 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.4 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_texture_samplers.samplers.5 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
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
