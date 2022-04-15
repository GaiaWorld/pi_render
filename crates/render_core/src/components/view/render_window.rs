use crate::{
    rhi::{
        device::RenderDevice,
        texture::{PiRenderDefault, TextureView},
        PresentMode, RenderInstance,
    },
};
use pi_ecs::prelude::{Res, ResMut, World};
use pi_share::ShareRefCell;
use pi_slotmap::{new_key_type, SlotMap};
use std::{ops::Deref, sync::Arc};
use wgpu::TextureFormat;
use winit::{dpi::PhysicalSize, window::Window};

use super::target::{TextureViewKey, TextureViews};

new_key_type! {
    pub struct RenderWindowKey;
}

pub type RenderWindows = SlotMap<RenderWindowKey, RenderWindow>;

pub struct RenderWindow {
    present_mode: PresentMode,
    last_size: PhysicalSize<u32>,
    handle: ShareRefCell<Window>,
    view: TextureViewKey,
}

impl RenderWindow {
    pub fn new(
        handle: ShareRefCell<Window>,
        present_mode: PresentMode,
        view: TextureViewKey,
    ) -> Self {
        Self {
            handle,
            present_mode,
            last_size: PhysicalSize::default(),
            view,
        }
    }

    pub fn get_view(&self) -> TextureViewKey {
        self.view
    }
}

#[inline]
pub fn insert_resources(world: &mut World) {
    world.insert_resource(RenderWindows::default());
}

pub async fn prepare_windows<'w>(
    device: Res<'w, RenderDevice>,
    instance: Res<'w, RenderInstance>,
    mut windows: ResMut<'w, RenderWindows>,
    mut views: ResMut<'w, TextureViews>,
) -> std::io::Result<()> {
    for (_, window) in windows.iter_mut() {
        let view = views.get_mut(window.view);

        assert!(view.is_some());
        let view = view.unwrap();

        let is_first = view.is_none();
        if is_first {
            let surface = unsafe { instance.create_surface(window.handle.deref()) };
            let surface = Arc::new(surface);
            *view = Some(TextureView::with_surface(surface));
        }
        let view = view.as_mut().unwrap();

        let PhysicalSize { width, height } = window.handle.inner_size();
        let config = wgpu::SurfaceConfiguration {
            format: TextureFormat::pi_render_default(),
            width,
            height,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: match window.present_mode {
                PresentMode::Fifo => wgpu::PresentMode::Fifo,
                PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
                PresentMode::Immediate => wgpu::PresentMode::Immediate,
            },
        };

        let is_size_changed =
            width != window.last_size.width || height != window.last_size.height;
        if is_size_changed {
            window.last_size.width = width;
            window.last_size.height = height;
        }
        // 记得 第一次 需要 Config
        if is_first || is_size_changed {
            let surface = view.surface().unwrap();
            device.configure_surface(surface, &config);
        }

        // 每帧 都要 设置 新的 SuraceTexture
        let _ = view.next_frame(&device, &config);
    }
    Ok(())
}
