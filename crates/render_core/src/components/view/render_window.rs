use crate::{
    components::{camera::render_target::{TextureViewKey, TextureViews}, option_slotmap::OptionSlotMap},
    rhi::{device::RenderDevice, texture::{PiRenderDefault, TextureView}, PresentMode, RenderInstance},
};
use pi_ecs::prelude::{Res, ResMut, World};
use pi_share::ShareRefCell;
use pi_slotmap::new_key_type;
use std::{ops::Deref, sync::Arc};
use wgpu::TextureFormat;
use winit::{dpi::PhysicalSize, window::Window};

new_key_type! {
    pub struct RenderWindowKey;
}

pub type RenderWindows = OptionSlotMap<RenderWindowKey, RenderWindow>;

pub struct RenderWindow {
    pub present_mode: PresentMode,
    pub last_size: PhysicalSize<u32>,
    pub handle: ShareRefCell<Window>,
    pub view: TextureViewKey,
}

#[inline]
pub fn insert_resources(world: &mut World) {
    world.insert_resource(RenderWindows::default());
}

pub fn prepare_windows(
    device: Res<RenderDevice>,
    instance: Res<RenderInstance>,
    mut windows: ResMut<RenderWindows>,
    mut views: ResMut<TextureViews>,
) {
    for (_, window) in windows.iter_mut() {
        let (view, is_first) = match views.get_mut(window.view) {
            Some(view) => (view, true),
            None => {
                let surface = unsafe { instance.create_surface(window.handle.deref()) };
                let surface = Arc::new(surface);
                window.view = views.insert(Some(TextureView::with_surface(surface)));
                
                let view = views.get_mut(window.view);
                (view.unwrap(), false)
            }
        };

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

        let is_size_changed = width != window.last_size.width || height != window.last_size.height;
        // 记得 第一次 需要 Config
        if is_first || is_size_changed {
            let surface = view.surface().unwrap();
            device.configure_surface(surface, &config);
        }

        // 每帧 都要 设置 新的 SuraceTexture
        let _ = view.next_frame();
    }
}
