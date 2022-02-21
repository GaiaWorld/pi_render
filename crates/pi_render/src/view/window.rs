use crate::rhi::{device::RenderDevice, texture::TextureView, RenderInstance};
use hash::{XHashMap, XHashSet};
use pi_ecs::prelude::{Res, ResMut, World};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::ops::{Deref, DerefMut};
use wgpu::TextureFormat;

pub type WindowId = u64;

/// Presentation mode for a window.
///
/// The presentation mode specifies when a frame is presented to the window. The `Fifo`
/// option corresponds to a traditional `VSync`, where the framerate is capped by the
/// display refresh rate. Both `Immediate` and `Mailbox` are low-latency and are not
/// capped by the refresh rate, but may not be available on all platforms. Tearing
/// may be observed with `Immediate` mode, but will not be observed with `Mailbox` or
/// `Fifo`.
///
/// `Immediate` or `Mailbox` will gracefully fallback to `Fifo` when unavailable.
///
/// The presentation mode may be declared in the [`WindowDescriptor`](WindowDescriptor::present_mode)
/// or updated on a [`Window`](Window::set_present_mode).
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[doc(alias = "vsync")]
pub enum PresentMode {
    /// The presentation engine does **not** wait for a vertical blanking period and
    /// the request is presented immediately. This is a low-latency presentation mode,
    /// but visible tearing may be observed. Will fallback to `Fifo` if unavailable on the
    /// selected platform and backend. Not optimal for mobile.
    Immediate = 0,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image, but frames may be submitted without delay. This is a low-latency
    /// presentation mode and visible tearing will **not** be observed. Will fallback to `Fifo`
    /// if unavailable on the selected platform and backend. Not optimal for mobile.
    Mailbox = 1,
    /// The presentation engine waits for the next vertical blanking period to update
    /// the current image. The framerate will be capped at the display refresh rate,
    /// corresponding to the `VSync`. Tearing cannot be observed. Optimal for mobile.
    Fifo = 2, // NOTE: The explicit ordinal values mirror wgpu and the vulkan spec.
}

/// This wrapper exist to enable safely passing a [`RawWindowHandle`] across threads. Extracting the handle
/// is still an unsafe operation, so the caller must still validate that using the raw handle is safe for a given context.
#[derive(Debug, Clone)]
pub struct RawWindowHandleWrapper(RawWindowHandle);

impl RawWindowHandleWrapper {
    pub(crate) fn new(handle: RawWindowHandle) -> Self {
        Self(handle)
    }

    /// # Safety
    /// This returns a [`HasRawWindowHandle`] impl, which exposes [`RawWindowHandle`]. Some platforms
    /// have constraints on where/how this handle can be used. For example, some platforms don't support doing window
    /// operations off of the main thread. The caller must ensure the [`RawWindowHandle`] is only used in valid contexts.
    pub unsafe fn get_handle(&self) -> HasRawWindowHandleWrapper {
        HasRawWindowHandleWrapper(self.0)
    }
}

// SAFE: RawWindowHandle is just a normal "raw pointer", which doesn't impl Send/Sync. However the pointer is only
// exposed via an unsafe method that forces the user to make a call for a given platform. (ex: some platforms don't
// support doing window operations off of the main thread).
// A recommendation for this pattern (and more context) is available here:
// https://github.com/rust-windowing/raw-window-handle/issues/59
unsafe impl Send for RawWindowHandleWrapper {}
unsafe impl Sync for RawWindowHandleWrapper {}

pub struct HasRawWindowHandleWrapper(RawWindowHandle);

// SAFE: the caller has validated that this is a valid context to get RawWindowHandle
unsafe impl HasRawWindowHandle for HasRawWindowHandleWrapper {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0
    }
}

pub struct RenderWindow {
    pub id: WindowId,
    pub handle: RawWindowHandleWrapper,
    pub physical_width: u32,
    pub physical_height: u32,
    pub present_mode: PresentMode,
    pub swap_chain_texture: Option<TextureView>,
    pub size_changed: bool,
}

#[derive(Default)]
pub struct RenderWindows {
    pub windows: XHashMap<WindowId, RenderWindow>,
}

impl Deref for RenderWindows {
    type Target = XHashMap<WindowId, RenderWindow>;

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
    surfaces: XHashMap<WindowId, wgpu::Surface>,
    /// List of windows that we have already called the initial `configure_surface` for
    configured_windows: XHashSet<WindowId>,
}

pub fn init_window(world: &mut World) {
    let windows = RenderWindows::default();
    world.insert_resource(windows);

    let surface = WindowSurfaces::default();
    world.insert_resource(surface);
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
            format: TextureFormat::pi_default(),
            width: window.physical_width,
            height: window.physical_height,
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
