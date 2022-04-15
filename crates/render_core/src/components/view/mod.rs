use pi_ecs::prelude::World;

pub mod target;
pub mod render_window;

#[inline]
pub fn init_ecs(world: &mut World) {
    render_window::insert_resources(world)
}
