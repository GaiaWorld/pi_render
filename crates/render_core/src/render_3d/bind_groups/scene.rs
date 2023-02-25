use std::sync::Arc;

use pi_assets::{mgr::AssetMgr, asset::Handle};
use pi_share::Share;

use crate::{renderer::{bind_group::{BindGroupUsage, BindGroupLayout}, bind::{EKeyBind, KeyBindBuffer}, shader::{TShaderSetBlock, TShaderBindCode}}, render_3d::{shader::skin_code::ESkinCode, binds::{model::{base::{BindUseModelMatrix, ShaderBindModelAboutMatrix}, skin::{BindUseSkinValue, ShaderBindModelAboutSkinValue}}, scene::{base::{ShaderBindSceneAboutBase, BindUseSceneAboutCamera}, effect::{ShaderBindSceneAboutEffect, BindUseSceneAboutEffect}}}}, rhi::{device::RenderDevice, bind_group::BindGroup}};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderSetScene {
    pub base_effect: bool,
    pub brdf: bool,
    pub env: bool,
}

#[derive(Debug, Clone)]
pub struct BindGroupScene {
    pub bind_group: BindGroupUsage,
    base: BindUseSceneAboutCamera,
    base_effect: Option<BindUseSceneAboutEffect>,
    pub key: KeyShaderSetScene,
}
impl BindGroupScene {
    pub fn new(
        bind_base: Arc<ShaderBindSceneAboutBase>,
        bind_base_effect: Option<Arc<ShaderBindSceneAboutEffect>>,
        device: &RenderDevice,
        asset_mgr_bind_group_layout: &Share<AssetMgr<BindGroupLayout>>,
        asset_mgr_bind_group: &Share<AssetMgr<BindGroup>>,
    ) -> Option<Self> {
        let mut key = KeyShaderSetScene::default();
        let mut binds = BindGroupUsage::none_binds();
        let mut base_effect = None;

        let mut binding = 0;
        let mut base = BindUseSceneAboutCamera { data: bind_base.clone(), bind: binding as u32 };
        binds[binding] = Some(EKeyBind::Buffer(KeyBindBuffer {
            data: bind_base.data.clone(),
            layout: Arc::new(bind_base.key_layout(binding as u16)),
        }));
        binding += 1;

        if let Some(bind_base_effect) = bind_base_effect {
            base_effect = Some(BindUseSceneAboutEffect { data: bind_base_effect.clone(), bind: binding as u32 });
            key.base_effect = true;
            binds[binding] = Some(EKeyBind::Buffer(KeyBindBuffer {
                data: bind_base_effect.data.clone(),
                layout: Arc::new(bind_base_effect.key_layout(binding as u16)),
            }));
            binding += 1;
        }

        if let Some(bind_group) = BindGroupUsage::create(device, binds, asset_mgr_bind_group_layout, asset_mgr_bind_group) {
            Some(
                Self {
                    bind_group,
                    base,
                    base_effect,
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
        let mut result = String::from("");

        result += self.bind_group.base.vs_define_code(self.set).as_str();
        if let Some(base_effect) = &self.bind_group.base_effect {
            result += base_effect.vs_define_code(self.set).as_str();
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