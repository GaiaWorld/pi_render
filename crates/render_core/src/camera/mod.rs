use pi_ecs::{prelude::World, world::ArchetypeInfo};

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

pub struct ClearOption {
    pub color: Option<ClearColor>,
    pub depth: Option<f32>,
    pub stencil: Option<u32>,
}

impl ClearOption {
    pub fn new() -> Self {
        Self {
            color: None,
            depth: None,
            stencil: None,
        }
    }

    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color = Some(ClearColor::new(r, g, b, a));
    }

    pub fn set_detph(&mut self, d: f32) {
        self.depth = Some(d);
    }

    pub fn set_stencil(&mut self, stencil: u32) {
        self.stencil = Some(stencil);
    }
}

#[inline]
pub fn register_components(archetype: ArchetypeInfo) -> ArchetypeInfo {
    archetype
        .register::<Viewport>()
        .register::<Scissor>()
        .register::<ClearOption>()
}

#[inline]
pub fn insert_resources(_world: &mut World) {}
