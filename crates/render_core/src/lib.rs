//! 基于 ECS 框架的 渲染库
//! 提供 rhi封装 和 渲染图

pub mod camera;
pub mod color;
pub mod render_graph;
pub mod render_nodes;
pub mod rhi;
pub mod texture;
pub mod view;
pub mod window;

use camera::init_camera;
use futures::{future::BoxFuture, FutureExt};
use log::info;
use nalgebra::{Matrix4, Transform3 as NalTransform3, Vector2, Vector3, Vector4};
use pi_async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use pi_ecs::{
    entity::Entity,
    prelude::{world::WorldMut, QueryState, StageBuilder, With, World},
    sys::system::IntoSystem,
};
use render_graph::{graph::RenderGraph, runner::RenderGraphRunner};
use rhi::{device::RenderDevice, options::RenderOptions, setup_render_context, RenderQueue};
use std::borrow::BorrowMut;
use thiserror::Error;
use view::{
    init_view, prepare_view_targets, prepare_view_uniforms,
    render_window::{extract_windows, init_window, prepare_windows, RenderWindows},
    ViewTarget,
};

#[derive(Error, Debug)]
pub enum RenderContextError {
    #[error("Create Device Error.")]
    DeviceError,
}

pub struct RenderArchetype;

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
pub type Mat4 = Matrix4<f32>;
pub type Transform3 = NalTransform3<f32>;

pub struct RenderStage {
    pub extract_stage: StageBuilder,
    pub prepare_stage: StageBuilder,
    pub render_stage: StageBuilder,
}

/// 初始化
pub async fn init_render<P>(
    world: &mut World,
    options: RenderOptions,
    rt: AsyncRuntime<(), P>,
) -> RenderStage
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    world.new_archetype::<RenderArchetype>().create();

    // world 加入 Res: RenderInstance, RenderQueue, RenderDevice, RenderOptions, AdapterInfo
    setup_render_context(world, options).await;

    init_window(world);
    init_view(world);
    init_camera(world);

    world.insert_resource(RenderGraph::new());

    let rg_runner = RenderGraphRunner::new(rt);
    world.insert_resource(rg_runner);

    let mut extract_stage = StageBuilder::new();
    extract_stage.add_node(extract_windows.system(world));

    let mut prepare_stage = StageBuilder::new();
    prepare_stage.add_node(prepare_windows.system(world));
    prepare_stage.add_node(prepare_view_targets.system(world));
    prepare_stage.add_node(prepare_view_uniforms.system(world));
    prepare_stage.add_node(prepare_rg::<P>.system(world));

    let mut render_stage = StageBuilder::new();
    render_stage.add_node(render_system::<P>.system(world));

    return RenderStage {
        extract_stage,
        prepare_stage,
        render_stage,
    };
}

// RenderGraph Build & Prepare
pub fn prepare_rg<P>(world: WorldMut) -> BoxFuture<'static, std::io::Result<()>>
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    let mut world = world.clone();

    async move {
        let mut query = QueryState::<RenderArchetype, &mut RenderGraphRunner<P>>::new(&mut world);
        for mut runner in query.iter_mut(&mut world.clone()) {
            if runner.run_graph.is_some() {
                // TODO: 要加上 渲染图 改变 的 代码
                return Ok(());
            }

            let mut query = QueryState::<
                RenderArchetype,
                (&mut RenderGraph, &RenderDevice, &RenderQueue),
            >::new(&mut world);
            for (mut rg, device, queue) in query.iter_mut(&mut world.clone()) {
                runner
                    .build(
                        world.clone(),
                        device.clone(),
                        queue.clone(),
                        rg.borrow_mut(),
                    )
                    .unwrap();

                runner.prepare().await;
            }
        }

        Ok(())
    }
    .boxed()
}

/// 每帧 调用一次，用于 驱动 渲染图
fn render_system<P>(mut world: WorldMut) -> BoxFuture<'static, std::io::Result<()>>
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    info!("begin render_system");

    async move {
        info!("begin async render_system");

        let mut w = world.clone();
        let mut query = QueryState::<RenderArchetype, &mut RenderGraphRunner<P>>::new(&mut w);

        let mut w = world.clone();
        for mut graph_runner in query.iter_mut(&mut w) {
            let graph_runner = graph_runner.borrow_mut();
            graph_runner.run().await;

            info!("render_system: after graph_runner.run");

            {
                // Remove ViewTarget components to ensure swap chain TextureViews are dropped.
                // If all TextureViews aren't dropped before present, acquiring the next swap chain texture will fail.
                let view_entities = world
                    .query_filtered::<RenderArchetype, Entity, With<ViewTarget>>()
                    .iter(&world)
                    .collect::<Vec<_>>();

                info!(
                    "render_system: before iter view_entities, len = {:?}",
                    view_entities.len()
                );
                for e in view_entities {
                    world.remove_component::<ViewTarget>(e);
                }

                let windows = world.get_resource_mut::<RenderWindows>().unwrap();

                info!(
                    "render_system: before iter windows, len = {:?}",
                    windows.len()
                );
                for window in windows.values_mut() {
                    if let Some(texture_view) = window.swap_chain_texture.take() {
                        if let Some(surface_texture) = texture_view.take_surface_texture() {
                            surface_texture.present();
                            info!("render_system: after surface_texture.present");
                        }
                    }
                }
            }
        }

        Ok(())
    }
    .boxed()
}
