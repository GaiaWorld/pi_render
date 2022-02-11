use std::{ops::Deref, sync::Arc};

#[derive(Clone, Debug)]
pub struct BindGroup(Arc<wgpu::BindGroup>);

impl From<wgpu::BindGroup> for BindGroup {
    fn from(value: wgpu::BindGroup) -> Self {
        BindGroup(Arc::new(value))
    }
}

impl Deref for BindGroup {
    type Target = wgpu::BindGroup;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct BindGroupLayout(Arc<wgpu::BindGroupLayout>);

impl From<wgpu::BindGroupLayout> for BindGroupLayout {
    fn from(value: wgpu::BindGroupLayout) -> Self {
        BindGroupLayout(Arc::new(value))
    }
}

impl Deref for BindGroupLayout {
    type Target = wgpu::BindGroupLayout;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
