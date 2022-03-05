use std::{ops::Deref, sync::Arc};

/// 提供 可 Clone的 Texture
#[derive(Clone, Debug)]
pub struct Texture(Arc<wgpu::Texture>);

impl Texture {
    pub fn create_view(&self, desc: &wgpu::TextureViewDescriptor) -> TextureView {
        TextureView::from(self.0.create_view(desc))
    }
}

impl From<wgpu::Texture> for Texture {
    fn from(value: wgpu::Texture) -> Self {
        Texture(Arc::new(value))
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
pub struct Sampler(Arc<wgpu::Sampler>);

impl From<wgpu::Sampler> for Sampler {
    fn from(value: wgpu::Sampler) -> Self {
        Sampler(Arc::new(value))
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
    TextureView(Arc<wgpu::TextureView>),

    /// 表面缓冲区 对应的 纹理
    SurfaceTexture {
        view: Arc<wgpu::TextureView>,
        texture: Arc<wgpu::SurfaceTexture>,
    },
}

impl TextureView {
    /// 取 表面纹理，对普通纹理返回 None
    #[inline]
    pub fn take_surface_texture(self) -> Option<wgpu::SurfaceTexture> {
        match self {
            TextureView::TextureView(_) => None,
            TextureView::SurfaceTexture { texture, .. } => Arc::try_unwrap(texture).ok(),
        }
    }
}

impl From<wgpu::TextureView> for TextureView {
    fn from(value: wgpu::TextureView) -> Self {
        TextureView::TextureView(Arc::new(value))
    }
}

impl From<wgpu::SurfaceTexture> for TextureView {
    fn from(value: wgpu::SurfaceTexture) -> Self {
        let texture = Arc::new(value);
        let view = Arc::new(texture.texture.create_view(&Default::default()));

        TextureView::SurfaceTexture { texture, view }
    }
}

impl Deref for TextureView {
    type Target = wgpu::TextureView;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match &self {
            TextureView::TextureView(view) => view,
            TextureView::SurfaceTexture { view, .. } => view,
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