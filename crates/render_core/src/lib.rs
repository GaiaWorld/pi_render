//! 基于 ECS 框架的 渲染库
//! 提供 rhi封装 和 渲染图

pub mod camera;
pub mod color;
pub mod render_graph;
pub mod render_nodes;
pub mod rhi;
pub mod texture;
pub mod view;

use camera::init_camera;
use futures::{future::BoxFuture, FutureExt};
use nalgebra::{Matrix4, Transform3 as NalTransform3, Vector2, Vector3, Vector4};
use pi_async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use pi_ecs::{
    entity::Entity,
    prelude::{world::WorldMut, ResMut, StageBuilder, With, World},
    sys::system::IntoSystem,
};
use raw_window_handle::HasRawWindowHandle;
use render_graph::{graph::RenderGraph, runner::RenderGraphRunner};
use rhi::{create_instance, create_render_context, create_surface, options::RenderOptions};
use thiserror::Error;
use view::{init_view, window::RenderWindows, ViewTarget};
use wgpu::{Instance, Surface};

#[derive(Error, Debug)]
pub enum RenderContextError {
    #[error("Create Device Error.")]
    DeviceError,
}

pub struct RenderArchetype;

pub struct RawWindowHandleWrapper<T: HasRawWindowHandle>(pub T);
unsafe impl<T: HasRawWindowHandle> Send for RawWindowHandleWrapper<T> {}
unsafe impl<T: HasRawWindowHandle> Sync for RawWindowHandleWrapper<T> {}

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
pub type Mat4 = Matrix4<f32>;
pub type Transform3 = NalTransform3<f32>;

pub fn create_instance_surface(
    window: &impl HasRawWindowHandle,
    options: &RenderOptions,
) -> (wgpu::Instance, wgpu::Surface) {
    let instance = create_instance(&options);
    let surface = create_surface(&instance, &window);

    (instance, surface)
}

/// 初始化
pub async fn init_render<P>(
    world: &mut World,
    instance: Instance,
    surface: Surface,
    options: RenderOptions,
    rt: AsyncRuntime<(), P>,
) -> StageBuilder
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    world.new_archetype::<RenderArchetype>().create();

    init_view(world);
    init_camera(world);

    let (device, queue, options) = create_render_context(instance, surface, options).await;

    world.insert_resource(RenderGraph::new());
    world.insert_resource(options);
    world.insert_resource(device);
    world.insert_resource(queue);

    let rg_runner = RenderGraphRunner::new(rt);
    world.insert_resource(rg_runner);

    let mut stage = StageBuilder::new();
    let rg = render_system::<P>.system(world);
    stage.add_node(rg);

    return stage;
}

/// 每帧 调用一次，用于 驱动 渲染图
fn render_system<P>(
    mut world: WorldMut,
    mut graph_runner: ResMut<RenderGraphRunner<P>>,
) -> BoxFuture<'static, std::io::Result<()>>
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    async move {
        graph_runner.run().await;

        {
            // Remove ViewTarget components to ensure swap chain TextureViews are dropped.
            // If all TextureViews aren't dropped before present, acquiring the next swap chain texture will fail.
            let view_entities = world
                .query_filtered::<RenderArchetype, Entity, With<ViewTarget>>()
                .iter(&world)
                .collect::<Vec<_>>();

            for e in view_entities {
                world.remove_component::<ViewTarget>(e);
            }

            let windows = world.get_resource_mut::<RenderWindows>().unwrap();
            for window in windows.values_mut() {
                if let Some(texture_view) = window.swap_chain_texture.take() {
                    if let Some(surface_texture) = texture_view.take_surface_texture() {
                        surface_texture.present();
                    }
                }
            }
        }

        Ok(())
    }
    .boxed()
}
