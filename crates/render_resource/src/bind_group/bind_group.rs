use std::num::NonZeroU64;

use pi_assets::{asset::{Handle, Asset}, mgr::AssetMgr};
use render_core::rhi::{bind_group_layout::BindGroupLayout, bind_group::BindGroup, asset::TextureRes, texture::Sampler, device::RenderDevice, buffer::Buffer};

use crate::{sampler::{AssetMgrSampler, AssetSampler}, buffer::dyn_mergy_buffer::DynMergyBufferRange};

use super::bind::KeyBind;

pub trait TGetKeyBindGroup {

}

pub struct BindGroupInfo {
    layouts: Vec<wgpu::BindGroupLayoutEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct KeyRenderBindgroup(pub Vec<KeyBind>);
impl KeyRenderBindgroup {
    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut result = vec![];
        self.0.iter().for_each(|val| {
            result.push(val.layout_entry());
        });
        result
    }
}

pub type AssetMgrRenderBindgroup = AssetMgr<RenderBindGroup>;

#[derive(Clone)]
pub enum TempBindData {
    Buffer(u32, Buffer, usize),
    Texture(u32, Handle<TextureRes>),
    Sampler(u32, Handle<AssetSampler>),
}

#[derive(Clone)]
pub struct RenderBindGroup {
    pub layout: BindGroupLayout,
    pub group: BindGroup,
    pub data: Vec<TempBindData>,
}
impl Asset for RenderBindGroup {
    type Key = KeyRenderBindgroup;
    fn size(&self) -> usize {
        1024
    }
}
impl RenderBindGroup {
    pub fn new(
        key: &KeyRenderBindgroup,
        device: &RenderDevice,
        asset_tex: &AssetMgr<TextureRes>,
        asset_sampler: &AssetMgrSampler,
    ) -> Option<Self> {
        let mut tempdata = vec![]; 
        let mut entries = vec![];
        let mut len = key.0.len();
        for index in 0..len  {
            let val = key.0.get(index).unwrap();
            match val {
                KeyBind::Buffer(val) => {
                    tempdata.push(TempBindData::Buffer(val.bind, val.id_buffer.buffer().clone(), val.id_buffer.size()));
                },
                KeyBind::Texture(val) => {
                    if let Some(texture) = asset_tex.get(&(val.id_texture.get_hash() as u64)) {
                        tempdata.push(TempBindData::Texture(val.bind, texture));
                    } else {
                        log::warn!("Not Found Texture {:?}", val.id_texture);
                        return None;
                    }
                },
                KeyBind::Sampler(val) => {
                    if let Some(sampler) = asset_sampler.get(&val.id_sampler) {
                        tempdata.push(TempBindData::Sampler(val.bind, sampler));
                    } else {
                        let sampler = device.create_sampler(&val.id_sampler.to_sampler_description());
                        if let Some(sampler) = asset_sampler.insert(val.id_sampler.clone(), AssetSampler(sampler)) {
                            tempdata.push(TempBindData::Sampler(val.bind, sampler));
                        } else {
                            log::warn!("Not Found Sampler {:?}", val.id_sampler);
                            return None;
                        }
                    }
                },
            }
        }

        let layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &key.layout_entries(),
            }
        );

        for index in 0..len  {
            let val = tempdata.get(index).unwrap();
            match val {
                TempBindData::Buffer(bind, val, size) => {
                    entries.push(
                        wgpu::BindGroupEntry {
                            binding: *bind,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: val, offset: 0, size: NonZeroU64::new(*size as u64) }),
                        }
                    );
                },
                TempBindData::Texture(bind, val) => {
                    entries.push(
                        wgpu::BindGroupEntry {
                            binding: *bind,
                            resource: wgpu::BindingResource::TextureView(&val.texture_view),
                        }
                    );
                },
                TempBindData::Sampler(bind, val) => {
                    entries.push(
                        wgpu::BindGroupEntry {
                            binding: *bind,
                            resource: wgpu::BindingResource::Sampler(&val.0),
                        }
                    );
                },
            }
        }

        let bindgoup = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &entries,
            }
        );

        Some(
            Self {
                layout,
                data: tempdata,
                group: bindgoup,
            }
        )
    }

    pub fn get(
        key_bindgroup: &KeyRenderBindgroup,
        device: &RenderDevice,
        asset_tex: &AssetMgr<TextureRes>,
        asset_sampler: &AssetMgrSampler,
        asset_bindgroup: &AssetMgrRenderBindgroup,
    ) -> Option<Handle<Self>> {
        if let Some(bindgroup) = asset_bindgroup.get(key_bindgroup) {
            Some(bindgroup)
        } else {
            if let Some(bindgroup) = RenderBindGroup::new(key_bindgroup, device, asset_tex, asset_sampler) {
                asset_bindgroup.insert(key_bindgroup.clone(), bindgroup)
            } else{
                None
            }
        }
    }
}


pub trait TBindGroup {
    fn bind_group(&self) -> Handle<RenderBindGroup>;
    fn bindgroup_offsets(
        &self,
    ) -> &Vec<wgpu::DynamicOffset>;
}
