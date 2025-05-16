
use std::{fmt::Debug, hash::{Hash, Hasher}, sync::Arc};

use pi_assets::asset::{Handle, Asset, Size};
use pi_hash::DefaultHasher;

use crate::{rhi::device::RenderDevice, asset::{TAssetKeyU64, ASSET_SIZE_FOR_UNKOWN}};

use super::bind::{EKeyBind, KeyBindLayout};

pub const MAX_BIND_COUNT: usize = 16;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct KeyBindGroupLayout(pub Vec<KeyBindLayout>);
impl KeyBindGroupLayout {
    pub fn entries(&self) -> Vec<wgpu::BindGroupLayoutEntry>  {
        let mut result = vec![];
        let mut index = 0;
        self.0.iter().for_each(|v| {
            result.push(v.layout_entry(index));
            index += 1;
        });

        result
    }
    pub fn layout(&self, device: &RenderDevice) -> crate::rhi::bind_group_layout::BindGroupLayout {
        device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &self.entries().as_slice(),
            }
        )
    }
}
impl TAssetKeyU64 for KeyBindGroupLayout { }


#[derive(Debug)]
pub struct BindGroupLayout {
    pub(crate) layout: crate::rhi::bind_group_layout::BindGroupLayout,
}
impl BindGroupLayout {
    pub fn new(
        device: &RenderDevice,
        key: &KeyBindGroupLayout,
    ) -> Self {
        let entries = key.entries();
        // log::warn!("BindGroupLayout entries {:?}", entries.len());
        Self {
            layout: device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: entries.as_slice(),
                }
            )
        }
    }
    pub fn layout(&self) -> &crate::rhi::bind_group_layout::BindGroupLayout {
        &self.layout
    }
}
impl Asset for BindGroupLayout {
    type Key = u64;
    // const TYPE: &'static str = "BindGroupLayout";
}

impl Size for BindGroupLayout {
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

        Self {
            group: key.bind_group(device, &bind_group_layout),
            layout: bind_group_layout,
        }
    }
}
impl Asset for BindGroup {
    type Key = u64;
    // const TYPE: &'static str = "BindGroup";
}

impl Size for BindGroup {
    fn size(&self) -> usize {
        ASSET_SIZE_FOR_UNKOWN
    }
}

#[derive(Debug, Default, Clone)]
pub struct KeyBindGroup(pub (crate)Arc::<Vec<EKeyBind>>, pub (crate) u64);
impl KeyBindGroup {
    pub fn new(val: Vec<EKeyBind>) -> Self {
        let mut state = DefaultHasher::default();
        val.hash(&mut state);
        let hash = state.finish();
        Self (Arc::new(val), hash)
    }
    pub fn key_bind_group_layout(&self) -> KeyBindGroupLayout {
        let mut result = KeyBindGroupLayout::default();

        self.0.iter().for_each(|v| {
            result.0.push(v.key_bind_layout());
        });

        result
    }
    pub fn bind_group(&self, device: &RenderDevice, layout: &BindGroupLayout) -> crate::rhi::bind_group::BindGroup {
        device.create_bind_group(
            &wgpu::BindGroupDescriptor { label: None, layout: &layout.layout, entries: self.entries().as_slice() }
        )
    }
    fn offsets(&self) -> Vec<wgpu::DynamicOffset> {
        let mut result = vec![];
        self.0.iter().for_each(|v| {
            match v {
                EKeyBind::Buffer(val) => {
                    if val.data.2 {
                        result.push(val.data.offset())
                    }
                },
                EKeyBind::Texture2D(_) => {},
                EKeyBind::Sampler(_) => {},
                // EKeyBind::Texture2DArray(_) => {},
            }
        });

        result
    }
    fn entries(&self) -> Vec<wgpu::BindGroupEntry<'_>> {
        let mut result = vec![];
        let mut binding = 0;
        self.0.iter().for_each(|v| {
            result.push(
                wgpu::BindGroupEntry {
                    binding,
                    resource: v.bind_source(),
                }
            );
            binding += 1;
        });

        result
    }
}
impl Hash for KeyBindGroup {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.1.hash(state);
    }
}
impl PartialEq for KeyBindGroup {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}
impl Eq for KeyBindGroup {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}

impl TAssetKeyU64 for KeyBindGroup {}

pub type KeyBindGroupU64 = u64;

#[derive(Debug, Clone)]
pub struct BindGroupUsage {
    // pub(crate) binds: Arc<IDBinds>,
    pub(crate) key_bind_group: KeyBindGroup,
    pub(crate) bind_group: Handle<BindGroup>,
}
impl BindGroupUsage {
    pub fn new(
        key_bind_group: KeyBindGroup,
        bind_group: Handle<BindGroup>,
    ) -> Self {
        Self { key_bind_group, bind_group }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group.group
    }

    pub fn key(&self) -> KeyBindGroup {
        self.key_bind_group.clone()
    }

    pub fn key_layout(&self) -> KeyBindGroupLayout {
        self.key_bind_group.key_bind_group_layout()
    }

    pub fn layout(&self) -> Handle<BindGroupLayout> {
        self.bind_group.layout.clone()
    }

    pub fn offsets(&self) -> Vec<wgpu::DynamicOffset> {
        self.key_bind_group.offsets()
    }
}

pub struct BindGroupLayoutUsage {
    pub set: u32,
    pub layout: Arc<KeyBindGroupLayout>,
}
