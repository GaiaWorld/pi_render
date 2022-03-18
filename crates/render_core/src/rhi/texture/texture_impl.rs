use pi_share::Share;
use std::ops::Deref;
use wgpu::SurfaceTexture;

/// 提供 可 Clone的 Texture
#[derive(Clone, Debug)]
pub struct Texture(Share<wgpu::Texture>);

impl Texture {
    pub fn create_view(&self, desc: &wgpu::TextureViewDescriptor) -> TextureView {
        TextureView::from(self.0.create_view(desc))
    }
}

impl From<wgpu::Texture> for Texture {
    fn from(value: wgpu::Texture) -> Self {
        Texture(Share::new(value))
    }
}

impl Deref for Texture {
    type Target = wgpu::Texture;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 提供 可 Clone的 Sampler
#[derive(Clone, Debug)]
pub struct Sampler(Share<wgpu::Sampler>);

impl From<wgpu::Sampler> for Sampler {
    fn from(value: wgpu::Sampler) -> Self {
        Sampler(Share::new(value))
    }
}

impl Deref for Sampler {
    type Target = wgpu::Sampler;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 统一 TextureView 和 SurfaceTexture，提供统一的 视图
#[derive(Clone, Debug)]
pub enum TextureView {
    /// 普通纹理
    Texture(Share<wgpu::TextureView>),

    /// 表面缓冲区 对应的 纹理
    Surface {
        surface: Share<wgpu::Surface>,
        texture: Option<Share<wgpu::SurfaceTexture>>,
        view: Option<Share<wgpu::TextureView>>,
    },
}

impl TextureView {
    #[inline]
    pub fn with_texture(view: Share<wgpu::TextureView>) -> Self {
        Self::Texture(view)
    }

    #[inline]
    pub fn with_surface(surface: Share<wgpu::Surface>) -> Self {
        Self::Surface {
            surface,
            texture: None,
            view: None,
        }
    }

    #[inline]
    pub fn surface(&self) -> Option<&wgpu::Surface> {
        match self {
            Self::Texture(_) => None,
            Self::Surface { surface, .. } => Some(surface.deref()),
        }
    }

    #[inline]
    pub fn next_frame(&mut self) {
        if let Self::Surface {
            surface,
            texture,
            view,
            ..
        } = self
        {
            let t = surface
                .get_current_texture()
                .expect("Error reconfiguring surface");

            let t = Share::new(t);
            let v = Share::new(t.texture.create_view(&Default::default()));

            *texture = Some(t);
            *view = Some(v);
        }
    }

    #[inline]
    pub fn take_surface_texture(&mut self) -> Option<SurfaceTexture> {
        match self {
            Self::Surface { texture, view, .. } => {
                let v = view.take().unwrap();
                Share::try_unwrap(v).ok();

                let t = texture.take().unwrap();
                Share::try_unwrap(t).ok()
            }
            _ => None,
        }
    }
}

impl From<wgpu::TextureView> for TextureView {
    fn from(value: wgpu::TextureView) -> Self {
        TextureView::Texture(Share::new(value))
    }
}

impl Deref for TextureView {
    type Target = wgpu::TextureView;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match &self {
            TextureView::Texture(view) => view,
            TextureView::Surface { view, .. } => view.as_ref().unwrap(),
        }
    }
}

pub trait PiRenderDefault {
    fn pi_render_default() -> Self;
}

impl PiRenderDefault for wgpu::TextureFormat {
    fn pi_render_default() -> Self {
        if cfg!(target_os = "android") || cfg!(target_arch = "wasm32") {
            // Bgra8UnormSrgb texture missing on some Android devices
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Bgra8UnormSrgb
        }
    }
}
