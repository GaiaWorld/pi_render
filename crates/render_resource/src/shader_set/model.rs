use std::sync::Arc;

use pi_assets::{asset::Handle, mgr::AssetMgr};
use render_core::rhi::{device::RenderDevice, asset::TextureRes};
use render_shader::{shader_set::KeyShaderModelAbout, unifrom_code::MaterialValueBindDesc, skin_code::ESkinCode, shader::{TShaderBlockCode, TShaderBindCode}, set_bind::ShaderSetBind, buildin_var::ShaderVarUniform};

use crate::{shader_bind::{ShaderBindModelAboutMatrix, ShaderBindEffectValue, ShaderBindModelAboutSkinValue, TShaderBind, BindUseSkinValue, BindUseEffectValue, BindUseModelMatrix}, bind_group::{bind_group::{RenderBindGroup, KeyRenderBindgroup, AssetMgrRenderBindgroup, TBindGroup}, bind::{KeyBind, TKeyBind}}, sampler::AssetMgrSampler};

#[derive(Clone)]
pub struct RenderBindGroupModel {
    pub set: u32,
    pub bind_model_matrix: BindUseModelMatrix,
    pub bind_effect: BindUseEffectValue,
    pub bind_skin: Option<BindUseSkinValue>,
    key_bindgroup: KeyRenderBindgroup,
    bindgroup: Handle<RenderBindGroup>,
    offsets: Vec<wgpu::DynamicOffset>,
}
impl RenderBindGroupModel {
    pub fn new(
        set: u32,
        bind_model: Arc<ShaderBindModelAboutMatrix>,
        bind_effect: &Arc<ShaderBindEffectValue>,
        bind_skin: Option<Arc<ShaderBindModelAboutSkinValue>>,
        device: &RenderDevice,
        asset_tex: &AssetMgr<TextureRes>,
        asset_sampler: &AssetMgrSampler,
        asset_bindgroup: &AssetMgrRenderBindgroup,
    ) -> Option<Self> {
        let mut key_bindgroup = KeyRenderBindgroup(vec![]);
        let mut offsets: Vec<wgpu::DynamicOffset> = vec![];

        let mut bind = 0;
        let bind_model_matrix = BindUseModelMatrix::new(bind, bind_model); bind += 1;
        offsets.push(bind_model_matrix.data.data.offset() as wgpu::DynamicOffset);
        key_bindgroup.0.push(bind_model_matrix.key_bind());

        let bind_effect = BindUseEffectValue::new(bind, bind_effect.clone()); bind += 1;
        offsets.push(bind_effect.data.data.offset() as wgpu::DynamicOffset);
        key_bindgroup.0.push(bind_effect.key_bind());

        let bind_skin = match bind_skin {
            Some(bind_skin) => {
                let result = BindUseSkinValue::new(bind, bind_skin); bind += 1;
                offsets.push(result.data.data.offset() as wgpu::DynamicOffset);
                key_bindgroup.0.push(result.key_bind());
                Some(result)
            },
            None => {
                None
            },
        };

        log::info!("RenderBindGroupModel: HHHHHHHHHHH {:?}", offsets);

        if let Some(bindgroup) = RenderBindGroup::get(&key_bindgroup, device, asset_tex, asset_sampler, asset_bindgroup) {
            Some(
                Self {
                    set,
                    bind_model_matrix,
                    bind_effect,
                    bind_skin,
                    key_bindgroup,
                    bindgroup,
                    offsets,
                }
            )
        } else{
            None
        }

    }

    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut entries = vec![];

        self.bind_model_matrix.layout_entry(&mut entries);
        // 当未设置任何参数 会有 4 个 占位u32; 对应MaterialValueBindDesc中也有处理
        self.bind_effect.layout_entry(&mut entries);
        if let Some(bind_skin) = &self.bind_skin {
            bind_skin.layout_entry(&mut entries);
        }

        entries
    }

    pub fn key_model(&self) -> KeyShaderModelAbout {
        if let Some(skin) = &self.bind_skin {
            KeyShaderModelAbout { skin: skin.data().skin.clone() }
        } else {
            KeyShaderModelAbout { skin: ESkinCode::None }
        }
    }
}
impl TShaderBlockCode for RenderBindGroupModel {
    fn vs_define_code(&self) -> String {
        
        let mut result = String::from("");

        result += self.bind_model_matrix.vs_define_code(self.set).as_str();

        result += self.bind_effect.vs_define_code(self.set).as_str();

        if let Some(bind_skin) = &self.bind_skin {
            bind_skin.vs_define_code(self.set).as_str();
        }

        result
    }

    fn fs_define_code(&self) -> String {
        let mut result = String::from("");

        result += self.bind_effect.fs_define_code(self.set).as_str();
        
        result
    }

    fn vs_running_code(&self) -> String {
        let mut result = String::from("");

        if let Some(bind_skin) = &self.bind_skin {
            bind_skin.vs_running_code(self.set).as_str();
        }

        result
    }

    fn fs_running_code(&self) -> String {
        String::from("")
    }
}

impl TBindGroup for RenderBindGroupModel {
    fn bind_group(&self) -> Handle<RenderBindGroup> {
        self.bindgroup.clone()
    }

    fn bindgroup_offsets(
        &self,
    ) -> &Vec<wgpu::DynamicOffset> {
        &self.offsets
    }
}
