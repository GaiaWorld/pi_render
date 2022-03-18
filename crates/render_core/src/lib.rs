//! 基于 ECS 框架的 渲染库
//!
//! ## 主要结构
//!
//! + rhi 封装 wgpu
//! + render_graph 基于 rhi 封装 渲染图
//! + render_nodes 具体的常用的 渲染节点
//! + components 渲染组件，比如 color, camera ...

pub mod components;
pub mod render_graph;
pub mod render_nodes;
pub mod rhi;

use crate::components::{
    camera::render_target::TextureViews,
    view::render_window::{prepare_windows, RenderWindows},
};
use futures::{future::BoxFuture, FutureExt};
use log::info;
use nalgebra::{Matrix4, Transform3 as NalTransform3, Vector2, Vector3, Vector4};
use pi_async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use pi_ecs::{
    prelude::{world::WorldMut, StageBuilder, World},
    sys::system::IntoSystem,
};
use pi_share::ShareRefCell;
use render_graph::{graph::RenderGraph, runner::RenderGraphRunner};
use rhi::{device::RenderDevice, options::RenderOptions, setup_render_context, RenderQueue};
use std::borrow::BorrowMut;
use thiserror::Error;
use winit::window::Window;

#[derive(Error, Debug)]
pub enum RenderContextError {
    #[error("Create Device Error.")]
    DeviceError,
}

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
    // register_components(world.new_archetype::<RenderArchetype>()).create();

    // 初始化渲染，加入如下 Res: RenderInstance, RenderQueue, RenderDevice, RenderOptions, AdapterInfo
    setup_render_context(world.clone(), options, window).await;
    // 添加 渲染图 Res
    insert_render_graph(world, rt);
    // 添加 其他 Res
    insert_resources(world);

    // 注册 必要的 Stage 和 System
    register_system::<P>(world)
}

// RenderGraph Build & Prepare
pub fn prepare_rg<P>(world: WorldMut) -> BoxFuture<'static, std::io::Result<()>>
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    let world = world.clone();

    async move {
        let runner = world.get_resource_mut::<RenderGraphRunner<P>>().unwrap();

        if runner.run_graph.is_some() {
            // TODO: 要加上 渲染图 改变 的 代码
            todo!();
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

    async move {
        info!("begin async render_system");

        let graph_runner = world.get_resource_mut::<RenderGraphRunner<P>>().unwrap();
        graph_runner.run().await;

        info!("render_system: after graph_runner.run");

        let world = world.clone();

        let views = world.get_resource_mut::<TextureViews>().unwrap();
        let windows = world.get_resource::<RenderWindows>().unwrap();
        for (_, window) in windows.iter() {
            let view = views.get_mut(window.view).unwrap();
            if let Some(view) = view.take_surface_texture() {
                view.present();
                info!("render_system: after surface_texture.present");
            }
        }
        Ok(())
    }
    .boxed()
}

fn insert_render_graph<P>(world: &mut World, rt: AsyncRuntime<(), P>)
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    world.insert_resource(RenderGraph::default());

    let rg_runner = RenderGraphRunner::new(rt);
    world.insert_resource(rg_runner);
}

// 添加 其他 Res
fn insert_resources(world: &mut World) {
    components::view::insert_resources(world);
    components::camera::insert_resources(world);
    render_nodes::insert_resources(world);
}

fn register_system<P>(world: &mut World) -> RenderStage
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    let extract_stage = StageBuilder::new();

    let mut prepare_stage = StageBuilder::new();
    prepare_stage.add_node(prepare_windows.system(world));
    prepare_stage.add_node(prepare_rg::<P>.system(world));

    let mut render_stage = StageBuilder::new();
    render_stage.add_node(render_system::<P>.system(world));

    RenderStage {
        extract_stage,
        prepare_stage,
        render_stage,
    }
}
