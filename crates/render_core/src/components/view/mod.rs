use pi_ecs::prelude::World;

pub mod render_window;

#[inline]
pub fn insert_resources(world: &mut World) {
    render_window::insert_resources(world)
}
