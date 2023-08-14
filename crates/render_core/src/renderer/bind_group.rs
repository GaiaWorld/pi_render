
use std::{sync::Arc, fmt::Debug, hash::{Hash}};

use pi_assets::{asset::{Handle, Asset, Size}};

use crate::{rhi::{device::RenderDevice, small_struct_allocator::{SmallStructAllocatorPool, IDSmallStruct, TSmallStructID}}, asset::{TAssetKeyU64, ASSET_SIZE_FOR_UNKOWN}};

use super::{bind::{EKeyBind, EBindResource}};

pub const MAX_BIND_COUNT: usize = 16;

pub trait TBinds {
    fn val(&self) -> &[Option<EKeyBind>];
    fn val_mut(&mut self) -> &mut [Option<EKeyBind>];
    fn offsets(&self) -> Vec<wgpu::DynamicOffset> {
        let mut result = vec![];
        self.val().iter().for_each(|v| {
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
    fn bind_sources(&self) -> Vec<EBindResource> {
        let mut result = vec![];
        let mut index = 0;
        self.val().iter().for_each(|v| {
            if let Some(v) = v {
                result.push(v.bind_source(index));
                index += 1;
            }
        });

        result
    }
    fn entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut result = vec![];
        let mut index = 0;
        self.val().iter().for_each(|v| {
            if let Some(v) = v {
                result.push(v.key_bind_layout().layout_entry());
                index += 1;
            }
        });

        result
    }
}

pub struct Binds01([Option<EKeyBind>;1]);
impl TBinds for Binds01 {
    fn val(&self) -> &[Option<EKeyBind>] { &self.0 }
    fn val_mut(&mut self) -> &mut [Option<EKeyBind>] { &mut self.0 }
}
impl Default for Binds01 { 
    fn default() -> Self {
        Self([None])
    }
}
impl TSmallStructID for Binds01 { const ID: u32 = 01; }

pub struct Binds02([Option<EKeyBind>;2]);
impl TBinds for Binds02 {
    fn val(&self) -> &[Option<EKeyBind>] { &self.0 }
    fn val_mut(&mut self) -> &mut [Option<EKeyBind>] { &mut self.0 }
}
impl Default for Binds02 { 
    fn default() -> Self {
        Self([None, None])
    }
}
impl TSmallStructID for Binds02 { const ID: u32 = 02; }

pub struct Binds04([Option<EKeyBind>;4]);
impl TBinds for Binds04 {
    fn val(&self) -> &[Option<EKeyBind>] { &self.0 }
    fn val_mut(&mut self) -> &mut [Option<EKeyBind>] { &mut self.0 }
}
impl Default for Binds04 { 
    fn default() -> Self {
        Self([None, None, None, None])
    }
}
impl TSmallStructID for Binds04 { const ID: u32 = 04; }

pub struct Binds08([Option<EKeyBind>;8]);
impl TBinds for Binds08 {
    fn val(&self) -> &[Option<EKeyBind>] { &self.0 }
    fn val_mut(&mut self) -> &mut [Option<EKeyBind>] { &mut self.0 }
}
impl Default for Binds08 { 
    fn default() -> Self {
        Self([None, None, None, None, None, None, None, None])
    }
}
impl TSmallStructID for Binds08 { const ID: u32 = 08; }

pub struct Binds16([Option<EKeyBind>;16]);
impl TBinds for Binds16 {
    fn val(&self) -> &[Option<EKeyBind>] { &self.0 }
    fn val_mut(&mut self) -> &mut [Option<EKeyBind>] { &mut self.0 }
}
impl Default for Binds16 { 
    fn default() -> Self {
        Self([None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None])
    }
}
impl TSmallStructID for Binds16 { const ID: u32 = 16; }

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum IDBinds {
    Binds00(Vec<wgpu::DynamicOffset>),
    Binds01(IDSmallStruct<Binds01>, Vec<wgpu::DynamicOffset>),
    Binds02(IDSmallStruct<Binds02>, Vec<wgpu::DynamicOffset>),
    Binds04(IDSmallStruct<Binds04>, Vec<wgpu::DynamicOffset>),
    Binds08(IDSmallStruct<Binds08>, Vec<wgpu::DynamicOffset>),
    Binds16(IDSmallStruct<Binds16>, Vec<wgpu::DynamicOffset>),
}
impl IDBinds {
    
    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        match self {
            IDBinds::Binds00(_) => vec![],
            IDBinds::Binds01(id, _) => {
                id.data().unwrap().entries()
            },
            IDBinds::Binds02(id, _) =>  {
                id.data().unwrap().entries()
            }
            IDBinds::Binds04(id, _) =>  {
                id.data().unwrap().entries()
            },
            IDBinds::Binds08(id, _) =>  {
                id.data().unwrap().entries()
            },
            IDBinds::Binds16(id, _) =>  {
                id.data().unwrap().entries()
            },
        }
    }
    pub fn bind_sources(&self) -> Vec<EBindResource> {
        match self {
            IDBinds::Binds00(_) => vec![],
            IDBinds::Binds01(id, _) => {
                id.data().unwrap().bind_sources()
            },
            IDBinds::Binds02(id, _) =>  {
                id.data().unwrap().bind_sources()
            }
            IDBinds::Binds04(id, _) =>  {
                id.data().unwrap().bind_sources()
            },
            IDBinds::Binds08(id, _) =>  {
                id.data().unwrap().bind_sources()
            },
            IDBinds::Binds16(id, _) =>  {
                id.data().unwrap().bind_sources()
            },
        }
    }
    pub fn offsets(&self) -> &Vec<wgpu::DynamicOffset> {
        match self {
            IDBinds::Binds00(v) => v,
            IDBinds::Binds01(id, v) => {
                v
            },
            IDBinds::Binds02(id, v) =>  {
                v
            }
            IDBinds::Binds04(id, v) =>  {
                v
            },
            IDBinds::Binds08(id, v) =>  {
                v
            },
            IDBinds::Binds16(id, v) =>  {
                v
            },
        }
    }
    pub fn binds(&self) -> Vec<Option<EKeyBind>> {
        match self {
            IDBinds::Binds00(v) => {
                vec![]
            },
            IDBinds::Binds01(id, v) => {
                id.data().unwrap().val().to_vec()
            },
            IDBinds::Binds02(id, v) =>  {
                id.data().unwrap().val().to_vec()
            }
            IDBinds::Binds04(id, v) =>  {
                id.data().unwrap().val().to_vec()
            },
            IDBinds::Binds08(id, v) =>  {
                id.data().unwrap().val().to_vec()
            },
            IDBinds::Binds16(id, v) =>  {
                id.data().unwrap().val().to_vec()
            },
        }
    }
} 
impl TAssetKeyU64 for IDBinds {

}

pub enum EBinds {
    Binds01(Binds01),
    Binds02(Binds02),
    Binds04(Binds04),
    Binds08(Binds08),
    Binds16(Binds16),
}
impl EBinds {
    pub fn new(count: u32) -> Option<Self> {
        if count == 0 {
            None
        } else if count == 1 {
            Some(Self::Binds01(Binds01::default()))
        } else if count <= 2 {
            Some(Self::Binds02(Binds02::default()))
        } else if count <= 4 {
            Some(Self::Binds04(Binds04::default()))
        } else if count <= 8 {
            Some(Self::Binds08(Binds08::default()))
        } else if count <= 16 {
            Some(Self::Binds16(Binds16::default()))
        } else {
            None
        }
    }
    pub fn set(&mut self, bind: usize, val: Option<EKeyBind>) {
        match self {
            EBinds::Binds01(item) => {
                if bind < 1 {
                    item.0[bind] = val;
                }
            },
            EBinds::Binds02(item) => {
                if bind < 02 {
                    item.0[bind] = val;
                }
            },
            EBinds::Binds04(item) => {
                if bind < 04 {
                    item.0[bind] = val;
                }
            },
            EBinds::Binds08(item) => {
                if bind < 08 {
                    item.0[bind] = val;
                }
            },
            EBinds::Binds16(item) => {
                if bind < 16 {
                    item.0[bind] = val;
                }
            },
        }
    }
    pub fn record(self, recorder: &mut BindsRecorder) -> Arc<IDBinds> {
        let id = match self {
            EBinds::Binds01(val) => {
                recorder.record_01(val)
            },
            EBinds::Binds02(val) => {
                recorder.record_02(val)
            },
            EBinds::Binds04(val) => {
                recorder.record_04(val)
            },
            EBinds::Binds08(val) => {
                recorder.record_08(val)
            },
            EBinds::Binds16(val) => {
                recorder.record_16(val)
            },
        };

        if let Some(id) = id {
            Arc::new(id)
        } else {
            Arc::new(IDBinds::Binds00(vec![]))
        }
    }
}

pub struct BindsRecorder {
    pool_01: SmallStructAllocatorPool<Binds01>,
    pool_02: SmallStructAllocatorPool<Binds02>,
    pool_04: SmallStructAllocatorPool<Binds04>,
    pool_08: SmallStructAllocatorPool<Binds08>,
    pool_16: SmallStructAllocatorPool<Binds16>,
}
impl BindsRecorder {
    pub fn new() -> Self {
        let item_size = 32;
        Self {
            pool_01: SmallStructAllocatorPool::<Binds01>::new(32 * 16, item_size),
            pool_02: SmallStructAllocatorPool::<Binds02>::new(32 * 08, item_size),
            pool_04: SmallStructAllocatorPool::<Binds04>::new(32 * 04, item_size),
            pool_08: SmallStructAllocatorPool::<Binds08>::new(32 * 02, item_size),
            pool_16: SmallStructAllocatorPool::<Binds16>::new(32 * 01, item_size),
        }
    }
    pub fn recycle(&mut self) {
        self.pool_01.recycle();
        self.pool_02.recycle();
        self.pool_04.recycle();
        self.pool_08.recycle();
        self.pool_16.recycle();
    }
    pub fn record_01(&mut self, val: Binds01) -> Option<IDBinds> {
        if let Some(id) = self.pool_01.allocate(val) {
            let v = id.data().unwrap().offsets();
            Some(IDBinds::Binds01(id, v))
        } else {
            None
        }
    }
    pub fn record_02(&mut self, val: Binds02) -> Option<IDBinds> {
        if let Some(id) = self.pool_02.allocate(val) {
            let v = id.data().unwrap().offsets();
            Some(IDBinds::Binds02(id, v))
        } else {
            None
        }
    }
    pub fn record_04(&mut self, val: Binds04) -> Option<IDBinds> {
        if let Some(id) = self.pool_04.allocate(val) {
            let v = id.data().unwrap().offsets();
            Some(IDBinds::Binds04(id, v))
        } else {
            None
        }
    }
    pub fn record_08(&mut self, val: Binds08) -> Option<IDBinds> {
        if let Some(id) = self.pool_08.allocate(val) {
            let v = id.data().unwrap().offsets();
            Some(IDBinds::Binds08(id, v))
        } else {
            None
        }
    }
    pub fn record_16(&mut self, val: Binds16) -> Option<IDBinds> {
        if let Some(id) = self.pool_16.allocate(val) {
            let v = id.data().unwrap().offsets();
            Some(IDBinds::Binds16(id, v))
        } else {
            None
        }
    }
}

pub type KeyBindGroupLayout = KeyBindGroup;

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
}
impl Asset for BindGroupLayout {
    type Key = u64;
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
        let resources: Vec<EBindResource> = key.bind_sources();
        let mut entries = vec![];
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
}

impl Size for BindGroup {
    fn size(&self) -> usize {
        ASSET_SIZE_FOR_UNKOWN
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct KeyBindGroup(pub Vec<Option<EKeyBind>>);
impl KeyBindGroup {
    pub fn bind_sources(&self) -> Vec<EBindResource> {
        let mut result = vec![];
        let mut index = 0;
        self.0.iter().for_each(|v| {
            if let Some(v) = v {
                result.push(v.bind_source(index));
                index += 1;
            }
        });

        result
    }
    pub fn entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut result = vec![];
        let mut index = 0;
        self.0.iter().for_each(|v| {
            if let Some(v) = v {
                result.push(v.key_bind_layout().layout_entry());
                index += 1;
            }
        });

        result
    }
}
impl TAssetKeyU64 for KeyBindGroup {}

pub type KeyBindGroupU64 = u64;

#[derive(Debug, Clone)]
pub struct BindGroupUsage {
    pub(crate) set: u32,
    pub(crate) binds: Arc<IDBinds>,
    pub(crate) key_bind_group: KeyBindGroup,
    pub(crate) bind_group: Handle<BindGroup>,
}
impl BindGroupUsage {
    pub fn new(
        set: u32,
        binds: Arc<IDBinds>,
        bind_group: Handle<BindGroup>,
    ) -> Self {
        let key_bind_group = KeyBindGroup(binds.binds());
        Self { set, binds, key_bind_group, bind_group }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group.group
    }

    pub fn key_layout(&self) -> KeyBindGroupLayout {
        KeyBindGroup(self.key_bind_group.0.clone())
    }

    pub fn layout(&self) -> Handle<BindGroupLayout> {
        self.bind_group.layout.clone()
    }

    pub fn offsets(&self) -> &Vec<wgpu::DynamicOffset> {
        self.binds.offsets()
    }
}

pub struct BindGroupLayoutUsage {
    pub set: u32,
    pub layout: Arc<KeyBindGroupLayout>,
}
