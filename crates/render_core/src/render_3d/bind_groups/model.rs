use std::sync::Arc;

use pi_assets::{mgr::AssetMgr};
use pi_share::Share;

use crate::{
    renderer::{bind_group::{BindGroupUsage, BindGroupLayout, KeyBindGroup}, bind::{EKeyBind, KeyBindBuffer}, shader::{TShaderSetBlock, TShaderBindCode}},
    render_3d::{
        shader::*,
        binds::{
            model::*,
            effect_value::{ShaderBindEffectValue, BindUseEffectValue}
        }
    },
    rhi::{device::RenderDevice, bind_group::BindGroup}
};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderSetModel {
    pub skin: ESkinCode,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyBindGroupModel {
    pub matrix: BindUseModelMatrix,
    pub skin: Option<BindUseSkinValue>,
    pub effect_value: Option<BindUseEffectValue>,
    pub key: KeyShaderSetModel,
}
impl KeyBindGroupModel {
    pub fn new(
        bind_matrix: Arc<ShaderBindModelAboutMatrix>,
        bind_skin: Option<Arc<ShaderBindModelAboutSkinValue>>,
        bind_effect_value: Option<Arc<ShaderBindEffectValue>>,
    ) -> Self {
        let mut key = KeyShaderSetModel::default();
        
        let mut skin: Option<BindUseSkinValue> = None;
        let mut effect_value: Option<BindUseEffectValue> = None;

        let mut binding = 0;
        let matrix: BindUseModelMatrix = BindUseModelMatrix { data: bind_matrix.clone(), bind: binding as u32 };
        binding += 1;

        if let Some(bind_skin) = bind_skin {
            skin = Some(BindUseSkinValue { data: bind_skin.clone(), bind: binding as u32 });
            key.skin = bind_skin.skin;
            binding += 1;
        }

        if let Some(bind_effect_value) = bind_effect_value {
            effect_value = Some(BindUseEffectValue { data: bind_effect_value.clone(), bind: binding as u32 });
            // binding += 1;
        }

        Self {
            matrix,
            skin,
            effect_value,
            key,
        }
    }
    pub fn key_bind_group(&self) -> KeyBindGroup {
        let mut binds = BindGroupUsage::none_binds();
        binds[self.matrix.bind as usize] = Some(EKeyBind::Buffer(KeyBindBuffer {
            data: self.matrix.data.data.clone(),
            layout: Arc::new(self.matrix.data.key_layout(self.matrix.bind as u16)),
        }));

        if let Some(bind) = &self.skin {
            binds[bind.bind as usize] = Some(EKeyBind::Buffer(KeyBindBuffer {
                data: bind.data.data.clone(),
                layout: Arc::new(bind.data.key_layout(bind.bind as u16)),
            }));
        }
        
        if let Some(bind) = &self.effect_value {
            binds[bind.bind as usize] = Some(EKeyBind::Buffer(KeyBindBuffer {
                data: bind.data.data.clone(),
                layout: Arc::new(bind.data.key_layout(bind.bind as u16)),
            }));
            // binding += 1;
        }

        KeyBindGroup(binds)
    }
}

#[derive(Debug, Clone)]
pub struct BindGroupModel {
    pub(crate) bind_group: BindGroupUsage,
    pub(crate) key: KeyBindGroupModel,
}
impl BindGroupModel {
    pub fn new(
        bind_group: BindGroupUsage,
        key: KeyBindGroupModel,
    ) -> Self {
        Self { bind_group, key }
    }
    pub fn key(&self) -> &KeyBindGroupModel { &self.key }
    pub fn bind_group(&self) -> &BindGroupUsage { &self.bind_group }
}
impl TShaderSetBlock for BindGroupModel {
    fn vs_define_code(&self) -> String {

        let mut result = String::from("");

        result += self.key.matrix.vs_define_code(self.bind_group.set).as_str();
        if let Some(skin) = &self.key.skin {
            result += skin.vs_define_code(self.bind_group.set).as_str();
        }
        if let Some(effect_value) = &self.key.effect_value {
            result += effect_value.vs_define_code(self.bind_group.set).as_str();
        }

        result
    }

    fn fs_define_code(&self) -> String {

        let mut result = String::from("");

        result += self.key.matrix.fs_define_code(self.bind_group.set).as_str();
        if let Some(skin) = &self.key.skin {
            result += skin.fs_define_code(self.bind_group.set).as_str();
        }
        if let Some(effect_value) = &self.key.effect_value {
            result += effect_value.fs_define_code(self.bind_group.set).as_str();
        }

        result
    }

    fn vs_running_code(&self) -> String {
        let mut result = String::from("");

        if let Some(skin) = &self.key.skin {
            result += skin.vs_running_code(self.bind_group.set).as_str();
        }

        result
    }

    fn fs_running_code(&self) -> String {
        String::from("")
    }
}