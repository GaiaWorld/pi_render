use std::sync::Arc;

use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_share::Share;

use crate::{
    renderer::{
        bind_group::{BindGroup, BindGroupLayout},
        bind::{KeyBindTexture2D, EKeyBind}, shader::TShaderSetBlock, texture::BindDataTexture2D
    },
    render_3d::{
        binds::{
            effect_texture2d::{EffectBindTexture2D01, EffectBindTexture2D02, EffectBindTexture2D03, EffectBindTexture2D04, EffectBindTexture2D05, EffectBindTexture2D06, TEffectBindTexture2D, TEffectBindTexture2DData},
            effect_sampler2d::{EffectBindSampler2D01, EffectBindSampler2D02, EffectBindSampler2D03, EffectBindSampler2D04, EffectBindSampler2D05, EffectBindSampler2D06, TEffectBindSampler2D, TEffectBindSampler2DData}
        },
        shader::shader_effect_meta::ShaderEffectMeta
    },
    rhi::device::RenderDevice
};


#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderSetTextureSamplers {

}

pub struct BindGroupTextureSamplers {
    pub bind_group: Handle<BindGroup>,
    pub key: KeyShaderSetTextureSamplers,
    pub effect_textures: (
        Option<EffectBindTexture2D01>, Option<EffectBindTexture2D02>, Option<EffectBindTexture2D03>,
        Option<EffectBindTexture2D04>, Option<EffectBindTexture2D05>, Option<EffectBindTexture2D06>,
    ),
    pub effect_samplers: (
        Option<EffectBindSampler2D01>, Option<EffectBindSampler2D02>, Option<EffectBindSampler2D03>,
        Option<EffectBindSampler2D04>, Option<EffectBindSampler2D05>, Option<EffectBindSampler2D06>,
    ),
    pub meta: Handle<ShaderEffectMeta>,
}

impl BindGroupTextureSamplers {
    pub fn new(
        meta: Handle<ShaderEffectMeta>,
        effect_textures: (
            Option<&EffectBindTexture2D01>, Option<&EffectBindTexture2D02>, Option<&EffectBindTexture2D03>,
            Option<&EffectBindTexture2D04>, Option<&EffectBindTexture2D05>, Option<&EffectBindTexture2D06>,
        ),
        effect_samplers: (
            Option<&EffectBindSampler2D01>, Option<&EffectBindSampler2D02>, Option<&EffectBindSampler2D03>,
            Option<&EffectBindSampler2D04>, Option<&EffectBindSampler2D05>, Option<&EffectBindSampler2D06>,
        ),
        device: &RenderDevice,
        asset_mgr_bind_group_layout: &Share<AssetMgr<BindGroupLayout>>,
        asset_mgr_bind_group: &Share<AssetMgr<BindGroup>>,
    ) -> Option<Self> {
        let mut key = KeyShaderSetTextureSamplers::default();
        let mut binds = BindGroup::none_binds();

        let mut binding = 0;
        let mut eff_textures: (
            Option<EffectBindTexture2D01>, Option<EffectBindTexture2D02>, Option<EffectBindTexture2D03>,
            Option<EffectBindTexture2D04>, Option<EffectBindTexture2D05>, Option<EffectBindTexture2D06>,
        ) = (None, None, None, None, None, None);
        let mut eff_samplers: (
            Option<EffectBindSampler2D01>, Option<EffectBindSampler2D02>, Option<EffectBindSampler2D03>,
            Option<EffectBindSampler2D04>, Option<EffectBindSampler2D05>, Option<EffectBindSampler2D06>,
        ) = (None, None, None, None, None, None);

        if let Some(val) = effect_textures.0 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_textures.0 = Some(val.clone());
        }
        if let Some(val) = effect_textures.1 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_textures.1 = Some(val.clone());
        }
        if let Some(val) = effect_textures.2 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_textures.2 = Some(val.clone());
        }
        if let Some(val) = effect_textures.3 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_textures.3 = Some(val.clone());
        }
        if let Some(val) = effect_textures.4 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_textures.4 = Some(val.clone());
        }
        if let Some(val) = effect_textures.5 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_textures.5 = Some(val.clone());
        }
        
        if let Some(val) = effect_samplers.0 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_samplers.0 = Some(val.clone());
        }
        if let Some(val) = effect_samplers.1 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_samplers.1 = Some(val.clone());
        }
        if let Some(val) = effect_samplers.2 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_samplers.2 = Some(val.clone());
        }
        if let Some(val) = effect_samplers.3 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_samplers.3 = Some(val.clone());
        }
        if let Some(val) = effect_samplers.4 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_samplers.4 = Some(val.clone());
        }
        if let Some(val) = effect_samplers.5 {
            if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds[binding] = Some(layout); binding += 1; } else { return None; }; eff_samplers.5 = Some(val.clone());
        }

        if let Some(bind_group) = BindGroup::create(device, binds, asset_mgr_bind_group_layout, asset_mgr_bind_group) {
            Some(
                Self {
                    bind_group,
                    key,
                    meta,
                    effect_textures: eff_textures,
                    effect_samplers: eff_samplers,
                }
            )
        } else {
            None
        }
    }
}

pub struct BindGroupTextureSamplersUsage {
    pub bind_group: BindGroupTextureSamplers,
    pub set: u32,
}
impl TShaderSetBlock for BindGroupTextureSamplersUsage {
    fn vs_define_code(&self) -> String {
        let mut binding = 0;

        let mut result = String::from("");

        if let Some(val) = &self.bind_group.effect_textures.0 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_textures.1 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_textures.2 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_textures.3 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_textures.4 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_textures.5 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        
        if let Some(val) = &self.bind_group.effect_samplers.0 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_samplers.1 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_samplers.2 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_samplers.3 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_samplers.4 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_samplers.5 {
            result += val.vs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }

        result
    }

    fn fs_define_code(&self) -> String {
        let mut binding = 0;

        let mut result = String::from("");

        if let Some(val) = &self.bind_group.effect_textures.0 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_textures.1 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_textures.2 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_textures.3 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_textures.4 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_textures.5 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        
        if let Some(val) = &self.bind_group.effect_samplers.0 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_samplers.1 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_samplers.2 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_samplers.3 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_samplers.4 {
            result += val.fs_define_code(&self.bind_group.meta, self.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.bind_group.effect_samplers.5 {
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
