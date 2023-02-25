
use std::sync::Arc;

use pi_assets::{asset::{Handle, Asset}, mgr::AssetMgr};
use pi_share::Share;

use crate::rhi::{device::RenderDevice, bind_group::BindGroup};

use super::{bind::{EKeyBind, EBindData, KeyBindLayout, EBindResource}, ASSET_SIZE_FOR_UNKOWN};

pub const MAX_BIND_COUNT: usize = 16;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindGroupLayout(pub [Option<KeyBindLayout>;MAX_BIND_COUNT]);

#[derive(Debug)]
pub struct BindGroupLayout {
    pub(crate) layout: crate::rhi::bind_group_layout::BindGroupLayout,
}
impl BindGroupLayout {
    pub fn create(
        device: &RenderDevice,
        key: &KeyBindGroupLayout,
        asset_mgr_bind_group_layout: &Share<AssetMgr<BindGroupLayout>>,
    ) -> Option<Handle<Self>> {
        if let Some(layout) = asset_mgr_bind_group_layout.get(key) {
            Some(layout)
        } else {
            let mut entries = vec![];
            key.0.iter().for_each(|v| {
                if let Some(v) = v {
                    entries.push(v.layout_entry());
                }
            });
    
            asset_mgr_bind_group_layout.insert(
                key.clone(), 
                Self {
                    layout: device.create_bind_group_layout(
                        &wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: entries.as_slice(),
                        }
                    )
                }
            )
        }
    }
}
impl Asset for BindGroupLayout {
    type Key = KeyBindGroupLayout;

    fn size(&self) -> usize {
        ASSET_SIZE_FOR_UNKOWN
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindGroup(pub [Option<EKeyBind>;MAX_BIND_COUNT]);

#[derive(Debug, Clone)]
pub struct BindGroupUsage {
    pub(crate) binds: [Option<EKeyBind>;MAX_BIND_COUNT],
    pub(crate) bind_group: Handle<BindGroup>,
    pub(crate) key_bind_group_layout: KeyBindGroupLayout,
    pub(crate) bind_group_layout: Handle<BindGroupLayout>,
}
impl BindGroupUsage {
    pub fn none_binds() -> [Option<EKeyBind>;MAX_BIND_COUNT] {
        [
            None, None, None, None, None, None, None, None, 
            None, None, None, None, None, None, None, None, 
        ]
    }
    pub fn create(
        device: &RenderDevice,
        binds: [Option<EKeyBind>;MAX_BIND_COUNT],
        asset_mgr_bind_group_layout: &Share<AssetMgr<BindGroupLayout>>,
        asset_mgr_bind_group: &Share<AssetMgr<BindGroup>>,
    ) -> Option<Self> {
        let mut resources: Vec<EBindResource> = vec![];

        let mut key_bind_group_layout = KeyBindGroupLayout::default();

        for i in 0..16 {
            if let Some(v) = &binds[i] {
                resources.push(
                    v.bind_source(i as u32)
                );
                key_bind_group_layout.0[i] = Some(v.key_bind_layout());
            }
        }


        let key_bind_group = KeyBindGroup(binds.clone());
        let bind_group = asset_mgr_bind_group.get(&key_bind_group);
        let bind_group_layout = BindGroupLayout::create(device, &key_bind_group_layout, asset_mgr_bind_group_layout);
        if let Some(bind_group) = bind_group {
            if let Some(bind_group_layout) = bind_group_layout {
                Some(
                    Self {
                        binds,
                        bind_group,
                        bind_group_layout,
                        key_bind_group_layout
                    }
                )
            } else {
                None
            }
        } else {
            if let Some(bind_group_layout) = bind_group_layout {
                let mut entries = vec![];
                resources.iter().for_each(|v| {
                    entries.push(v.entry())
                });
        
                let bind_group = device.create_bind_group(
                    &wgpu::BindGroupDescriptor { label: None, layout: &bind_group_layout.layout, entries: entries.as_slice() }
                );
        
                if let Some(bind_group) = asset_mgr_bind_group.insert(key_bind_group, bind_group) {
                    Some(
                        Self {
                            binds,
                            bind_group,
                            bind_group_layout,
                            key_bind_group_layout
                        }
                    )
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn offsets(&self) -> Vec<wgpu::DynamicOffset> {
        let mut result = vec![];
        self.binds.iter().for_each(|v| {
            if let Some(v) = v {
                match v {
                    EKeyBind::Buffer(val) => {
                        result.push(val.data.offset())
                    },
                    EKeyBind::Texture2D(_) => todo!(),
                    EKeyBind::Sampler(_) => todo!(),
                    EKeyBind::Texture2DArray(_) => todo!(),
                }
            }
        });

        result
    }
}
impl Asset for BindGroup {
    type Key = KeyBindGroup;
    fn size(&self) -> usize {
        ASSET_SIZE_FOR_UNKOWN
    }
}

pub struct BindGroupLayoutUsage {
    pub set: u32,
    pub layout: Arc<KeyBindGroupLayout>,
}
