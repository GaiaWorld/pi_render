
use pi_assets::{asset::Handle, mgr::AssetMgr};
use crate::{renderer::{attributes::EVertexDataKind, vertex_buffer::{EVertexBufferRange, VertexBufferAllocator, KeyVertexBuffer}}, rhi::{device::RenderDevice, RenderQueue}};
use pi_share::Share;

/// 用于标识目标类型实例化数据Buffer
pub trait TInstanceFlag {
    fn dirty(&self) -> bool;
    fn reset(&mut self);
}

/// * 用于实例化的数据
pub trait TInstancedData {
    /// 数据类型
    fn vertex_kind(&self) -> EVertexDataKind;
    /// 数据搜集处理
    fn collect(list: &Vec<&Self>, key: KeyVertexBuffer, device: &RenderDevice, queue: &RenderQueue, allocator: &mut VertexBufferAllocator, asset_mgr: &Share<AssetMgr<EVertexBufferRange>>) -> Option<Handle<EVertexBufferRange>> ;
}
