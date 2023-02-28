
use crate::{renderer::vertex_buffer::{KeyVertexBuffer}};

use crate::renderer::vertex_buffer_desc::EVertexBufferSlot;

pub trait TInstancedBuffer {
    /// * Buffer 类型名称
    /// * `example` "InstanceWorldMatrix"
    fn display_name() -> String;
    /// * 数据在 顶点Buffer数据的第几个槽位
    fn slot(&self) -> EVertexBufferSlot;
    /// * Buffer ID
    /// * 内部应当记录该方法调用的次数, 并体现在返回值上 `每次更新都认为是一个新数据`
    fn id(&mut self) -> KeyVertexBuffer;
}