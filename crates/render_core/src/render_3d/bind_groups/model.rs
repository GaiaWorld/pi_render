use std::sync::Arc;

use crate::{
    renderer::{
        bind_group::*,
        shader::{TShaderSetBlock, TShaderBindCode},
        bind::*
    },
    render_3d::{
        shader::*,
        binds::{
            model::*,
            effect_value::{ShaderBindEffectValue, BindUseEffectValue}
        }
    },
    asset::TAssetKeyU64
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
    bind_count: u32,
    key_binds: Arc<IDBinds>,
}
impl KeyBindGroupModel {
    pub fn new(
        bind_matrix: Arc<ShaderBindModelAboutMatrix>,
        bind_skin: Option<Arc<ShaderBindModelAboutSkinValue>>,
        bind_effect_value: Option<Arc<ShaderBindEffectValue>>,
        recorder: &mut BindsRecorder
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
            binding += 1;
        }

        let mut result = Self {
            matrix,
            skin,
            effect_value,
            key,
            bind_count: binding,
            key_binds: Arc::new(IDBinds::Binds00(vec![]))
        };
        result.key_binds = result.binds(recorder);

        result
    }
    
    pub fn binds(&self, recorder: &mut BindsRecorder) -> Arc<IDBinds> {
        log::warn!("Model Binds {:?} {:?}", self.key_binds, self.bind_count);
        if let Some(mut binds) = EBinds::new(self.bind_count) {
            binds.set(
                self.matrix.bind as usize, 
                Some(EKeyBind::Buffer(KeyBindBuffer {
                    data: self.matrix.data.data.clone(),
                    layout: Arc::new(self.matrix.data.key_layout(self.matrix.bind as u16)),
                }))
            );

            if let Some(bind) = &self.skin {
                binds.set(
                    bind.bind as usize,
                    Some(EKeyBind::Buffer(KeyBindBuffer {
                        data: bind.data.data.clone(),
                        layout: Arc::new(bind.data.key_layout(bind.bind as u16)),
                    }))
                );
            }
            
            if let Some(bind) = &self.effect_value {
                binds.set(
                    bind.bind as usize,
                    Some(EKeyBind::Buffer(KeyBindBuffer {
                        data: bind.data.data.clone(),
                        layout: Arc::new(bind.data.key_layout(bind.bind as u16)),
                    }))
                );
            }
            binds.record(recorder)
        } else {
            Arc::new(IDBinds::Binds00(vec![]))
        }
    }
    pub fn key_bind_group(&self) -> KeyBindGroup {
        self.key_binds.clone()
    }
    pub fn key_bind_group_layout(&self) -> KeyBindGroupLayout {
        self.key_binds.clone()
    }
}
impl TAssetKeyU64 for KeyBindGroupModel {}

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