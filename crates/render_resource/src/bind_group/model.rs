use render_core::rhi::dyn_uniform_buffer::BufferGroup;

use super::{bind_group::KeyRenderBindgroup};


pub struct BindGroupModelValue {
    pub key: KeyRenderBindgroup,
    pub bind_group: BufferGroup,
}

impl BindGroupModelValue {
    // pub fn new(pool: &mut RenderBindGroupPool, set: &ShaderSetModel) -> Self {
    //     let key = set.layout_entries();
    //     let bind_group = pool.buffer(&super::KeyGroupAlloter(key));

    //     Self {
    //         set: set.set,
    //         bind_group,
    //     }
    // }
}