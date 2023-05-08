use std::{fmt::Debug, hash::Hash};

use derive_deref_rs::Deref;
use pi_assets::asset::{Asset};
use pi_share::{Share, ShareMutex};

use super::id_alloter::{IdAlloterWithCountLimit, Index};


/// 单个buffer的分配器
#[derive(Deref)]
pub struct SmallStructAlloter<T: Default> {
    array: Vec<T>,
	// 空位标识
	#[deref]
    occupied_mark: IdAlloterWithCountLimit,
}

impl<T: Default> SmallStructAlloter<T> {
	/// 创建 BindGroup Buffer分配器
	/// - block_count：块数量
	/// - block_size： 每块的大小（单位：字节）
    pub fn new(item_count: u32) -> Self {
        let mut array = Vec::with_capacity(item_count as usize);
        for _ in 0..item_count {
            array.push(T::default());
        }
        Self {
            array: array,
            occupied_mark: IdAlloterWithCountLimit::new(item_count as u32),
        }
    }
    pub fn update(&mut self, offset: u32, value: T) {
        if (offset as usize) < self.array.len() {
            self.array[offset as usize] = value;
        }
    }
    pub fn data(&self, offset: u32) -> Option<&T> {
        self.array.get(offset as usize)
    }
    pub fn free(&mut self, index: usize) {
        self.array[index as usize] = T::default();
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IDFixedSmallStructAllocator {
    /// * 在 分配器池 中的序号
    pub pool_index: u32,
}


pub struct AssetFixedSmallStructAllocator<T: Default + 'static>(SmallStructAlloter<T>, usize);
impl<T: Default + 'static> Asset for AssetFixedSmallStructAllocator<T> {
    /// * 在 分配器池 中的序号
    type Key = usize;
    fn size(&self) -> usize {
        self.1 as usize
    }
}
impl<T: Default + 'static> AssetFixedSmallStructAllocator<T> {
    pub fn alloc(&self) -> Option<Index> {
        unsafe {
            let temp = &mut *(self as *const Self as usize as *mut Self);
            temp.0.alloc()
        }
    }
    pub fn fill(&self, index: Index, value: T) {
        unsafe {
            let temp = &mut *(self as *const Self as usize as *mut Self);
            temp.0.update(index.index(), value);
        }
    }
    pub fn free(&self, index: Index) {
        unsafe {
            let temp = &mut *(self as *const Self as usize as *mut Self);
            temp.0.free(index.index() as usize);
            temp.0.recycle(index);
        }
    }
    pub fn data(&self, index: Index) -> Option<&T> {
        self.0.data(index.index())
    }
}

pub trait TSmallStructID {
    const ID: u32;
}

/// * Hash, PartialEq, Eq 只判断 RWBufferRange 使用的 IDBindBuffer 是否相同 (也标识 Buffer 是否相同), 不判断数据是否相同
/// * 判断数据 是否相同需要 判断 offset, IDBindBuffer 是否都相同
#[derive(Clone)]
pub struct IDSmallStruct<T: Default + TSmallStructID + 'static> {
    index: Index,
    id_allocator: u32,
    id_type: u32,
    allocator: Share<AssetFixedSmallStructAllocator<T>>,
}
impl<T: Default + TSmallStructID + 'static> Debug for IDSmallStruct<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IDSmallStruct").field("index", &self.index).field("id_allocator", &self.id_allocator).finish()
    }
}
impl<T: Default + TSmallStructID + 'static> IDSmallStruct<T> {
    pub fn data(&self) -> Option<&T> {
        self.allocator.data(self.index)
    }
    pub fn id_allocator(&self) -> u32 {
        self.id_allocator
    }
}
impl<T: Default + TSmallStructID + 'static> Drop for IDSmallStruct<T> {
    fn drop(&mut self) {
        self.allocator.free(self.index);
    }
}
impl<T: Default + TSmallStructID + 'static> Hash for IDSmallStruct<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.id_type.hash(state);
        self.id_allocator.hash(state);
    }
}
impl<T: Default + TSmallStructID + 'static> PartialEq for IDSmallStruct<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id_allocator == other.id_allocator && self.index == other.index && self.id_type == other.id_type
    }
}
impl<T: Default + TSmallStructID + 'static> Eq for IDSmallStruct<T> {
    fn assert_receiver_is_total_eq(&self) {
        
    }
}

pub struct UseFixedSmallStructAllocator<T: Default + 'static>(Option<Share<AssetFixedSmallStructAllocator<T>>>);
impl<T: Default + 'static> UseFixedSmallStructAllocator<T> {
    pub fn new(item_count: u32, item_size: usize) -> Self {
        Self(Some(
            Share::new(
                AssetFixedSmallStructAllocator(SmallStructAlloter::new(item_count), item_count as usize * item_size)
            )
        ))
    }
} 

pub(crate) struct SmallStructAllocatorPool<T: Default + TSmallStructID + 'static> {
    /// * 大内存块列表 (第i个的尺寸为 i*block_count)
    pool: Vec<UseFixedSmallStructAllocator<T>>,
    /// * 大内存块的存储的基础数目
    item_count: u32,
    item_size: usize,
    mutex: ShareMutex<()>,
}
impl<T: Default + TSmallStructID + 'static> SmallStructAllocatorPool<T> {
    /// * `block_size` 大内存块的基础尺寸
    /// * `fixed_size` 目标区间尺寸
    pub(crate) fn new(
        block_count: u32,
        item_size: usize,
    ) -> Self {
        Self {
            pool: vec![],
            item_count: block_count,
            item_size,
            mutex: ShareMutex::new(()),
        }
    }
    pub fn recycle(&mut self) {
        for item in self.pool.iter_mut() {
            if let Some(v) = &mut item.0 {
                if v.0.occupied_mark.is_empty() {
                    item.0 = None;
                }
            }
        }
    }
    pub(crate) fn allocate(&mut self, val: T) -> Option<IDSmallStruct<T>> {
        let len = self.pool.len();
        let mut key_buffer = None;
        // 寻找可用区间
        for i in 0..len {
            if let Some(allocator) = self.pool.get(i) {
                if let Some(allocator) = &allocator.0 {
                    let _clock = self.mutex.lock();
    
                    if let Some(index) = allocator.alloc() {
                        allocator.fill(index, val);
                        return Some(
                            IDSmallStruct { index, id_allocator: i as u32, id_type: T::ID, allocator: allocator.clone() }
                        );
                    }
                } else {
                    key_buffer = Some(i);
                }
            }
        }

        // 寻找 是否有缓存 块
        if let Some(key_buffer) = key_buffer {
            let item_count = self.item_count * (key_buffer as u32 + 1);
            self.pool[key_buffer] = UseFixedSmallStructAllocator::new(item_count, self.item_size);

            let allocator = self.pool[key_buffer].0.as_ref().unwrap();

            if let Some(index) = allocator.alloc() {
                allocator.fill(index, val);
                return Some(
                    IDSmallStruct { index, id_allocator: key_buffer as u32, id_type: T::ID, allocator: allocator.clone() }
                );
            } else {
                return None;
            }
        } else {
            let item_count = self.item_count * (len as u32 + 1);
            self.pool.push(
                UseFixedSmallStructAllocator::new(item_count, self.item_size)
            );
            
            let allocator = self.pool[len].0.as_ref().unwrap();

            if let Some(index) = allocator.alloc() {
                allocator.fill(index, val);
                return Some(
                    IDSmallStruct { index, id_allocator: len as u32, id_type: T::ID, allocator: allocator.clone() }
                );
            } else {
                // log::warn!("Fail !!!!!!!!!!");
                return None;
            }
        };
    }
}