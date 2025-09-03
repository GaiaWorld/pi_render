
#[macro_use]
extern crate lazy_static;

mod target_alloc;
#[cfg(feature = "debug_info")]
pub mod target_alloc_debug;

pub use target_alloc::*;
