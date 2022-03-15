use crate::{
    rhi::{
        device::RenderDevice,
        texture::{PiRenderDefault, TextureView},
        PresentMode, RenderInstance, RenderSurface,
    },
    RenderArchetype,
};
use pi_ecs::{
    prelude::{Query, Res, World},
    world::ArchetypeInfo,
};
use pi_share::ShareRefCell;
use std::ops::Deref;
use wgpu::TextureFormat;
use winit::{dpi::PhysicalSize, window::Window};

pub struct RenderWindow {
    pub present_mode: PresentMode,
    pub last_size: PhysicalSize<u32>,
    pub handle: ShareRefCell<Window>,

    pub rt: Option<TextureView>,
    pub surface: Option<RenderSurface>,
}

impl RenderWindow {
    pub fn new(handle: ShareRefCell<Window>, present_mode: PresentMode) -> Self {
        Self {
            present_mode,
            last_size: PhysicalSize::default(),
            handle,
            surface: None,
            rt: None,
        }
    }
}

#[inline]
pub fn register_components(archetype: ArchetypeInfo) -> ArchetypeInfo {
    archetype.register::<RenderWindow>()
}

#[inline]
pub fn insert_resources(_world: &mut World) {}

pub fn prepare_windows(
    render_device: Res<RenderDevice>,
    render_instance: Res<RenderInstance>,
    mut query: Query<RenderArchetype, &mut RenderWindow>,
) {
    for mut window in query.iter_mut() {
        let is_surface_none = window.surface.is_none();
        if is_surface_none {
            let handle = window.handle.deref();
            window.surface = Some(unsafe { render_instance.create_surface(handle) });
        };

        let surface = window.surface.as_ref().unwrap();
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

        // 记得 第一次 需要 Config
        let is_size_changed = width != window.last_size.width || height != window.last_size.height;
        let has_config = is_surface_none || is_size_changed;
        if has_config {
            render_device.configure_surface(surface, &config);
        }

        // 每帧 都要 设置 新的 SuraceTexture
        let frame = match surface.get_current_texture() {
            Ok(swap_chain) => swap_chain,
            Err(wgpu::SurfaceError::Outdated) => {
                render_device.configure_surface(surface, &config);
                surface
                    .get_current_texture()
                    .expect("Error reconfiguring surface")
            }
            err => err.expect("Failed to acquire next swap chain texture!"),
        };

        window.rt = Some(TextureView::from(frame));
    }
}
