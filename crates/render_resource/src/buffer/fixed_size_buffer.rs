use std::ops::Range;

use bitvec::vec::BitVec;
use pi_share::{Share, ShareMutex};


pub struct FixedSizeBufferAllocator {
    pub blocks: Vec<Share<FixedSizeBufferBlock>>,
    size: u64,
    block_size: usize,
    min_fixed_size: usize,
}
impl FixedSizeBufferAllocator {
    pub fn size(&self) -> u64 {
        self.size
    }
    pub fn allocate(&mut self, fixed_size: usize, count_maybe: usize) -> FixedSizeBufferRange {

    }
}

pub struct FixedSizeBufferBlock {
    fixed_size: usize,
    count: usize,
    used_index: BitVec,
    data: Vec<u8>,
    mutex: ShareMutex<()>,
}
impl FixedSizeBufferBlock {
    // pub fn new(fixed_size: usize, count: usize) {
    //     let mut used_index = BitVec::with_capacity(count);
    //     used_index.fill(false);

    //     let size = fixed_size * count;
    //     let Buffer = 
    // }
    pub fn allocate(&mut self, fixed_size: usize, count: usize) -> Option<FixedSizeBufferRange> {
        None
    }
    pub fn de_allocate(&mut self, info: &FixedSizeBufferRange) {

    }
}

pub struct FixedSizeBufferRange {
    pub block_index: usize,
    pub wait_list: Range<usize>,
    pub context: Share<FixedSizeBufferBlock>,
}
impl Drop for FixedSizeBufferRange {
    fn drop(&mut self) {
        let _lock = self.context.mutex.lock();
        let block = unsafe { &mut *(Share::as_ptr(&self.context) as usize as *mut FixedSizeBufferBlock) };
        block.de_allocate(self);
    }
}

pub trait TFixedSizeBufferArg {
    fn min_fixed_size(&self) -> usize;
}