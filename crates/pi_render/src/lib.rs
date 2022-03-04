//! 基于 ECS 框架的 渲染库
//! 提供 rhi封装 和 渲染图

pub mod camera;
pub mod color;
pub mod render_graph;
pub mod rhi;
pub mod texture;
pub mod view;

use futures::{future::BoxFuture, FutureExt};
use nalgebra::{Matrix4, Vector2, Vector3, Vector4, Transform3 as NalTransform3};
use pi_ecs::{
    entity::Entity,
    prelude::{ResMut, With, World},
};
use r#async::rt::{AsyncTaskPool, AsyncTaskPoolExt};
use raw_window_handle::HasRawWindowHandle;
use render_graph::{graph::RenderGraph, runner::RenderGraphRunner};
use rhi::{create_render_context, device::RenderDevice, options::RenderOptions, RenderQueue};
use thiserror::Error;
use view::{window::RenderWindows, ViewTarget};
// use view::{init_view, window::RenderWindows, ViewTarget};

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

/// 初始化
pub async fn init_render<T: HasRawWindowHandle>(
    world: &mut World,
    window: T,
    options: RenderOptions,
) {
    world.new_archetype::<RenderArchetype>().create();

    // init_view(world);
    // init_camera(world);

    let (device, queue, options) = create_render_context(&window, options).await;

    let window = RawWindowHandleWrapper(window);

    world.insert_resource(options);

    world.insert_resource(device);
    world.insert_resource(queue);
}

/// 每帧 调用一次，用于 驱动 渲染图
pub fn render_system<P>(
    mut world: World,
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
