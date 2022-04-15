//! Pass2D Entity

pub mod camera;
pub mod draw_object;

use self::{
    camera::Camera2D,
    draw_object::{DrawObjectArchetype, DrawState},
};
use crate::{
    components::view::target::{RenderTarget, RenderTargetKey, RenderTargets, TextureViews},
    graph::{
        node::{Node, NodeRunError, RealValue},
        node_slot::SlotInfo,
        RenderContext,
    },
    rhi::CommandEncoder,
};
use futures::{future::BoxFuture, FutureExt};
use pi_ecs::{
    entity::Entity,
    prelude::{QueryState, World},
};
use pi_share::ShareRefCell;

/// Pass2D 原型，描述 将 2D物件 渲染到 指定 渲染目标的流程
/// 用指定 Camera2D 的 视图矩阵 和 投影矩阵
/// 输出到 RenderTargetKey 指定的 渲染目标
/// 挨个 渲染 不透明物件 Opaque2D
/// 挨个 渲染 半透明物件 Transparent2D
pub struct Pass2DArchetype;

// 渲染 物件 列表
pub struct Draw2DList {
    /// 不透明 列表
    /// 注：渲染时，假设 Vec已经 排好序 了
    // Entity 是 DrawObjectArchetype
    pub opaque: Vec<Entity>,

    /// 透明 列表
    /// 注：渲染时，假设 Vec已经 排好序 了
    /// Entity 是 DrawObjectArchetype
    pub transparent: Vec<Entity>,
}

impl Default for Draw2DList {
    fn default() -> Self {
        Self {
            opaque: Vec::default(),
            transparent: Vec::default(),
        }
    }
}

/// 初始化 ECS
pub fn init_ecs(world: &mut World) {
    world
        .new_archetype::<Pass2DArchetype>()
        .register::<Camera2D>()
        .register::<RenderTargetKey>()
        .register::<Draw2DList>()
        .create();
}

pub struct Pass2DNode;

impl Node for Pass2DNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn output(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn prepare(
        &self,
        _context: RenderContext,
        _inputs: &[Option<RealValue>],
        _outputs: &[Option<RealValue>],
    ) -> Option<BoxFuture<'static, Result<(), NodeRunError>>> {
        None
    }

    fn run(
        &self,
        context: RenderContext,
        mut commands: ShareRefCell<CommandEncoder>,
        _inputs: &[Option<RealValue>],
        _outputs: &[Option<RealValue>],
    ) -> BoxFuture<'static, Result<(), NodeRunError>> {
        let RenderContext { mut world, .. } = context;

        let mut pass_query = QueryState::<
            Pass2DArchetype,
            (
                &'static Camera2D,
                &'static RenderTargetKey,
                &'static Draw2DList,
            ),
        >::new(&mut world);

        let draw_query = QueryState::<DrawObjectArchetype, &'static DrawState>::new(&mut world);

        async move {
            let rts = world.get_resource::<RenderTargets>().unwrap();
            let views = world.get_resource::<TextureViews>().unwrap();

            for (camera, rt_key, list) in pass_query.iter(&world) {
                let rt = rts.get(*rt_key).unwrap();
                let RenderTarget { colors, .. } = rt;
                let color_attachments = colors
                    .iter()
                    .map(|view| {
                        let view = views.get(*view).unwrap();
                        wgpu::RenderPassColorAttachment {
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                            view: view.as_ref().unwrap(),
                        }
                    })
                    .collect::<Vec<wgpu::RenderPassColorAttachment>>();

                // TODO Detph-Stencil
                let depth_stencil_attachment = None;

                let mut rp = commands.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &color_attachments,
                    depth_stencil_attachment,
                });

                // 渲染不透明
                for e in &list.opaque {
                    if let Some(state) = draw_query.get(&world, *e) {
                        state.draw(&mut rp, camera);
                    }
                }

                // 渲染透明
                for e in &list.transparent {
                    if let Some(state) = draw_query.get(&world, *e) {
                        state.draw(&mut rp, camera);
                    }
                }
            }

            Ok(())
        }
        .boxed()
    }
}
