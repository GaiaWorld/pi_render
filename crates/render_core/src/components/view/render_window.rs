use crate::{
    rhi::{
        device::RenderDevice,
        texture::{PiRenderDefault, ScreenTexture},
        PresentMode, RenderInstance,
    },
};
use pi_ecs::prelude::{Res, ResMut, World, res::WriteRes};
use pi_share::Share;
use pi_slotmap::{new_key_type, SlotMap};
use std::{ops::Deref};
use wgpu::TextureFormat;
use winit::{dpi::PhysicalSize, window::Window};

new_key_type! {
    pub struct RenderWindowKey;
}

pub type RenderWindows = SlotMap<RenderWindowKey, RenderWindow>;

pub struct RenderWindow {
    pub present_mode: PresentMode,
    last_size: PhysicalSize<u32>,
    pub handle: Share<Window>,
}

impl RenderWindow {
    pub fn new(
        handle: Share<Window>,
        present_mode: PresentMode,
    ) -> Self {
        Self {
            handle,
            present_mode,
            last_size: PhysicalSize::default(),
        }
    }
}

#[inline]
pub fn insert_resources(world: &mut World) {
    world.insert_resource(RenderWindows::default());
}

pub(crate) async fn prepare_windows<'w>(
    device: Res<'w, RenderDevice>,
    instance: Res<'w, RenderInstance>,
    mut windows: ResMut<'w, RenderWindows>,
    mut view: WriteRes<'w, ScreenTexture>,
) -> std::io::Result<()> {
    for (_, window) in windows.iter_mut() {
        // let view = views.get_mut(window.view);

        // assert!(view.is_some());
        // let view = view.unwrap();

        let is_first = view.get().is_none();
        if is_first {
            let surface = unsafe { instance.create_surface(window.handle.deref()) };
            let surface = Share::new(surface);
			view.write(ScreenTexture::with_surface(surface));
        }

		let view = view.get_mut().unwrap();

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
            device.configure_surface(view.surface(), &config);
        }

		// log::warn!("next_frame========================");
        // 第一次 设置 一下 新的 SuraceTexture
        if is_first{
            let _ = view.next_frame(&device, &config);
        }
    }
    Ok(())
}
