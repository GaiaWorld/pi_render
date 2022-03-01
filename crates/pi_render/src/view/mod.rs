pub mod window;

use self::window::RenderWindows;
use crate::{
    Mat4,
    camera::{RenderCamera, RenderCameraNames},
    rhi::{
        device::RenderDevice,
        texture::{Texture, TextureView},
        uniform_vec::DynamicUniformVec,
        RenderQueue,
    },
    texture::texture_cache::TextureCache,
    RenderArchetype, Vec3,
};
use nalgebra::Matrix1;
use pi_crevice::std140::AsStd140;
use pi_ecs::prelude::*;
use wgpu::{Color, Operations, RenderPassColorAttachment};

pub fn init_view(world: &mut World) {
    let views = ViewUniforms::default();
    world.insert_resource(views);
}

pub struct RenderView {
    pub projection: Mat4,
    pub transform: Mat4,
    pub width: u32,
    pub height: u32,
    pub near: f32,
    pub far: f32,
}

// TODO 
#[derive(Clone, AsStd140)]
pub struct PiMat4 {
    // imp: Mat4
}

// TODO 
#[derive(Clone, AsStd140)]
pub struct PiVec3 {
    // imp: Vec3
}

#[derive(Clone, AsStd140)]
pub struct ViewUniform {
    view_proj: PiMat4,
    view: PiMat4,
    inverse_view: PiMat4,
    projection: PiMat4,
    world_position: PiVec3,
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
    // for (entity, camera) in views.iter() {
    //     let projection = camera.projection;
    //     let view = camera.transform.compute_matrix();
    //     let inverse_view = view.inverse();
    //     let view_uniforms = ViewUniformOffset {
    //         offset: view_uniforms.uniforms.push(ViewUniform {
    //             view_proj: projection * inverse_view,
    //             view,
    //             inverse_view,
    //             projection,
    //             world_position: camera.transform.translation,
    //             near: camera.near,
    //             far: camera.far,
    //             width: camera.width as f32,
    //             height: camera.height as f32,
    //         }),
    //     };

    //     commands.insert(entity, view_uniforms);
    // }

    view_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);
}

pub fn prepare_view_targets(
    mut commands: Commands<ViewTarget>,
    camera_names: Res<RenderCameraNames>,
    windows: Res<RenderWindows>,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    cameras: Query<RenderArchetype, &RenderCamera>,
) {
//     for entity in camera_names.entities.values().copied() {
//         let camera = if let Ok(camera) = cameras.get(entity) {
//             camera
//         } else {
//             continue;
//         };

//         let window = if let Some(window) = windows.get(&camera.window_id) {
//             window
//         } else {
//             continue;
//         };

//         let swap_chain_texture = if let Some(texture) = &window.swap_chain_texture {
//             texture
//         } else {
//             continue;
//         };

//         let sampled_target = None;

//         commands.insert(
//             entity,
//             ViewTarget {
//                 view: swap_chain_texture.clone(),
//                 sampled_target,
//             },
//         );
//     }
}
