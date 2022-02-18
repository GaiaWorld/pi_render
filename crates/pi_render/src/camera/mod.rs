pub mod active_cameras;
pub mod bundle;
#[allow(clippy::module_inception)]
pub mod camera;
pub mod projection;

use self::{active_cameras::ActiveCameras, camera::Camera};
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

pub fn init_camera(world: &World) {
    world.insert(RenderCameraNames::default());

    world.init_resource::<RenderCameraNames>();
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

fn extract_cameras(
    mut cmd_camera: Commands<RenderCamera>,
    mut cmd_view: Commands<RenderView>,
    mut camera_names: ResMut<RenderCameraNames>,
    active_cameras: Res<ActiveCameras>,
    windows: Res<RenderWindows>,
    query: Query<RenderArchetype, (Entity, &Camera, &Mat4)>,
) {
    let mut entities = XHashMap::default();
    for camera in active_cameras.iter() {
        let name = &camera.name;
        if let Some((entity, camera, transform, visible_entities)) =
            camera.entity.and_then(|e| query.get(e).ok())
        {
            if let Some(window) = windows.get(camera.window) {
                entities.insert(name.clone(), entity);

                cmd_camera.insert(
                    entity,
                    RenderCamera {
                        window_id: camera.window,
                        name: camera.name.clone(),
                    },
                );

                cmd_view.insert(
                    entity,
                    RenderView {
                        projection: camera.projection_matrix,
                        transform: *transform,
                        width: window.physical_width().max(1),
                        height: window.physical_height().max(1),
                        near: camera.near,
                        far: camera.far,
                    },
                );
            }
        }
    }

    for (n, e) in entities {
        camera_names.insert(n, e);
    }
}
