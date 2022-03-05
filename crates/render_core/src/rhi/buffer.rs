use std::{
    ops::{Bound, Deref, RangeBounds},
    sync::Arc,
};

/// 可Clone的 Buffer
#[derive(Clone, Debug)]
pub struct Buffer(Arc<wgpu::Buffer>);

impl Buffer {
    /// 取 切片
    pub fn slice(&self, bounds: impl RangeBounds<wgpu::BufferAddress>) -> BufferSlice {
        BufferSlice {
            // need to compute and store this manually because wgpu doesn't export offset on wgpu::BufferSlice
            offset: match bounds.start_bound() {
                Bound::Included(&bound) => bound,
                Bound::Excluded(&bound) => bound + 1,
                Bound::Unbounded => 0,
            },
            value: self.0.slice(bounds),
        }
    }
    
    /// 取消 映射
    #[inline]
    pub fn unmap(&self) {
        self.0.unmap()
    }
}

impl From<wgpu::Buffer> for Buffer {
    fn from(value: wgpu::Buffer) -> Self {
        Buffer(Arc::new(value))
    }
}

impl Deref for Buffer {
    type Target = wgpu::Buffer;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// 可Clone的 Buffer 切片
#[derive(Clone, Debug)]
pub struct BufferSlice<'a> {
    offset: wgpu::BufferAddress,
    value: wgpu::BufferSlice<'a>,
}

impl<'a> BufferSlice<'a> {
    /// 取 偏移
    #[inline]
    pub fn offset(&self) -> wgpu::BufferAddress {
        self.offset
    }
}

impl<'a> Deref for BufferSlice<'a> {
    type Target = wgpu::BufferSlice<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}