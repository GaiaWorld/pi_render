
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub use wgpu::*;
// #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
// pub use pi_wgpu::*;
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub use wgpu::*;