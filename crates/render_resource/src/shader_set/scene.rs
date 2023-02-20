use std::sync::Arc;

use pi_assets::{asset::Handle, mgr::AssetMgr};
use render_core::rhi::{device::RenderDevice, asset::TextureRes};
use render_shader::{
    shader_set::KeyShaderSceneAbout,
    shader::{TShaderSetCode, TShaderBindCode},
    set_bind::ShaderSetBind,
    buildin_var::ShaderVarUniform
};

use crate::{
    shader_bind::{ShaderBindSceneAboutCamera, ShaderBindSceneAboutTime, ShaderBindSceneAboutFog, TShaderBind, BindUseSceneAboutCamera, BindUseSceneAboutTime, BindUseSceneAboutFog, BindUseSceneAboutEffect, ShaderBindSceneAboutEffect}, bind_group::{bind_group::{KeyRenderBindgroup, RenderBindGroup, AssetMgrRenderBindgroup, TBindGroup}, bind::TKeyBind}, sampler::AssetMgrSampler,
};

use super::TShaderSetLayout;

#[derive(Clone)]
pub struct RenderBindGroupScene {
    pub set: u32,
    pub key: KeyShaderSceneAbout,
    pub bind_camera: BindUseSceneAboutCamera,
    pub bind_effect: BindUseSceneAboutEffect,
    pub bind_brdf_texture: u32,
    pub bind_brdf_sampler: u32,
    pub bind_env_param: u32,
    pub bind_env_texture: u32,
    pub bind_env_sampler: u32,
    pub key_bindgroup: KeyRenderBindgroup,
    bindgroup: Handle<RenderBindGroup>,
    offsets: Vec<wgpu::DynamicOffset>,
}
impl RenderBindGroupScene {
    pub fn new(
        set: u32,
        key: KeyShaderSceneAbout,
        bind_camera: Arc<ShaderBindSceneAboutCamera>,
        bind_effect: Arc<ShaderBindSceneAboutEffect>,
        device: &RenderDevice,
        asset_tex: &AssetMgr<TextureRes>,
        asset_sampler: &AssetMgrSampler,
        asset_bindgroup: &AssetMgrRenderBindgroup,
    ) -> Option<Self> {
        let mut key_bindgroup = KeyRenderBindgroup(vec![]);
        let mut offsets = vec![];
        let mut bind = 0;

        let bind_camera = BindUseSceneAboutCamera::new(bind, bind_camera); bind += 1;
        offsets.push(bind_camera.data.data.start() as wgpu::DynamicOffset);
        key_bindgroup.0.push(bind_camera.key_bind());

        let bind_effect = BindUseSceneAboutEffect::new(bind, bind_effect); bind += 1; 
        offsets.push(bind_effect.data.data.start() as wgpu::DynamicOffset);
        key_bindgroup.0.push(bind_effect.key_bind());

        let (bind_brdf_texture, bind_brdf_sampler) = if key.brdf {
            let result = bind; bind += 2;
            (result, result + 1)
        } else {
            (u32::MAX, u32::MAX)
        };

        let (bind_env_param, bind_env_texture, bind_env_sampler) = if key.env {
            let result = bind; bind += 3; (result, result + 1, result + 2)
        } else {
            (u32::MAX, u32::MAX, u32::MAX)
        };

        if let Some(bindgroup) = RenderBindGroup::get(&key_bindgroup, device, asset_tex, asset_sampler, asset_bindgroup) {
            Some(
                Self {
                    set,
                    key,
                    bind_camera,
                    bind_effect,
                    bind_brdf_texture,
                    bind_brdf_sampler,
                    bind_env_param,
                    bind_env_texture,
                    bind_env_sampler,
                    key_bindgroup,
                    offsets,
                    bindgroup,
                }
            )
        } else {
            None
        }

    }
    pub fn brdf(&self) -> bool {
        self.key.brdf 
    }
    pub fn env(&self) -> bool {
        self.key.env 
    }
    pub fn bind_brdf_texture(&self) -> u32 {
        self.bind_brdf_texture 
    }
    pub fn bind_brdf_sampler(&self) -> u32 {
        self.bind_brdf_sampler 
    }
    pub fn bind_env_param(&self) -> u32 {
        self.bind_env_param 
    }
    pub fn bind_env_texture(&self) -> u32 {
        self.bind_env_texture 
    }
    pub fn bind_env_sampler(&self) -> u32 {
        self.bind_env_sampler 
    }

    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut result = vec![];

        self.bind_camera.layout_entry(&mut result);

        self.bind_effect.layout_entry(&mut result);

        result
    }

    pub fn label(&self) -> &'static str {
        "SceneAbout"
    }

}
impl TShaderSetCode for RenderBindGroupScene {

    fn vs_define_code(&self) -> String {
        let mut result = String::from("");

        result += self.bind_camera.vs_define_code(self.set).as_str();
        result += self.bind_effect.vs_define_code(self.set).as_str();

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
impl TShaderSetLayout for RenderBindGroupScene {
    fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut result = vec![];
        
        self.bind_camera.layout_entry(&mut result);

        self.bind_effect.layout_entry(&mut result);

        result
    }
}
impl TBindGroup for RenderBindGroupScene {
    fn bind_group(&self) -> Handle<RenderBindGroup> {
        self.bindgroup.clone()
    }

    fn bindgroup_offsets(
        &self,
    ) -> &Vec<wgpu::DynamicOffset> {
        &self.offsets
    }
}
