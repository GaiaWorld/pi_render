use crate::{color::Color, camera::render_target::RenderTarget};
use pi_hash::XHashMap;

#[derive(Clone, Debug)]
pub struct ClearColor(pub Color);

impl Default for ClearColor {
    fn default() -> Self {
        Self(Color::rgb(0.4, 0.4, 0.4))
    }
}

#[derive(Clone, Debug, Default)]
pub struct RenderTargetClearColors {
    pub colors: XHashMap<RenderTarget, Color>,
}

impl RenderTargetClearColors {
    pub fn get(&self, target: &RenderTarget) -> Option<&Color> {
        self.colors.get(target)
    }

    pub fn insert(&mut self, target: RenderTarget, color: Color) {
        self.colors.insert(target, color);
    }
}
