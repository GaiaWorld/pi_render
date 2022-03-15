use pi_ecs::{world::ArchetypeInfo, prelude::World};

pub mod clear_pass;

#[inline]
pub fn register_components(archetype: ArchetypeInfo) -> ArchetypeInfo {
    clear_pass::register_components(archetype)
}

#[inline]
pub fn insert_resources(world: &mut World) {
    clear_pass::insert_resources(world)
}
