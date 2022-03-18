pub mod render_target;

use pi_ecs::prelude::World;

pub struct Viewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,

    pub min_depth: f32,
    pub max_depth: f32,
}

impl Viewport {
    pub fn new_with_detph(
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        min_depth: f32,
        max_depth: f32,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            min_depth,
            max_depth,
        }
    }

    pub fn new_with_rect(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            min_depth: 0.0,
            max_depth: 1.0,
        }
    }
}

pub struct Scissor {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Scissor {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug, Default)]
pub struct ClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ClearColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<&ClearColor> for wgpu::Color {
    fn from(clear: &ClearColor) -> Self {
        Self {
            r: clear.r as f64,
            g: clear.g as f64,
            b: clear.b as f64,
            a: clear.a as f64,
        }
    }
}


#[inline]
pub fn insert_resources(_world: &mut World) {}
