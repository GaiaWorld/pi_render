use pi_ecs::prelude::World;

pub mod target;
pub mod render_window;
pub mod target_alloc;

#[inline]
pub fn init_ecs(world: &mut World) {
    render_window::insert_resources(world);
	target::insert_resources(world);
}
