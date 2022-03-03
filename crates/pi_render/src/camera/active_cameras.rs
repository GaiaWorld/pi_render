use hash::XHashMap;
use pi_ecs::prelude::*;

#[derive(Debug, Default)]
pub struct ActiveCamera {
    pub name: String,
    pub entity: Option<Entity>,
}

#[derive(Debug, Default)]
pub struct ActiveCameras {
    cameras: XHashMap<String, ActiveCamera>,
}
