use std::ops::Range;

use bitvec::vec::BitVec;
use pi_share::{Share, ShareMutex};


pub struct FixedSizeMemoryAllocator {
    pub blocks: Vec<Share<FixedSizeMemoryBlock>>,
    size: u64,
    block_size: usize,
    min_fixed_size: usize,
}
impl FixedSizeMemoryAllocator {
    pub fn size(&self) -> u64 {
        self.size
    }
    // pub fn allocate(&mut self, fixed_size: usize, count_maybe: usize) -> FixedSizeMemoryData {

    // }
}

pub struct FixedSizeMemoryBlock {
    fixed_size: usize,
    count: usize,
    used_index: BitVec,
    memory: Vec<u8>,
    mutex: ShareMutex<()>,
}
impl FixedSizeMemoryBlock {
    // pub fn new(fixed_size: usize, count: usize) {
    //     let mut used_index = BitVec::with_capacity(count);
    //     used_index.fill(false);

    //     let size = fixed_size * count;
    //     let memory = 
    // }
    pub fn allocate(&mut self, fixed_size: usize, count: usize) -> Option<FixedSizeMemoryData> {

        None
    }
    pub fn de_allocate(&mut self, info: &FixedSizeMemoryData) {

    }
}

pub struct FixedSizeMemoryData {
    pub block_index: usize,
    pub wait_list: Range<usize>,
    pub context: Share<FixedSizeMemoryBlock>,
}
impl Drop for FixedSizeMemoryData {
    fn drop(&mut self) {
        let _lock = self.context.mutex.lock();
        let block = unsafe { &mut *(Share::as_ptr(&self.context) as usize as *mut FixedSizeMemoryBlock) };
        block.de_allocate(self);
    }
}

pub trait TFixedSizeMemoryArg {
    fn min_fixed_size(&self) -> usize;
}