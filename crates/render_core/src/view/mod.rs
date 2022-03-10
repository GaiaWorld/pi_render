pub mod render_window;

use self::render_window::RenderWindows;
use crate::{
    camera::{RenderCamera, RenderCameraNames},
    rhi::{
        device::RenderDevice,
        texture::{Texture, TextureView},
        uniform_vec::DynamicUniformVec,
        RenderQueue,
    },
    Mat4, RenderArchetype, Vec3,
};
use render_crevice::std140::AsStd140;
use pi_ecs::prelude::*;
use wgpu::{Color, Operations, RenderPassColorAttachment};

pub fn init_view(world: &mut World) {
    let views = ViewUniforms::default();
    world.insert_resource(views);
}

pub struct RenderView {
    pub projection: Mat4,
    pub transform: Mat4,
    // TODO 之后会用 Transform3 直接换掉
    pub translation: Vec3,
    pub width: u32,
    pub height: u32,
    pub near: f32,
    pub far: f32,
}

#[derive(Clone, AsStd140)]
pub struct ViewUniform {
    view_proj: Mat4,
    view: Mat4,
    inverse_view: Mat4,
    projection: Mat4,
    world_position: Vec3,
    near: f32,
    far: f32,
    width: f32,
    height: f32,
}

#[derive(Default)]
pub struct ViewUniforms {
    pub uniforms: DynamicUniformVec<ViewUniform>,
}

pub struct ViewUniformOffset {
    pub offset: u32,
}

pub struct ViewTarget {
    pub view: TextureView,
    pub sampled_target: Option<TextureView>,
}

impl ViewTarget {
    pub fn get_color_attachment(&self, ops: Operations<Color>) -> RenderPassColorAttachment {
        RenderPassColorAttachment {
            view: if let Some(sampled_target) = &self.sampled_target {
                sampled_target
            } else {
                &self.view
            },
            resolve_target: if self.sampled_target.is_some() {
                Some(&self.view)
            } else {
                None
            },
            ops,
        }
    }
}

pub struct ViewDepthTexture {
    pub texture: Texture,
    pub view: TextureView,
}

pub fn prepare_view_uniforms(
    mut commands: Commands<ViewUniformOffset>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut view_uniforms: ResMut<ViewUniforms>,
    views: Query<RenderArchetype, (Entity, &RenderView)>,
) {
    view_uniforms.uniforms.clear();
    for (entity, camera) in views.iter() {
        let projection = camera.projection;
        let view = camera.transform;
        let inverse_view = view.try_inverse().unwrap();
        let view_uniforms = ViewUniformOffset {
            offset: view_uniforms.uniforms.push(ViewUniform {
                view_proj: projection * inverse_view,
                view,
                inverse_view,
                projection,
                world_position: camera.translation,
                near: camera.near,
                far: camera.far,
                width: camera.width as f32,
                height: camera.height as f32,
            }),
        };

        commands.insert(entity, view_uniforms);
    }

    view_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);
}

pub fn prepare_view_targets(
    mut commands: Commands<ViewTarget>,
    camera_names: Res<RenderCameraNames>,
    windows: Res<RenderWindows>,
    cameras: Query<RenderArchetype, &RenderCamera>,
) {
    for entity in camera_names.entities.values().copied() {
        let camera = if let Some(camera) = cameras.get(entity) {
            camera
        } else {
            continue;
        };

        if let Some(_size) = camera.size {
            if let Some(texture_view) = camera.target.get_texture_view(&windows) {
                let sampled_target = None;

                commands.insert(
                    entity,
                    ViewTarget {
                        view: texture_view.clone(),
                        sampled_target,
                    },
                );
            }
        }
    }
}
