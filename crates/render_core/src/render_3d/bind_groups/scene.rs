use std::sync::Arc;

use pi_assets::{mgr::AssetMgr};
use pi_share::Share;

use crate::{
    renderer::{
        bind_group::{BindGroupUsage, BindGroupLayout, KeyBindGroup},
        bind::{EKeyBind, KeyBindBuffer},
        shader::{TShaderSetBlock, TShaderBindCode}
    },
    render_3d::{
        binds::{
            scene::{
                base::{ShaderBindSceneAboutBase, BindUseSceneAboutCamera},
                effect::{ShaderBindSceneAboutEffect, BindUseSceneAboutEffect}
            }
        }
    },
    rhi::{device::RenderDevice}
};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderSetScene {
    pub base_effect: bool,
    pub brdf: bool,
    pub env: bool,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyBindGroupScene {
    pub bind_base: BindUseSceneAboutCamera,
    pub bind_base_effect: Option<BindUseSceneAboutEffect>,
    pub key_set: KeyShaderSetScene,
}
impl KeyBindGroupScene {
    pub fn new(
        bind_base: Arc<ShaderBindSceneAboutBase>,
        bind_base_effect: Option<Arc<ShaderBindSceneAboutEffect>>,
    ) -> Self {
        let mut key_set = KeyShaderSetScene::default();

        let mut binding = 0;
        let bind_base = BindUseSceneAboutCamera { data: bind_base, bind: binding as u32 };
        binding += 1;

        let bind_base_effect = if let Some(bind_base_effect) = bind_base_effect {
            key_set.base_effect = true;
            Some(BindUseSceneAboutEffect { data: bind_base_effect, bind: binding as u32 })
            // binding += 1;
        } else { None };

        Self {
            bind_base,
            bind_base_effect,
            key_set,
        }
    }
    pub fn key_bind_group(&self) -> KeyBindGroup {
        let mut binds = BindGroupUsage::none_binds();
        binds[self.bind_base.bind as usize] = Some(EKeyBind::Buffer(KeyBindBuffer {
            data: self.bind_base.data.data.clone(),
            layout: Arc::new(self.bind_base.data.key_layout(self.bind_base.bind as u16)),
        }));

        if let Some(bind_base_effect) = &self.bind_base_effect {
            binds[bind_base_effect.bind as usize] = Some(EKeyBind::Buffer(KeyBindBuffer {
                data: bind_base_effect.data.data.clone(),
                layout: Arc::new(bind_base_effect.data.key_layout(bind_base_effect.bind as u16)),
            }));
            // binding += 1;
        }

        KeyBindGroup(binds)
    }
}

#[derive(Debug, Clone)]
pub struct BindGroupScene {
    pub(crate) bind_group: BindGroupUsage,
    pub(crate) key: KeyBindGroupScene,
}
impl BindGroupScene {
    pub fn new(
        bind_group: BindGroupUsage,
        key: KeyBindGroupScene,
    ) -> Self {
        Self { bind_group, key }
    }
    pub fn key(&self) -> &KeyBindGroupScene { &self.key }
    pub fn bind_group(&self) -> &BindGroupUsage { &self.bind_group }
}
impl TShaderSetBlock for BindGroupScene {
    fn vs_define_code(&self) -> String {
        let mut result = String::from("");

        result += self.key.bind_base.vs_define_code(self.bind_group.set).as_str();
        if let Some(base_effect) = &self.key.bind_base_effect {
            result += base_effect.vs_define_code(self.bind_group.set).as_str();
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