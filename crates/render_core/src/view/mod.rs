use pi_ecs::{prelude::World, world::ArchetypeInfo};

pub mod render_window;
pub mod render_target;

#[inline]
pub fn register_components(archetype: ArchetypeInfo) -> ArchetypeInfo {
    render_window::register_components(archetype)
}

#[inline]
pub fn insert_resources(world: &mut World) {
    render_window::insert_resources(world)
}
