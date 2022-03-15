//! 基于 ECS 框架的 渲染库
//! 提供 rhi封装 和 渲染图

pub mod camera;
pub mod color;
pub mod render_graph;
pub mod render_nodes;
pub mod rhi;
pub mod texture;
pub mod view;

use futures::{future::BoxFuture, FutureExt};
use log::info;
use nalgebra::{Matrix4, Transform3 as NalTransform3, Vector2, Vector3, Vector4};
use pi_async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use pi_ecs::{
    prelude::{world::WorldMut, QueryState, StageBuilder, World},
    sys::system::IntoSystem,
};
use pi_share::ShareRefCell;
use render_graph::{graph::RenderGraph, runner::RenderGraphRunner};
use rhi::{device::RenderDevice, options::RenderOptions, setup_render_context, RenderQueue};
use std::borrow::BorrowMut;
use thiserror::Error;
use view::render_window::prepare_windows;
use winit::window::Window;

use crate::view::render_window::RenderWindow;

// 组件：激活的
pub struct Active;

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
    window: ShareRefCell<Window>,
    rt: AsyncRuntime<(), P>,
) -> RenderStage
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    // 一次性 注册 所有的 组件
    let archetype = world.new_archetype::<RenderArchetype>();
    let archetype = view::register_components(archetype);
    let archetype = camera::register_components(archetype);
    let archetype = render_nodes::register_components(archetype);
    archetype.register::<Active>().create();

    // world 加入 Res: RenderInstance, RenderQueue, RenderDevice, RenderOptions, AdapterInfo
    setup_render_context(world.clone(), options, window).await;

    view::insert_resources(world);
    camera::insert_resources(world);
    render_nodes::insert_resources(world);

    world.insert_resource(RenderGraph::new());

    let rg_runner = RenderGraphRunner::new(rt);
    world.insert_resource(rg_runner);

    let extract_stage = StageBuilder::new();

    let mut prepare_stage = StageBuilder::new();
    prepare_stage.add_node(prepare_windows.system(world));
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

            let rg = world.get_resource_mut::<RenderGraph>().unwrap();
            let device = world.get_resource::<RenderDevice>().unwrap();
            let queue = world.get_resource::<RenderQueue>().unwrap();
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

        Ok(())
    }
    .boxed()
}

/// 每帧 调用一次，用于 驱动 渲染图
fn render_system<P>(world: WorldMut) -> BoxFuture<'static, std::io::Result<()>>
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    info!("begin render_system");

    let mut query: ShareRefCell<QueryState<RenderArchetype, &'static mut RenderWindow>> =
        ShareRefCell::new(QueryState::new(&mut world.clone()));

    async move {
        info!("begin async render_system");

        let graph_runner = world.get_resource_mut::<RenderGraphRunner<P>>().unwrap();
        graph_runner.run().await;

        info!("render_system: after graph_runner.run");

        let mut world = world.clone();
        for mut window in query.iter_mut(&mut world) {
            if let Some(texture_view) = window.rt.take() {
                if let Some(surface_texture) = texture_view.take_surface_texture() {
                    surface_texture.present();
                    info!("render_system: after surface_texture.present");
                }
            }
        }
        Ok(())
    }
    .boxed()
}
