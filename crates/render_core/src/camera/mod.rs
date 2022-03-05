pub mod active_cameras;
pub mod render_target;

use self::render_target::RenderTarget;
use crate::Vec2;
use pi_ecs::{entity::Entity, prelude::World};
use pi_hash::XHashMap;

#[derive(Default)]
pub struct CameraPlugin;

impl CameraPlugin {
    pub const CAMERA_2D: &'static str = "camera_2d";
    pub const CAMERA_3D: &'static str = "camera_3d";
}

pub fn init_camera(world: &mut World) {
    world.insert_resource::<RenderCameraNames>(RenderCameraNames::default());
}

#[derive(Default)]
pub struct RenderCameraNames {
    pub entities: XHashMap<String, Entity>,
}

#[derive(Debug)]
pub struct RenderCamera {
    pub target: RenderTarget,
    pub name: Option<String>,
    pub size: Option<Vec2>,
}
