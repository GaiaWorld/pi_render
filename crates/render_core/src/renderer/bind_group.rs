
use std::{sync::Arc, fmt::Debug, hash::{Hash, Hasher}};

use pi_assets::{asset::{Handle, Asset}, mgr::AssetMgr};
use pi_hash::DefaultHasher;
use pi_share::Share;

use crate::rhi::{device::RenderDevice};

use super::{bind::{EKeyBind, KeyBindLayout, EBindResource}, ASSET_SIZE_FOR_UNKOWN};

pub const MAX_BIND_COUNT: usize = 16;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindGroupLayout(pub [Option<KeyBindLayout>;MAX_BIND_COUNT]);
impl KeyBindGroupLayout {
    pub fn new(binds: &[Option<EKeyBind>;MAX_BIND_COUNT]) -> Self {
        let mut key_bind_group_layout = KeyBindGroupLayout::default();

        for i in 0..16 {
            if let Some(v) = &binds[i] {
                key_bind_group_layout.0[i] = Some(v.key_bind_layout());
            }
        }

        key_bind_group_layout
    }
    pub fn as_u64(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug)]
pub struct BindGroupLayout {
    pub(crate) layout: crate::rhi::bind_group_layout::BindGroupLayout,
}
impl BindGroupLayout {
    pub fn new(
        device: &RenderDevice,
        key: &KeyBindGroupLayout,
    ) -> Self {
        let mut entries = vec![];
        key.0.iter().for_each(|v| {
            if let Some(v) = v {
                entries.push(v.layout_entry());
            }
        });
        Self {
            layout: device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: entries.as_slice(),
                }
            )
        }
    }
}
impl Asset for BindGroupLayout {
    type Key = u64;

    fn size(&self) -> usize {
        ASSET_SIZE_FOR_UNKOWN
    }
}

#[derive(Debug)]
pub struct BindGroup {
    pub(crate) group: crate::rhi::bind_group::BindGroup,
    pub(crate) layout: Handle<BindGroupLayout>,
}
impl BindGroup {
    pub fn new(device: &RenderDevice, key: &KeyBindGroup, bind_group_layout: Handle<BindGroupLayout>) -> Self {
        let mut resources: Vec<EBindResource> = vec![];
        let mut entries = vec![];
        for i in 0..16 {
            if let Some(v) = &key.0[i] {
                let val = v.bind_source(i as u32);
                resources.push(val);
            }
        }
        resources.iter().for_each(|v| {
            entries.push(v.entry())
        });
        
        Self {
            group: device.create_bind_group(
                &wgpu::BindGroupDescriptor { label: None, layout: &bind_group_layout.layout, entries: entries.as_slice() }
            ),
            layout: bind_group_layout,
        }
    }
}
impl Asset for BindGroup {
    type Key = u64;

    fn size(&self) -> usize {
        ASSET_SIZE_FOR_UNKOWN
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindGroup(pub [Option<EKeyBind>;MAX_BIND_COUNT]);
impl KeyBindGroup {
    pub fn new(binds: [Option<EKeyBind>;MAX_BIND_COUNT]) -> Self {
        Self(binds)
    }
    pub fn key_bind_group_layout(&self) -> KeyBindGroupLayout {
        KeyBindGroupLayout::new(&self.0)
    }
    pub fn as_u64(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[derive(Debug, Clone)]
pub struct BindGroupUsage {
    pub(crate) set: u32,
    pub(crate) key_bind_group: KeyBindGroup,
    pub(crate) bind_group: Handle<BindGroup>,
}
impl BindGroupUsage {
    pub fn none_binds() -> [Option<EKeyBind>;MAX_BIND_COUNT] {
        [
            None, None, None, None, None, None, None, None, 
            None, None, None, None, None, None, None, None, 
        ]
    }
    pub fn new(
        set: u32,
        key_bind_group: KeyBindGroup,
        bind_group: Handle<BindGroup>,
    ) -> Self {
        Self { set, key_bind_group, bind_group }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group.group
    }

    pub fn key_layout(&self) -> KeyBindGroupLayout {
        let mut key_bind_group_layout = KeyBindGroupLayout::default();

        for i in 0..16 {
            if let Some(v) = &self.key_bind_group.0[i] {
                key_bind_group_layout.0[i] = Some(v.key_bind_layout());
            }
        }

        key_bind_group_layout
    }

    pub fn layout(&self) -> Handle<BindGroupLayout> {
        self.bind_group.layout.clone()
    }

    pub fn offsets(&self) -> Vec<wgpu::DynamicOffset> {
        let mut result = vec![];
        self.key_bind_group.0.iter().for_each(|v| {
            if let Some(v) = v {
                match v {
                    EKeyBind::Buffer(val) => {
                        result.push(val.data.offset())
                    },
                    EKeyBind::Texture2D(_) => {},
                    EKeyBind::Sampler(_) => {},
                    EKeyBind::Texture2DArray(_) => {},
                }
            }
        });

        result
    }
}

pub struct BindGroupLayoutUsage {
    pub set: u32,
    pub layout: Arc<KeyBindGroupLayout>,
}
