

use std::{hash::Hash, sync::Arc};

use pi_assets::asset::Handle;

use crate::{
    renderer::{
        bind_group::*,
        shader::TShaderSetBlock,
    },
    render_3d::{
        binds::{
            effect_texture2d::*,
            effect_sampler2d::*
        },
        shader::*
    },
    asset::TAssetKeyU64
};


#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderSetTextureSamplers {

}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct EffectTextureSamplers {
    pub textures: (
        Option<EffectBindTexture2D01>, Option<EffectBindTexture2D02>, Option<EffectBindTexture2D03>, Option<EffectBindTexture2D04>, 
        Option<EffectBindTexture2D05>, Option<EffectBindTexture2D06>, Option<EffectBindTexture2D07>, Option<EffectBindTexture2D08>,
    ),
    pub samplers: (
        Option<EffectBindSampler2D01>, Option<EffectBindSampler2D02>, Option<EffectBindSampler2D03>, Option<EffectBindSampler2D04>, 
        Option<EffectBindSampler2D05>, Option<EffectBindSampler2D06>, Option<EffectBindSampler2D07>, Option<EffectBindSampler2D08>,
    ),
    pub binding_count: u32,
}

#[derive(Debug, Clone)]
pub struct KeyBindGroupTextureSamplers {
    pub key: KeyShaderSetTextureSamplers,
    pub effect_texture_samplers: EffectTextureSamplers,
    pub meta: Handle<ShaderEffectMeta>,
    key_binds: Option<Arc<IDBinds>>,
}
impl KeyBindGroupTextureSamplers {
    pub fn new(
        key: KeyShaderSetTextureSamplers,
        effect_texture_samplers: EffectTextureSamplers,
        meta: Handle<ShaderEffectMeta>,
        recorder: &mut BindsRecorder,
    ) -> Self {
        let idbinds = if let Some(mut binds) = EBinds::new(effect_texture_samplers.binding_count) {
            let mut error = false;
            let mut binding = 0;

            if let Some(val) = &effect_texture_samplers.textures.0 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) {  binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.textures.1 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.textures.2 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.textures.3 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.textures.4 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.textures.5 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.textures.6 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.textures.7 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            
            if let Some(val) = &effect_texture_samplers.samplers.0 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.samplers.1 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.samplers.2 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.samplers.3 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.samplers.4 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.samplers.5 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.samplers.6 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)); binding += 1; } else { error = true; };
            }
            if let Some(val) = &effect_texture_samplers.samplers.7 {
                if let Some(layout) = val.key_bind(&meta, binding as u16) { binds.set(binding, Some(layout)) } else { error = true; };
            }
            if !error {
                Some(binds.record(recorder))
            } else {
                None
            }
        } else {
            None
        };

        Self { key, effect_texture_samplers, meta, key_binds: idbinds  }
    }
    pub fn key_bind_group(&self) -> Option<KeyBindGroup> {
        if let Some(binds) = &self.key_binds {
            Some(
                KeyBindGroup(binds.binds())
            )
        } else {
            None
        }
    }
    pub fn key_bind_group_layout(&self) -> Option<KeyBindGroupLayout> {
        if let Some(binds) = &self.key_binds {
            Some(
                KeyBindGroup(binds.binds())
            )
        } else {
            None
        }
    }
    pub fn binds(&self) -> Option<Arc<IDBinds>> {
        self.key_binds.clone()
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
impl TAssetKeyU64 for KeyBindGroupTextureSamplers {}

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
        if let Some(val) = &self.key.effect_texture_samplers.textures.6 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.7 {
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
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.6 {
            result += val.vs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.7 {
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
        if let Some(val) = &self.key.effect_texture_samplers.textures.6 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.textures.7 {
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
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.6 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); binding += 1;
        }
        if let Some(val) = &self.key.effect_texture_samplers.samplers.7 {
            result += val.fs_define_code(&self.key.meta, self.bind_group.set, binding).as_str(); //binding += 1;
        }

        result
    }

    // fn vs_running_code(&self) -> String {
    //     String::from("")
    // }

    // fn fs_running_code(&self) -> String {
    //     String::from("")
    // }
}