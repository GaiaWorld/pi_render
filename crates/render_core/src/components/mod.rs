use pi_ecs::prelude::World;

pub mod view;

#[inline]
pub fn init_ecs(world: &mut World) {
    view::init_ecs(world);
}
