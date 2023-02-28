pub mod types;
pub mod instanced_buffer;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum EInstanceKind {
    None,
    WorldMatrix,
    Color,
    TillOffset,
}
