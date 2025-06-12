use pi_share::Share;
use std::ops::Deref;
use uuid::Uuid;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub struct BindGroupId(Uuid);

#[derive(Clone, Debug)]
pub struct BindGroup {
    id: BindGroupId,
    value: Share<wgpu::BindGroup>,
}

impl BindGroup {
    #[inline]
    pub fn id(&self) -> BindGroupId {
        self.id
    }
    
    #[inline]
    pub fn value(&self) -> &wgpu::BindGroup {
        self.value.as_ref()
    }
}

impl From<wgpu::BindGroup> for BindGroup {
    fn from(value: wgpu::BindGroup) -> Self {
        BindGroup {
            id: BindGroupId(Uuid::new_v4()),
            value: Share::new(value),
        }
    }
}

impl Deref for BindGroup {
    type Target = wgpu::BindGroup;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
