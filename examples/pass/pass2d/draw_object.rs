//! Pass2D Entity

use std::sync::Arc;

use super::camera::Camera2D;
use pi_render::rhi::{bind_group::BindGroup, buffer::Buffer, pipeline::RenderPipeline, IndexFormat};
use pi_ecs::prelude::World;
use pi_map::vecmap::VecMap;
use wgpu::RenderPass;

pub struct DrawObject;

#[derive(Debug, Default)]
pub struct DrawState {
    // 一个 Pipeleine
    pub pipeline: Option<Arc<RenderPipeline>>,

    // 一堆 UBO
    pub bind_groups: VecMap<Arc<BindGroup>>,

    // 一堆 VB
    pub vbs: VecMap<(Arc<Buffer>, u64)>,

    // IB 可有 可无
    pub ib: Option<(Arc<Buffer>, u64, IndexFormat)>,
}

/// 初始化 ECS
pub fn init_ecs(world: &mut World) {
    world
        .new_archetype::<DrawObject>()
        .register::<DrawState>()
        .create();
}

impl DrawState {
    pub fn draw<'w>(&self, _rp: &mut RenderPass<'w>, _camera: &Camera2D) {
        // 在这里 写 wgpu 的 指令
        todo!()
    }
}