use crate::{
    rhi::texture::TextureView,
    view::window::{RenderWindows, WindowId},
    Vec2,
};

/// 渲染目标的 封装
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RenderTarget {
    /// Window to which the camera's view is rendered.
    Window(WindowId),
}

impl Default for RenderTarget {
    fn default() -> Self {
        Self::Window(Default::default())
    }
}

impl RenderTarget {
    pub fn get_texture_view<'a>(&self, windows: &'a RenderWindows) -> Option<&'a TextureView> {
        match self {
            RenderTarget::Window(window_id) => windows
                .get(window_id)
                .and_then(|window| window.swap_chain_texture.as_ref()),
        }
    }

    pub fn get_size(&self, windows: &RenderWindows) -> Option<Vec2> {
        match self {
            RenderTarget::Window(window_id) => windows
                .get(window_id)
                .map(|window| Vec2::new(window.width as f32, window.height as f32)),
        }
    }
}
