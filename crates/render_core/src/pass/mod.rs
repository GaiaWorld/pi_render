use pi_ecs::prelude::World;

pub mod clear_pass;

#[inline]
pub fn insert_resources(world: &mut World) {
    clear_pass::insert_resources(world)
}
