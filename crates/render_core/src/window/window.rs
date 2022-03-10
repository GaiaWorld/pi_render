use super::window_wrapper::RawWindowHandleWrapper;
use pi_share::ShareRefCell;

use raw_window_handle::HasRawWindowHandle;
pub use winit::window::Window as WindowInner;
pub use winit::window::WindowId as PiWindowId;

#[derive(Debug, Clone)]
pub struct PiWindow {
    present_mode: PresentMode,
    raw_window_handle: RawWindowHandleWrapper,
    inner: ShareRefCell<WindowInner>,
}

impl PiWindow {
    #[inline]
    pub fn new(inner: ShareRefCell<WindowInner>) -> Self {
        let handle = inner.raw_window_handle().clone();
        Self {
            inner,
            present_mode: PresentMode::Mailbox,
            raw_window_handle: RawWindowHandleWrapper::new(handle),
        }
    }

    #[inline]
    pub fn id(&self) -> PiWindowId {
        self.inner.id()
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.inner.inner_size().width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.inner.inner_size().height
    }

    #[inline]
    #[doc(alias = "set_vsync")]
    pub fn set_present_mode(&mut self, mode: PresentMode) {
        self.present_mode = mode
    }

    #[inline]
    #[doc(alias = "vsync")]
    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    pub fn raw_window_handle(&self) -> RawWindowHandleWrapper {
        self.raw_window_handle.clone()
    }
}

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
