use std::sync::Arc;

use pi_assets::{asset::{Handle, Asset, GarbageEmpty}, mgr::AssetMgr};
use pi_hash::XHashMap;
use render_core::rhi::{bind_group::BindGroup, bind_group_layout::BindGroupLayout, dyn_uniform_buffer::{GroupAlloter, BufferGroup}, texture::{Texture, Sampler}, asset::TextureRes, device::RenderDevice};

use crate::uniform_buffer::{KeyBindgroupForValue, KeyBindgroupForTexture};


pub trait AsMaterialBindGroup {
    const LABEL: &'static str;
    fn bind_group_layout(&self) -> &BindGroupLayout;
    fn bind_group(&self) -> &BindGroup;
}

pub struct AssetBindgroupForTexture {
    pub layout: BindGroupLayout,
    pub group: BindGroup,
    pub textures: Vec<(Handle<TextureRes>, Sampler)>,
}
impl Asset for AssetBindgroupForTexture {
    type Key = KeyBindgroupForTexture;

    fn size(&self) -> usize {
        24 + 24 + self.textures.len() * 24
    }
}

pub type AssetMgrBindgroupForTexture = AssetMgr<AssetBindgroupForTexture>;

pub struct RenderBindGroupPool {
    value: XHashMap<KeyBindgroupForValue, GroupAlloter>,
    texture: Arc<AssetMgrBindgroupForTexture>,
}

impl Default for RenderBindGroupPool {
    fn default() -> Self {
        let texture = AssetMgrBindgroupForTexture::new(GarbageEmpty(), false, 4096, 10);

        Self {
            value: XHashMap::default(),
            texture,
        }
    }
}

impl RenderBindGroupPool {
    pub fn texture(
        &self,
        device: &RenderDevice,
        key: &KeyBindgroupForTexture,
        textures: Vec<(Handle<TextureRes>, Sampler)>,
    ) -> Handle<AssetBindgroupForTexture> {
        if let Some(texture) = self.texture.get(key) {
            texture
        } else {
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &key.entries,
            });

            let mut entries = vec![];
            let mut count = 0;
            textures.iter().for_each(|item| {
                entries.push(
                    wgpu::BindGroupEntry {
                        binding: count * 2 + 0,
                        resource: wgpu::BindingResource::TextureView(&item.0.texture_view)
                    }
                );
                entries.push(
                    wgpu::BindGroupEntry {
                        binding: count * 2 + 1,
                        resource: wgpu::BindingResource::Sampler(&item.1)
                    }
                );
                count += 1;
            });

            let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &entries,
            });

            self.texture.insert(key.clone(), AssetBindgroupForTexture {
                layout,
                group,
                textures,
            }).unwrap()
        }
    }

    pub fn buffer(
        &mut self,
        key: &KeyBindgroupForValue,
    ) -> BufferGroup {
        let allocter = if let Some(allocter) = self.value.get(key) {
            allocter
        } else {
            self.value.insert(key.clone(), GroupAlloter::new(None, 16, 65536, Some(4096), key.entries.clone()).unwrap());
            self.value.get(key).unwrap()
        };

        allocter.alloc()
    }
}