use crate::rhi::{
    device::RenderDevice,
    texture::{PiRenderDefault, ScreenTexture},
    PresentMode, RenderInstance,
};
use pi_share::Share;
use pi_slotmap::{new_key_type, SlotMap};
use std::{ops::Deref, sync::Arc};
use wgpu::TextureFormat;
use winit::{dpi::PhysicalSize, window::Window};

new_key_type! {
    pub struct RenderWindowKey;
}

pub type RenderWindows = SlotMap<RenderWindowKey, RenderWindow>;

pub struct RenderWindow {
    present_mode: PresentMode,
    last_size: PhysicalSize<u32>,
    handle: Arc<Window>,
}

impl RenderWindow {
    pub fn new(handle: Arc<Window>, present_mode: PresentMode) -> Self {
        Self {
            handle,
            present_mode,
            last_size: PhysicalSize::default(),
        }
    }
}

pub fn prepare_windows<'w>(
    device: &RenderDevice,
    instance: &RenderInstance,
    mut windows: &mut RenderWindows,
    mut view: Option<ScreenTexture>,
) -> std::io::Result<()> {
    for (_, window) in windows.iter_mut() {
        let is_first = view.is_none();
        if is_first {
            let surface = unsafe { instance.create_surface(window.handle.deref()) };
            let surface = Share::new(surface);
            view = Some(ScreenTexture::with_surface(surface));
        }

        let view = view.as_mut().unwrap();

        let PhysicalSize { width, height } = window.handle.inner_size();
        let config = wgpu::SurfaceConfiguration {
            format: TextureFormat::pi_render_default(),
            width,
            height,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: window.present_mode.clone(),
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        let is_size_changed = width != window.last_size.width || height != window.last_size.height;
        if is_size_changed {
            window.last_size.width = width;
            window.last_size.height = height;
        }
        // 记得 第一次 需要 Config
        if is_first || is_size_changed {
            device.configure_surface(view.surface(), &config);
        }

        // log::warn!("next_frame========================");
        // 每帧 都要 设置 新的 SuraceTexture
        let _ = view.next_frame(&device, &config);
    }
    Ok(())
}
