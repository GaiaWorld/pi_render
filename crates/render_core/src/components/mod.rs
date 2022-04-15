use pi_ecs::prelude::World;

pub mod color;
pub mod mesh;
pub mod view;

#[inline]
pub fn init_ecs(world: &mut World) {
    color::init_ecs(world);
    view::init_ecs(world);
    mesh::init_ecs(world);
}
