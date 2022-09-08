//! 基于 ECS 框架的 渲染库
//!
//! ## 主要结构
//!
//! + rhi 封装 wgpu
//! + render_graph 基于 rhi 封装 渲染图
//! + render_nodes 具体的常用的 渲染节点
//! + components 渲染组件，比如 color, camera ...
#![feature(specialization)]

#[macro_use]
extern crate lazy_static;

pub mod components;
pub mod font;
pub mod graph;
pub mod rhi;

mod math;
pub use math::*;

use crate::components::view::render_window::{prepare_windows, RenderWindows};
use graph::{graph::RenderGraph, GraphError};
use log::trace;
use pi_async::rt::AsyncRuntime;
use pi_ecs::{
    prelude::{world::WorldMut, StageBuilder, World},
    sys::system::IntoSystem,
};
use pi_share::Share;
use rhi::{
    device::RenderDevice, options::RenderOptions, setup_render_context, texture::ScreenTexture,
    RenderQueue,
};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
use thiserror::Error;
use winit::window::Window;

#[derive(Error, Debug)]

pub enum RenderContextError {
    #[error("Create Device Error.")]
    DeviceError,
}

pub struct RenderStage {
    pub prepare_stage: StageBuilder,
    pub render_stage: StageBuilder,
}

/// 渲染用的 异步运行时
struct RenderAsyncRuntime<A: 'static + AsyncRuntime + Send> {
    rt: A,
}

/// 初始化
pub async fn init_render<A>(
    world: &mut World,
    options: RenderOptions,
    window: Share<Window>,
    rt: A,
) -> RenderStage
where
    A: 'static + AsyncRuntime + Send,
{
    // 一次性 注册 所有的 组件
    // register_components(world.new_archetype::<RenderArchetype>()).create();

    // 初始化渲染，加入如下 Res: RenderInstance, RenderQueue, RenderDevice, RenderOptions, AdapterInfo
    setup_render_context(world.clone(), options, window).await;
    // 添加 渲染图 Res
    insert_render_graph::<A>(world, rt);
    // 添加 其他 Res
    insert_resources(world);

    // 注册 必要的 Stage 和 System
    register_system::<A>(world)
}

// Build RenderGraph
// 注：System 的返回值 一定要 std::io::Result 才是 异步类型
async fn build_graph<A>(world: WorldMut) -> std::io::Result<()> 
where
    A: 'static + AsyncRuntime + Send,
{
    let rg = world.get_resource_mut::<RenderGraph>().unwrap();
    let device = world.get_resource::<RenderDevice>().unwrap();
    let queue = world.get_resource::<RenderQueue>().unwrap();

    let rt = world.get_resource::<RenderAsyncRuntime<A>>().unwrap();

    rg.build(&rt.rt, device.clone(), queue.clone(), world.clone())
        .await
        .unwrap();

    Ok(())
}

// 每帧 调用一次，用于 驱动 渲染图
// 注：System 的返回值 一定要 std::io::Result 才是 异步类型
async fn render_system<A>(world: WorldMut) -> std::io::Result<()> 
where
    A: 'static + AsyncRuntime + Send,
{
    let graph = world.get_resource_mut::<RenderGraph>().unwrap();

    let rt = world.get_resource::<RenderAsyncRuntime<A>>().unwrap();
    graph.run(&rt.rt).await.unwrap();

    let world = world.clone();

    let screen_texture = world.get_resource_mut::<ScreenTexture>().unwrap();
    let windows = world.get_resource::<RenderWindows>().unwrap();

    // 呈现 所有的 窗口 -- 交换链
    for (_, _window) in windows.iter() {
        if let Some(view) = screen_texture.take_surface_texture() {
            view.present();
            trace!("render_system: after surface_texture.present");
        }
    }

    Ok(())
}

fn insert_render_graph<A>(world: &mut World, rt: A)
where
    A: 'static + AsyncRuntime + Send,
{
    let r = RenderAsyncRuntime { rt };
    world.insert_resource(r);
    world.insert_resource(RenderGraph::default());
}

// 添加 其他 Res
fn insert_resources(world: &mut World) {
    components::init_ecs(world);
}

fn register_system<A>(world: &mut World) -> RenderStage
where
    A: 'static + AsyncRuntime + Send,
{
    let mut prepare_stage = StageBuilder::new();
    prepare_stage.add_node(build_graph::<A>.system(world));
    prepare_stage.add_node(prepare_windows.system(world));

    let mut render_stage = StageBuilder::new();
    render_stage.add_node(render_system::<A>.system(world));

    RenderStage {
        prepare_stage,
        render_stage,
    }
}

use parking_lot::RwLock;

lazy_static! {
    static ref IS_FIRST: AtomicBool = AtomicBool::new(true);
    static ref FAQ_HANDLE: AtomicU64 = AtomicU64::new(0);
    static ref FAQ_MAP: RwLock<HashMap<u64, Box<dyn Fn() + Send + Sync + 'static>>> =
        RwLock::new(HashMap::new());
}

pub fn request_animation_frame<F: Fn() + Send + Sync + 'static>(cb: F) -> u64 {
    let handle = FAQ_HANDLE.fetch_add(1, Ordering::Relaxed);
    let _ = FAQ_MAP.write().insert(handle, Box::new(cb));

    if IS_FIRST.load(Ordering::Relaxed) {
        IS_FIRST.store(false, Ordering::Relaxed);
        render();
    }

    handle
}

pub fn cancel_animation_frame(handle: u64) {
    let _ = FAQ_MAP.write().remove(&handle);
}

fn render() {
    // let frame_time = 17;
    std::thread::spawn(move || loop {
        // next_frame();
        // let begin = std::time::Instant::now();
        for cb in FAQ_MAP.read().values() {
            cb();
        }
        // let end = begin.elapsed().as_millis() as u64;
        // TODO: 必须强制休眠一点时间，必然会崩溃
        std::thread::sleep(std::time::Duration::from_millis(1));
        // if frame_time > end {
        //     std::thread::sleep(std::time::Duration::from_millis(20));
        // }
        // prepare();
    });
}
