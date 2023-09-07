use std::ops::{Bound, Deref, RangeBounds};
use pi_share::Share;
use uuid::Uuid;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct BufferId(Uuid);

/// 可Clone的 Buffer
#[derive(Clone, Debug)]
pub struct Buffer {
    id: BufferId,
    value: Share<wgpu::Buffer>,
    size: wgpu::BufferAddress,
}

impl Buffer {

    #[inline]
    pub fn id(&self) -> BufferId {
        self.id
    }

    #[inline]
    pub fn size(&self) -> wgpu::BufferAddress {
        self.size
    }

    /// 取 切片
    pub fn slice(&self, bounds: impl RangeBounds<wgpu::BufferAddress>) -> BufferSlice {
        BufferSlice {
            id: self.id,
            // need to compute and store this manually because wgpu doesn't export offset on wgpu::BufferSlice
            offset: match bounds.start_bound() {
                Bound::Included(&bound) => bound,
                Bound::Excluded(&bound) => bound + 1,
                Bound::Unbounded => 0,
            },
            value: self.value.slice(bounds),
        }
    }
    
    // /// 取消 映射
    // #[inline]
    // pub fn unmap(&self) {
    //     self.value.unmap()
    // }
}

impl From<(wgpu::Buffer, wgpu::BufferAddress)> for Buffer {
    fn from(value: (wgpu::Buffer, wgpu::BufferAddress)) -> Self {
        Buffer {
            id: BufferId(Uuid::new_v4()),
            value: Share::new(value.0),
            size: value.1,
        }
    }
}

impl Deref for Buffer {
    type Target = wgpu::Buffer;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// 可Clone的 Buffer 切片
#[derive(Clone, Debug)]
pub struct BufferSlice<'a> {
    id: BufferId,
    offset: wgpu::BufferAddress,
    value: wgpu::BufferSlice<'a>,
}

impl<'a> BufferSlice<'a> {
    
    #[inline]
    pub fn id(&self) -> BufferId {
        self.id
    }

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