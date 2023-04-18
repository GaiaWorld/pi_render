mod types;
mod instanced_buffer;

pub use types::*;
pub use instanced_buffer::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EInstanceKind {
    None,
    WorldMatrix,
    Color,
    TillOffset,
}
