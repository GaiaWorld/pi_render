//! 定义 和 glsl 的 `std140` 布局 相匹配 的 类型 和 Trait

mod dynamic_uniform;
mod primitives;
mod sizer;
mod traits;
#[cfg(feature = "std")]
mod writer;

pub use self::dynamic_uniform::*;
pub use self::primitives::*;
pub use self::sizer::*;
pub use self::traits::*;
#[cfg(feature = "std")]
pub use self::writer::*;

use mint::Vector3;
pub use pi_crevice_derive::AsStd140;