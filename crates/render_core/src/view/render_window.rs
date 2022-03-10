use crate::{
    rhi::{
        device::RenderDevice,
        texture::{PiRenderDefault, TextureView},
        RenderInstance, RenderSurface,
    },
    window::{
        window::{PiWindowId, PresentMode},
        window_wrapper::RawWindowHandleWrapper,
        windows::Windows,
    },
};
use log::debug;
use pi_ecs::prelude::{Res, ResMut, World};
use pi_hash::{XHashMap, XHashSet};
use std::ops::{Deref, DerefMut};
use wgpu::TextureFormat;

pub struct RenderWindow {
    pub id: PiWindowId,
    pub handle: RawWindowHandleWrapper,
    pub width: u32,
    pub height: u32,
    pub present_mode: PresentMode,
    pub swap_chain_texture: Option<TextureView>,
    pub size_changed: bool,
}

#[derive(Default)]
pub struct RenderWindows {
    pub windows: XHashMap<PiWindowId, RenderWindow>,
}

impl Deref for RenderWindows {
    type Target = XHashMap<PiWindowId, RenderWindow>;

    fn deref(&self) -> &Self::Target {
        &self.windows
    }
}

impl DerefMut for RenderWindows {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.windows
    }
}

#[derive(Default)]
pub struct WindowSurfaces {
    surfaces: XHashMap<PiWindowId, RenderSurface>,
    configured_windows: XHashSet<PiWindowId>,
}

pub fn init_window(world: &mut World) {
    world.insert_resource(RenderWindows::default());
    world.insert_resource(WindowSurfaces::default());
}

pub fn extract_windows(windows: Res<Windows>, mut render_windows: ResMut<RenderWindows>) {
    for window in windows.iter() {
        let new_width = window.width().max(1);
        let new_height = window.height().max(1);

        let render_windows = render_windows.deref_mut();

        let mut render_window = render_windows.entry(window.id()).or_insert(RenderWindow {
            id: window.id(),
            handle: window.raw_window_handle(),
            width: new_width,
            height: new_height,
            present_mode: window.present_mode(),
            swap_chain_texture: None,
            size_changed: false,
        });

        // NOTE: Drop the swap chain frame here
        render_window.swap_chain_texture = None;
        render_window.size_changed =
            new_width != render_window.width || new_height != render_window.height;

        if render_window.size_changed {
            debug!(
                "Window size changed from {}x{} to {}x{}",
                render_window.width, render_window.height, new_width, new_height
            );
            render_window.width = new_width;
            render_window.height = new_height;
        }
    }
}

pub fn prepare_windows(
    mut windows: ResMut<RenderWindows>,
    mut window_surfaces: ResMut<WindowSurfaces>,
    render_device: Res<RenderDevice>,
    render_instance: Res<RenderInstance>,
) {
    let window_surfaces = window_surfaces.deref_mut();
    for window in windows.windows.values_mut() {
        let surface = window_surfaces
            .surfaces
            .entry(window.id)
            .or_insert_with(|| unsafe {
                // NOTE: On some OSes this MUST be called from the main thread.
                render_instance.create_surface(&window.handle.get_handle())
            });

        let swap_chain_descriptor = wgpu::SurfaceConfiguration {
            format: TextureFormat::pi_render_default(),
            width: window.width,
            height: window.height,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            present_mode: match window.present_mode {
                PresentMode::Fifo => wgpu::PresentMode::Fifo,
                PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
                PresentMode::Immediate => wgpu::PresentMode::Immediate,
            },
        };

        // Do the initial surface configuration if it hasn't been configured yet
        if window_surfaces.configured_windows.insert(window.id) || window.size_changed {
            render_device.configure_surface(surface, &swap_chain_descriptor);
        }

        let frame = match surface.get_current_texture() {
            Ok(swap_chain_frame) => swap_chain_frame,
            Err(wgpu::SurfaceError::Outdated) => {
                render_device.configure_surface(surface, &swap_chain_descriptor);
                surface
                    .get_current_texture()
                    .expect("Error reconfiguring surface")
            }
            err => err.expect("Failed to acquire next swap chain texture!"),
        };

        window.swap_chain_texture = Some(TextureView::from(frame));
    }
}
