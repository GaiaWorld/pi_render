use render_core::rhi::{bind_group::BindGroup, bind_group_layout::BindGroupLayout};


pub trait AsMaterialBindGroup {
    const LABEL: &'static str;
    fn bind_group_layout(&self) -> &BindGroupLayout;
    fn bind_group(&self) -> &BindGroup;
}