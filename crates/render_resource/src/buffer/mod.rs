pub mod dyn_mergy_buffer;
// pub mod fixed_size_buffer;

#[derive(Debug, Clone, Copy)]
pub enum EErrorBuffer {
    AllocatorOverSize,
    SizeOverBlock,
}

pub trait TBufferLimit {
    fn alignment(&self) -> wgpu::BufferAddress;
}