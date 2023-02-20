use std::{ops::Range};

use pi_share::{Share, ShareMutex};

use crate::base::{MemoryRange, TMemoryAllocatorLimit};

pub struct DynMergyMemoryAllocator {
    blocks: Vec<Share<DynMergyMemoryBlock>>,
    size: usize,
    max_size: u64,
    block_size: usize,
}
impl DynMergyMemoryAllocator {
    pub fn new<T: TMemoryAllocatorLimit>(t: &T, block_size: usize) -> Self {
        Self {
            blocks: vec![],
            size: 0,
            max_size: t.max_size(),
            block_size,
        }
    }
    pub fn allocate(&mut self, size: usize) -> DynMergyMemoryData {
        let len = self.blocks.len();
        for i in 0..len {
            let item = self.blocks.get(i).unwrap();
            let _lock = item.mutex.lock();
            let block = unsafe { &mut *(Share::as_ptr(item) as usize as *mut DynMergyMemoryBlock) };

            let temp = block.allocate(size);

            if let Some(temp) = temp {
                return DynMergyMemoryData::new(temp.start, temp.size, item.clone());
            }
        }

        let new_size = self.size + self.block_size;
        if (new_size as u64) < self.max_size {
            let mut item = DynMergyMemoryBlock::new(len, self.block_size);
            let temp = item.allocate(size);
            let item = Share::new(item);
            self.blocks.push(item.clone());
            if let Some(temp) = temp {
                return DynMergyMemoryData::new(temp.start, temp.size, item);
            } else {
                panic!("DynMergyMemoryAllocator Out Of Memory !!!")
            }
        } else {
            panic!("DynMergyMemoryAllocator Out Of Memory !!!")
        }
    }
}

#[derive(Debug)]
pub struct DynMergyMemoryBlock {
    pub index: usize,
    pub data: Vec<u8>,
    pub wait_list: Vec<MemoryRange>,
    mutex: ShareMutex<()>,
}
impl DynMergyMemoryBlock {
    pub fn new(index: usize, size: usize) -> Self {
        Self {
            index,
            data: Vec::with_capacity(size),
            wait_list: vec![MemoryRange::new(0, size)],
            mutex: ShareMutex::new(()),
        }
    }
    pub fn allocate(&mut self, size: usize) -> Option<MemoryRange> {
        self.wait_list.sort_by(|a, b| a.size.partial_cmp(&b.size).unwrap());

        let index = match self.wait_list.binary_search_by(|a| a.size.cmp(&size)) {
            Ok(index) => {
                index
            },
            Err(index) => {
                index
            },
        };

        if index >= self.wait_list.len() {
            None
        } else {
            let wait = self.wait_list.get_mut(index).unwrap();
            let useinfo = MemoryRange::new(wait.start, size);
            wait.start = wait.start + size;

            Some(useinfo)
        }
    }
    pub fn de_allocate(&mut self, data: &DynMergyMemoryData) {
        self.wait_list.push(MemoryRange::new(data.start, data.size));
        self.mergy();
    }
    fn mergy(&mut self) {
        self.wait_list.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
        let mut result = vec![];
        let mut start = 0;
        let mut size = 0;
        self.wait_list.iter().for_each(|item| {
            if start + size == item.start {
                size += item.size;
            } else {
                result.push(MemoryRange::new(start, size));
                start = item.start;
                size = item.size;
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct DynMergyMemoryData {
    start: usize,
    size: usize,
    context: Share<DynMergyMemoryBlock>,
}
impl DynMergyMemoryData {
    pub fn new(start: usize, size: usize, context: Share<DynMergyMemoryBlock>) -> Self {
        Self {
            start,
            size,
            context
        }
    }
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn size(&self) -> usize {
        self.size
    }
    pub fn range(&self) -> Range<usize> {
        Range { start: self.start, end: self.start + self.size }
    }
}
impl Drop for DynMergyMemoryData {
    fn drop(&mut self) {
        let _lock = self.context.mutex.lock();
        let block = unsafe { &mut *(Share::as_ptr(&self.context) as usize as *mut DynMergyMemoryBlock) };
        block.de_allocate(self);
    }
}