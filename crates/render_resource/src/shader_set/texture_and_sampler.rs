use std::sync::Arc;
use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_share::Share;
use render_core::rhi::{device::RenderDevice, asset::TextureRes};
use render_shader::{
    shader_set::{KeyShaderSceneAbout, KeyShaderModelAbout},
    shader::{TShaderSetCode, TShaderBindCode},
    set_bind::ShaderSetBind,
    buildin_var::ShaderVarUniform, unifrom_code::{UniformSamplerDesc, UniformTextureDesc, EffectUniformTextureWithSamplerUseinfo}, skin_code::ESkinCode
};

use crate::{
    shader_bind::{ShaderBindTexture, ShaderBindSampler, TShaderBind, BindUseTextureWithSampler, BindUseTexture, BindUseSampler, EffectTextureAndSamplerBinds}, bind_group::{bind_group::{TBindGroup, RenderBindGroup, KeyRenderBindgroup, AssetMgrRenderBindgroup}, bind::{KeyBindTexture, TKeyBind}}, sampler::AssetMgrSampler,
};

pub trait TUniformSamplerDesc {
    fn uniform_sampler_desc(&self) -> Arc<UniformSamplerDesc>;
}
impl TUniformSamplerDesc for Arc<UniformSamplerDesc> {
    fn uniform_sampler_desc(&self) -> Arc<UniformSamplerDesc> {
        self.clone()
    }
}
// impl TUniformSamplerDesc for UniformTextureWithSamplerUseinfo {
//     fn uniform_sampler_desc(&self) -> Arc<UniformSamplerDesc> {
//         Arc::new(
//                 UniformSamplerDesc {
//                 slotname: UniformPropertyName::from(String::from("sampler") + self.texture_uniform.slotname.as_str()),
//                 ty: if self.param.filter { wgpu::SamplerBindingType::Filtering } else { wgpu::SamplerBindingType::NonFiltering },
//                 stage: self.texture_uniform.stage,
//             }
//         )
//     } 
// }

pub trait TUniformTextureDesc {
    fn uniform_texture_desc(&self) -> Arc<UniformTextureDesc>;
}
impl TUniformTextureDesc for Arc<UniformTextureDesc> {
    fn uniform_texture_desc(&self) -> Arc<UniformTextureDesc> {
        self.clone()
    }
}

pub struct EffectTextureDesc(pub Vec<()>);

/// * 纹理bindgroup
/// * 包含效果纹理 包含功能纹理
#[derive(Clone)]
pub struct RenderBindGroupTextureSamplers {
    pub set: u32,
    pub bind_tex_skin: Option<BindUseTextureWithSampler>,
    pub bind_textures: Vec<BindUseTextureWithSampler>,
    key_bindgroup: KeyRenderBindgroup,
    bindgroup: Handle<RenderBindGroup>,
    offsets: Vec<wgpu::DynamicOffset>,
}
impl RenderBindGroupTextureSamplers {
    pub fn new(
        set: u32,
        // key_model: &KeyShaderModelAbout,
        effect_textures: &EffectTextureAndSamplerBinds,
        device: &RenderDevice,
        asset_tex: &Share<AssetMgr<TextureRes>>,
        asset_sampler: &Share<AssetMgrSampler>,
        asset_bindgroup: &Share<AssetMgrRenderBindgroup>,
    ) -> Option<Self> {
        let offsets = vec![];

        let mut bind = 0;
        let mut key_bindgroup = KeyRenderBindgroup(vec![]);

        let mut bind_textures = vec![];
        effect_textures.list.iter().for_each(|useinfo| {
            let bind_texture = bind; bind += 1;
            let bind_sampler = bind; bind += 1;
            let item = BindUseTextureWithSampler(
                BindUseTexture { bind: bind_texture, data: useinfo.0.clone() },
                BindUseSampler { bind: bind_sampler, data: useinfo.1.clone() }
            );
            key_bindgroup.0.push(item.0.key_bind());
            key_bindgroup.0.push(item.1.key_bind());
            bind_textures.push(item);
        });

        if bind_textures.len() == 0 {
            None
        } else {
            if let Some(bindgroup) = RenderBindGroup::get(&key_bindgroup, device, asset_tex, asset_sampler, asset_bindgroup) {
                Some(
                    Self {
                        set,
                        bind_tex_skin: None,
                        bind_textures,
                        key_bindgroup,
                        bindgroup,
                        offsets,
                    }
                )
            } else{
                None
            }
        }
        // let bind_model_skin = match key_model.skin {
        //     ESkinCode::None => {
        //         None
        //     },
        //     ESkinCode::UBO(_, _) => {
        //         None
        //     },
        //     _ => {
        //         let bind_model_skin_tex = bind; bind += 1;
        //         let bind_model_skin_samp = bind; bind += 1;
        //         Some(
        //             BindUseTextureWithSampler(
        //                 BindUseTexture { bind: bind_model_skin_tex, data: ESkinCode::uniform_desc_tex() },
        //                 BindUseSampler { bind: bind_model_skin_samp, data: ESkinCode::uniform_desc_sampler() }
        //             )
        //         )
        //     }
        // };
    }

    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut entries = vec![];

        self.bind_textures.iter().for_each(|item| {
            item.0.layout_entry(&mut entries);
            item.1.layout_entry(&mut entries);
        });
        // self.bind_samplers.iter().for_each(|sampler| {
        //     sampler.layout_entry(&mut entries);
        // });

        if let Some(item) = &self.bind_tex_skin {
            item.0.layout_entry(&mut entries);
            item.1.layout_entry(&mut entries);
        }

        entries
    }
}
impl TShaderSetCode for RenderBindGroupTextureSamplers {
    fn vs_define_code(&self) -> String {
        let mut result = String::from("");

        self.bind_textures.iter().for_each(|item| {
            result += item.0.vs_define_code(self.set).as_str();
            result += item.1.vs_define_code(self.set).as_str();
        });

        if let Some(item) = &self.bind_tex_skin {
            result += item.0.vs_define_code(self.set).as_str();
            result += item.1.vs_define_code(self.set).as_str();
        }

        result
    }

    fn fs_define_code(&self) -> String {
        let mut result = String::from("");

        self.bind_textures.iter().for_each(|item| {
            result += item.0.fs_define_code(self.set).as_str();
            result += item.1.fs_define_code(self.set).as_str();
        });

        if let Some(item) = &self.bind_tex_skin {
            result += item.0.fs_define_code(self.set).as_str();
            result += item.1.fs_define_code(self.set).as_str();
        }

        result
    }

    fn vs_running_code(&self) -> String {
        String::from("")
    }

    fn fs_running_code(&self) -> String {
        String::from("")
    }
}

impl TBindGroup for RenderBindGroupTextureSamplers {
    fn bind_group(&self) -> Handle<RenderBindGroup> {
        self.bindgroup.clone()
    }

    fn bindgroup_offsets(
        &self,
    ) -> &Vec<wgpu::DynamicOffset> {
        &self.offsets
    }
}
