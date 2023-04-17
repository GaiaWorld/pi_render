

use crate::{renderer::{attributes::EVertexDataKind,}};


/// 用于标识目标类型实例化数据Buffer
pub trait TInstanceFlag {
    fn dirty(&self) -> bool;
    fn reset(&mut self);
}

/// * 用于实例化的数据
pub trait TInstanceData {
    /// 数据类型
    fn vertex_kind(&self) -> EVertexDataKind;
    /// 数据搜集处理
    fn collect(list: &Vec<&Self>) -> Vec<u8> ;
}
