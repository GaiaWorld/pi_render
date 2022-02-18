//! 基于 ECS 框架的 渲染库
//! 提供 rhi封装 和 渲染图

pub mod camera;
pub mod color;
pub mod graph;
pub mod mesh;
pub mod primitives;
pub mod rhi;
pub mod texture;
pub mod view;

use camera::init_camera;
use futures::{future::BoxFuture, FutureExt};
use graph::graph::RenderGraph;
use nalgebra::{Matrix4, Vector2, Vector3, Vector4};
use pi_ecs::prelude::{Entity, ResMut, With, World};
use raw_window_handle::RawWindowHandle;
use rhi::{create_render_context, device::RenderDevice, options::RenderOptions, RenderQueue};
use thiserror::Error;
use view::{init_view, window::RenderWindows, ViewTarget};

pub(crate) struct RenderGraphRunner;

#[derive(Error, Debug)]
pub enum RenderContextError {
    #[error("Create Device Error.")]
    DeviceError,
}

pub struct RenderArchetype;

pub struct RawWindowHandleWrapper(pub RawWindowHandle);
unsafe impl Send for RawWindowHandleWrapper {}
unsafe impl Sync for RawWindowHandleWrapper {}

pub type Vec2 = Vector2<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec4 = Vector4<f32>;
pub type Mat4 = Matrix4<f32>;

/// 初始化
pub async fn init_render(world: &mut World, window: RawWindowHandle, options: RenderOptions) {
    world.new_archetype::<RenderArchetype>().create();

    init_view(world);
    init_camera(world);

    let (device, queue, options) = create_render_context(&window, options).await;

    let window = RawWindowHandleWrapper(window);

    world.insert_resource(window);
    world.insert_resource(options);

    world.insert_resource(device);
    world.insert_resource(queue);
}

/// 每帧 调用一次，用于 驱动 渲染图
pub fn render_system(
    world: &World,
    graph: ResMut<RenderGraph>,
    device: ResMut<RenderDevice>,
    queue: ResMut<RenderQueue>,
) -> BoxFuture<'static, std::io::Result<()>> {
    async move {
        graph.update(world);

        RenderGraphRunner::run(graph, device.clone(), queue).unwrap();

        {
            // Remove ViewTarget components to ensure swap chain TextureViews are dropped.
            // If all TextureViews aren't dropped before present, acquiring the next swap chain texture will fail.
            let view_entities = world
                .query_filtered::<Entity, With<ViewTarget>>()
                .iter(world)
                .collect::<Vec<_>>();
            for view_entity in view_entities {
                world.entity_mut(view_entity).remove::<ViewTarget>();
            }

            let mut windows = world.get_resource_mut::<RenderWindows>().unwrap();
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
