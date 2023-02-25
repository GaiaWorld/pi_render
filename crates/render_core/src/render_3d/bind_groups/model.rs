use std::sync::Arc;

use pi_assets::{mgr::AssetMgr, asset::Handle};
use pi_share::Share;

use crate::{renderer::{bind_group::{BindGroup, BindGroupLayout}, bind::{EKeyBind, KeyBindBuffer}, shader::TShaderSetBlock}, render_3d::{shader::skin_code::ESkinCode, binds::{model::{base::{BindUseModelMatrix, ShaderBindModelAboutMatrix}, skin::{BindUseSkinValue, ShaderBindModelAboutSkinValue}}, effect_value::ShaderBindEffectValue}}, rhi::device::RenderDevice};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderSetModel {
    pub skin: ESkinCode,
}

pub struct BindGroupModel {
    pub bind_group: Handle<BindGroup>,
    pub effect_value: Option<ShaderBindEffectValue>,
    pub key: KeyShaderSetModel,
}
impl BindGroupModel {
    pub fn new(
        bind_matrix: &ShaderBindModelAboutMatrix,
        bind_skin: Option<&ShaderBindModelAboutSkinValue>,
        bind_effect_value: Option<&ShaderBindEffectValue>,
        device: &RenderDevice,
        asset_mgr_bind_group_layout: &Share<AssetMgr<BindGroupLayout>>,
        asset_mgr_bind_group: &Share<AssetMgr<BindGroup>>,
    ) -> Option<BindGroupModel> {
        let mut key = KeyShaderSetModel::default();
        let mut effect_value = None;
        let mut binds = BindGroup::none_binds();

        let mut binding = 0;
        binds[binding] = Some(EKeyBind::Buffer(KeyBindBuffer {
            data: bind_matrix.data.clone(),
            layout: Arc::new(bind_matrix.key_layout(binding as u16)),
        }));
        binding += 1;

        if let Some(bind_skin) = bind_skin {
            key.skin = bind_skin.skin;
            binds[binding] = Some(EKeyBind::Buffer(KeyBindBuffer {
                data: bind_skin.data.clone(),
                layout: Arc::new(bind_skin.key_layout(binding as u16)),
            }));
            binding += 1;
        }

        if let Some(bind_effect_value) = bind_effect_value {
            effect_value = Some(bind_effect_value.clone());
            binds[binding] = Some(EKeyBind::Buffer(KeyBindBuffer {
                data: bind_effect_value.data.clone(),
                layout: Arc::new(bind_effect_value.key_layout(binding as u16)),
            }));
            binding += 1;
        }

        if let Some(bind_group) = BindGroup::create(device, binds, asset_mgr_bind_group_layout, asset_mgr_bind_group) {
            Some(
                Self {
                    bind_group,
                    effect_value,
                    key,
                }
            )
        } else {
            None
        }
    }
}

pub struct BindGroupModelUsage {
    pub bind_group: BindGroupModel,
    pub set: u32,
}
impl TShaderSetBlock for BindGroupModelUsage {
    fn vs_define_code(&self) -> String {
        let mut binding = 0;

        let mut result = String::from("");

        result += ShaderBindModelAboutMatrix::vs_define_code(self.set, binding).as_str(); binding += 1;
        match self.bind_group.key.skin {
            ESkinCode::None => {},
            _ => {
                result += ShaderBindModelAboutSkinValue::vs_define_code(&self.bind_group.key.skin, self.set, binding).as_str(); binding += 1;
            }
        }
        if let Some(effect_value) = &self.bind_group.effect_value {
            result += ShaderBindEffectValue::vs_define_code(&effect_value.meta, self.set, binding).as_str(); binding += 1;
        }

        result
    }

    fn fs_define_code(&self) -> String {
        let mut binding = 0;

        let mut result = String::from("");

        result += ShaderBindModelAboutMatrix::fs_define_code(self.set, binding).as_str(); binding += 1;
        match self.bind_group.key.skin {
            ESkinCode::None => {},
            _ => {
                binding += 1;
            }
        }
        if let Some(effect_value) = &self.bind_group.effect_value {
            result += ShaderBindEffectValue::fs_define_code(&effect_value.meta, self.set, binding).as_str(); binding += 1;
        }

        result
    }

    fn vs_running_code(&self) -> String {
        let mut result = String::from("");

        result += ShaderBindModelAboutSkinValue::vs_running_code(&self.bind_group.key.skin).as_str();

        result
    }

    fn fs_running_code(&self) -> String {
        String::from("")
    }
}