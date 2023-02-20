use std::ops::Range;

pub enum EErrorMemory {
    OverSize,
}

pub trait TMemoryAllocatorLimit {
    fn max_size(&self) -> u64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryRange {
    pub(crate) start: usize,
    pub(crate) size: usize,
}
impl MemoryRange {
    pub fn new(start: usize, size: usize) -> Self {
        Self {
            start,
            size,
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

pub fn bytes_write_to_memory(
    bytes: &[u8],
    offset: usize,
    memory: &mut [u8],
) {
    let mut index = 0;
    for v in bytes.iter() {
        memory[offset + index] = *v;
        index += 1;
    }
}