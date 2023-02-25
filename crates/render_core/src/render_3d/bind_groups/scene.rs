use std::sync::Arc;

use pi_assets::{mgr::AssetMgr, asset::Handle};
use pi_share::Share;

use crate::{renderer::{bind_group::{BindGroup, BindGroupLayout}, bind::{EKeyBind, KeyBindBuffer}, shader::TShaderSetBlock}, render_3d::{shader::skin_code::ESkinCode, binds::{model::{base::{BindUseModelMatrix, ShaderBindModelAboutMatrix}, skin::{BindUseSkinValue, ShaderBindModelAboutSkinValue}}, scene::{base::ShaderBindSceneAboutBase, effect::ShaderBindSceneAboutEffect}}}, rhi::device::RenderDevice};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderSetScene {
    pub base_effect: bool,
    pub brdf: bool,
    pub env: bool,
}

#[derive(Debug, Clone)]
pub struct BindGroupScene {
    pub bind_group: Handle<BindGroup>,
    pub key: KeyShaderSetScene,
}
impl BindGroupScene {
    pub fn new(
        bind_base: &ShaderBindSceneAboutBase,
        bind_base_effect: Option<&ShaderBindSceneAboutEffect>,
        device: &RenderDevice,
        asset_mgr_bind_group_layout: &Share<AssetMgr<BindGroupLayout>>,
        asset_mgr_bind_group: &Share<AssetMgr<BindGroup>>,
    ) -> Option<Self> {
        let mut key = KeyShaderSetScene::default();
        let mut binds = BindGroup::none_binds();

        let mut binding = 0;
        binds[binding] = Some(EKeyBind::Buffer(KeyBindBuffer {
            data: bind_base.data.clone(),
            layout: Arc::new(bind_base.key_layout(binding as u16)),
        }));
        binding += 1;

        if let Some(bind_base_effect) = bind_base_effect {
            key.base_effect = true;
            binds[binding] = Some(EKeyBind::Buffer(KeyBindBuffer {
                data: bind_base_effect.data.clone(),
                layout: Arc::new(bind_base_effect.key_layout(binding as u16)),
            }));
            binding += 1;
        }

        if let Some(bind_group) = BindGroup::create(device, binds, asset_mgr_bind_group_layout, asset_mgr_bind_group) {
            Some(
                Self {
                    bind_group,
                    key,
                }
            )
        } else {
            None
        }
    }
}

pub struct BindGroupSceneUsage {
    pub bind_group: BindGroupScene,
    pub set: u32,
}
impl TShaderSetBlock for BindGroupSceneUsage {
    fn vs_define_code(&self) -> String {
        let mut binding = 0;

        let mut result = String::from("");

        result += ShaderBindSceneAboutBase::vs_define_code(self.set, binding).as_str(); binding += 1;
        if self.bind_group.key.base_effect {
            result += ShaderBindSceneAboutEffect::vs_define_code(self.set, binding).as_str();
        }

        result
    }

    fn fs_define_code(&self) -> String {
        self.vs_define_code()
    }

    fn vs_running_code(&self) -> String {
        String::from("")
    }

    fn fs_running_code(&self) -> String {
        String::from("")
    }
}