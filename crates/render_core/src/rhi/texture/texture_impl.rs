use pi_share::Share;
use std::ops::Deref;
use wgpu::{SurfaceConfiguration, SurfaceTexture};

use crate::rhi::device::RenderDevice;

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
    pub fn texture_format(&self) -> Option<wgpu::TextureFormat> {
        self.texture.as_ref().map(|t| {
            t.texture.format()
        })
    }

    #[inline]
    pub fn surface(&self) -> Option<&wgpu::Surface> {
        match self {
            Self::Texture(_) => None,
            Self::Surface { surface, .. } => Some(surface.deref()),
        }
    }

    #[inline]
    pub fn next_frame(&mut self, device: &RenderDevice, config: &SurfaceConfiguration) {
        if let Self::Surface {
            surface,
            texture,
            view,
            ..
        } = self
        {
            let t = match surface.get_current_texture() {
                Ok(swap_chain_frame) => swap_chain_frame,
                Err(wgpu::SurfaceError::Outdated) => {
                    device.configure_surface(surface, config);
                    surface
                        .get_current_texture()
                        .expect("Error reconfiguring surface")
                }
                err => err.expect("Failed to acquire next swap chain texture!"),
            };

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

pub struct ScreenTexture {
	surface: Share<wgpu::Surface>,
	texture: Option<Share<wgpu::SurfaceTexture>>,
	pub view: Option<Share<wgpu::TextureView>>,
}

impl ScreenTexture {
	#[inline]
    pub fn with_surface(surface: Share<wgpu::Surface>) -> Self {
        Self {
            surface,
            texture: None,
            view: None,
        }
    }

	#[inline]
    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

	#[inline]
    pub fn texture(&self) -> &Option<Share<wgpu::SurfaceTexture>> {
        &self.texture
    }

	#[inline]
    pub fn next_frame(&mut self, device: &RenderDevice, config: &SurfaceConfiguration) {
        assert_eq!(self.texture.is_some(), self.view.is_some());
        
        if self.texture.is_none() {
            let t = match self.surface.get_current_texture() {
                Ok(swap_chain_frame) => swap_chain_frame,
                Err(wgpu::SurfaceError::Outdated) => {
                    device.configure_surface(self.surface.as_ref(), config);
                    self.surface
                        .get_current_texture()
                        .expect("Error reconfiguring surface")
                }
                err => err.expect("Failed to acquire next swap chain texture!"),
            };

            let t = Share::new(t);
            let v = Share::new(t.texture.create_view(&Default::default()));

            self.texture = Some(t);
            self.view = Some(v);
        }
    }

	#[inline]
    pub fn take_surface_texture(&mut self) -> Option<SurfaceTexture> {
        let v = self.view.take().unwrap();
		Share::try_unwrap(v).ok();

		let t = self.texture.take().unwrap();
		Share::try_unwrap(t).ok()
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
        } else  {
            wgpu::TextureFormat::Bgra8Unorm
        }
    }
}
