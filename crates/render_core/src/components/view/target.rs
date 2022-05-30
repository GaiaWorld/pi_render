use crate::{
    rhi::texture::{TextureView},
};
use pi_ecs::prelude::World;
use pi_slotmap::{new_key_type, SlotMap};

new_key_type! {
    pub struct TextureViewKey;
    pub struct RenderTargetKey;
}

pub type TextureViews = SlotMap<TextureViewKey, Option<TextureView>>;

pub type RenderTargets = SlotMap<RenderTargetKey, RenderTarget>;

#[inline]
pub fn insert_resources(world: &mut World) {
    world.insert_resource(TextureViews::default());
    world.insert_resource(RenderTargets::default());
}

#[derive(Clone, Default)]
pub struct RenderTarget {
    pub depth: TextureViewKey,
    pub colors: Vec<TextureViewKey>, // TODO smallvec
}

impl RenderTarget {
    pub fn add_color(&mut self, view: TextureViewKey) {
        self.colors.push(view);
    }

    pub fn remove_color(&mut self, view: TextureViewKey) {
        self.colors.retain(|v| view != *v);
    }

    pub fn set_depth(&mut self, depth: TextureViewKey) {
        self.depth = depth;
    }
}
