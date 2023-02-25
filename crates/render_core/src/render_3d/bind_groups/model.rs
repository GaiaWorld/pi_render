use std::sync::Arc;

use pi_assets::{mgr::AssetMgr};
use pi_share::Share;

use crate::{
    renderer::{bind_group::{BindGroupUsage, BindGroupLayout}, bind::{EKeyBind, KeyBindBuffer}, shader::{TShaderSetBlock, TShaderBindCode}},
    render_3d::{
        shader::skin_code::ESkinCode,
        binds::{
            model::{
                base::{BindUseModelMatrix, ShaderBindModelAboutMatrix},
                skin::{BindUseSkinValue, ShaderBindModelAboutSkinValue}
            },
            effect_value::{ShaderBindEffectValue, BindUseEffectValue}
        }
    },
    rhi::{device::RenderDevice, bind_group::BindGroup}
};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderSetModel {
    pub skin: ESkinCode,
}

#[derive(Debug, Clone)]
pub struct BindGroupModel {
    pub bind_group: BindGroupUsage,
    matrix: BindUseModelMatrix,
    skin: Option<BindUseSkinValue>,
    effect_value: Option<BindUseEffectValue>,
    pub key: KeyShaderSetModel,
}
impl BindGroupModel {
    pub fn new(
        bind_matrix: Arc<ShaderBindModelAboutMatrix>,
        bind_skin: Option<Arc<ShaderBindModelAboutSkinValue>>,
        bind_effect_value: Option<Arc<ShaderBindEffectValue>>,
        device: &RenderDevice,
        asset_mgr_bind_group_layout: &Share<AssetMgr<BindGroupLayout>>,
        asset_mgr_bind_group: &Share<AssetMgr<BindGroup>>,
    ) -> Option<BindGroupModel> {
        let mut key = KeyShaderSetModel::default();
        let mut binds = BindGroupUsage::none_binds();
        
        let mut skin: Option<BindUseSkinValue> = None;
        let mut effect_value: Option<BindUseEffectValue> = None;

        let mut binding = 0;
        let matrix: BindUseModelMatrix = BindUseModelMatrix { data: bind_matrix.clone(), bind: binding as u32 };
        binds[binding] = Some(EKeyBind::Buffer(KeyBindBuffer {
            data: bind_matrix.data.clone(),
            layout: Arc::new(bind_matrix.key_layout(binding as u16)),
        }));
        binding += 1;

        if let Some(bind_skin) = bind_skin {
            skin = Some(BindUseSkinValue { data: bind_skin.clone(), bind: binding as u32 });
            key.skin = bind_skin.skin;
            binds[binding] = Some(EKeyBind::Buffer(KeyBindBuffer {
                data: bind_skin.data.clone(),
                layout: Arc::new(bind_skin.key_layout(binding as u16)),
            }));
            binding += 1;
        }

        if let Some(bind_effect_value) = bind_effect_value {
            effect_value = Some(BindUseEffectValue { data: bind_effect_value.clone(), bind: binding as u32 });
            binds[binding] = Some(EKeyBind::Buffer(KeyBindBuffer {
                data: bind_effect_value.data.clone(),
                layout: Arc::new(bind_effect_value.key_layout(binding as u16)),
            }));
            // binding += 1;
        }

        if let Some(bind_group) = BindGroupUsage::create(1, device, binds, asset_mgr_bind_group_layout, asset_mgr_bind_group) {
            Some(
                Self {
                    bind_group,
                    matrix,
                    skin,
                    effect_value,
                    key,
                }
            )
        } else {
            None
        }
    }
}
impl TShaderSetBlock for BindGroupModel {
    fn vs_define_code(&self) -> String {

        let mut result = String::from("");

        result += self.matrix.vs_define_code(self.bind_group.set).as_str();
        if let Some(skin) = &self.skin {
            result += skin.vs_define_code(self.bind_group.set).as_str();
        }
        if let Some(effect_value) = &self.effect_value {
            result += effect_value.vs_define_code(self.bind_group.set).as_str();
        }

        result
    }

    fn fs_define_code(&self) -> String {

        let mut result = String::from("");

        result += self.matrix.fs_define_code(self.bind_group.set).as_str();
        if let Some(skin) = &self.skin {
            result += skin.fs_define_code(self.bind_group.set).as_str();
        }
        if let Some(effect_value) = &self.effect_value {
            result += effect_value.fs_define_code(self.bind_group.set).as_str();
        }

        result
    }

    fn vs_running_code(&self) -> String {
        let mut result = String::from("");

        if let Some(skin) = &self.skin {
            result += skin.vs_running_code(self.bind_group.set).as_str();
        }

        result
    }

    fn fs_running_code(&self) -> String {
        String::from("")
    }
}