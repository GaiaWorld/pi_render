use pi_ecs::prelude::World;

pub mod camera;
pub mod color;
pub mod view;

#[inline]
pub fn insert_resources(world: &mut World) {
    camera::insert_resources(world);
    color::insert_resources(world);
    view::insert_resources(world);
}
