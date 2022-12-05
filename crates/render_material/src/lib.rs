// pub mod texture;
// pub mod material;
pub mod error;
// pub mod uniform_info;
// pub mod binding;
// pub mod buffer;

use render_core::rhi::buffer::Buffer;
use render_data_container::TMaterialBlockKindKey;

pub trait TBufferAllocator {
    fn allocat(&mut self, size: u32) -> (Buffer, u32);
}

pub struct UniformSetMeta {
    pub set: u32,

}

pub struct UniformBindMeta {
    pub bind: u32,
    pub offset: u32,
    pub size: u32,
}

/// Uniform 元数据
pub struct UniformMeta {
    pub offset: u32,
}

pub struct MaterialPropertypeBlock<MBKK: TMaterialBlockKindKey> {
    pub kinds: Vec<MBKK>,
    // pub datas: Vec<Dyna>,
}


