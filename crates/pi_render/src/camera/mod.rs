pub mod active_cameras;

use self::active_cameras::ActiveCameras;
use crate::{
    view::{
        window::{RenderWindows, WindowId},
        RenderView,
    },
    Mat4, RenderArchetype,
};
use hash::XHashMap;
use pi_ecs::{
    entity::Entity,
    prelude::{Commands, Query, Res, ResMut, World},
};

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
    pub window_id: WindowId,
    pub name: Option<String>,
}

