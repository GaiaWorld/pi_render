
mod scene;
mod model;
// mod effect_value;
mod texture_and_sampler;
// mod draw_value;

pub use scene::*;
pub use model::*;
// pub use draw_value::*;
// pub use effect_value::*;
pub use texture_and_sampler::*;

pub trait TShaderSetLayout {
    fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry>;
}

