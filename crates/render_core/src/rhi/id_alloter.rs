use std::sync::atomic::{AtomicU32, Ordering};

use bitvec::vec::BitVec;
use crossbeam::queue::SegQueue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Index {
	version: u32,
	index: u32,
}

impl Index {
	#[inline]
	pub fn version(&self) -> u32 {
		self.version
	}
	#[inline]
	pub fn index(&self) -> u32 {
		self.index
	}
}

/// `IdAlloter` 结构体用于线程安全地分配和回收索引。
/// 结构体包含两个字段，`max_index`表示已分配索引的最大值，`recycled`用于存储曾经分配出去，后又被回收的索引
/// 分配索引时， 如果recycled长度大于0，将从recycled中弹出一个索引，否则，分配的索引值为`max_index`,并且`max_index`会自增1
pub struct IndexAlloter {
	max_index: AtomicU32,
	recycled: SegQueue<Index>
}

impl IndexAlloter {
	/// 构造方法
	pub fn new() -> Self {
		IndexAlloter {
			max_index: AtomicU32::new(0),
			recycled: SegQueue::new(),
		}
	}

	/// 分配一个索引
	pub fn alloc(&self) -> Index {
		// 如果recycled中存在回收索引，将从recycled中弹出一个索引，否则，分配的索引值为`max_index`,并且`max_index`会自增1
		match self.recycled.pop() {
			Some(r) => Index{
				index: r.index,
				version: r.version + 1,
			},
			None => Index {
				version: 0,
				index: self.max_index.fetch_add(1, Ordering::Relaxed)
			},
		}
	}

	/// 回收一个索引
	pub fn recycle(&self, id: Index) {
		self.recycled.push(id);
	}

	
	/// 已回收的索引个数
	pub fn recycle_len(&self) -> u32 {
		self.recycled.len() as u32
	}

	/// 当前已分配索引的最大值
	pub fn cur_max(&self) -> u32 {
		self.max_index.load(Ordering::Relaxed)
	}
}

/// `IdAlloterWithCountLimit`一个有数量限制的索引分配器
pub struct IdAlloterWithCountLimit {
	id_alloter: IndexAlloter,
	capacity: u32,
}

impl IdAlloterWithCountLimit {
	/// 构造方法
	pub fn new(capacity: u32) -> Self {
		Self {
			id_alloter: IndexAlloter::new(),
			capacity,
		}
	}

	/// 分配一个索引
	pub fn alloc(&self) -> Option<Index> {
		let id = self.id_alloter.alloc();
		if id.index() >= self.capacity { 
			// 如果已经分配到最大的id， 则返回None
			// 并且分配到的id永不释放（无所谓，该id不占用内存）
			None
		} else {
			Some(id)
		}
	}

	/// 回收一个索引
	#[inline]
	pub fn recycle(&self, id: Index) {
		self.id_alloter.recycle(id);
	}
	
	// 是否为空（一个id也未分配, 或分配过但又全部回收了）
	pub fn is_empty(&self) -> bool {
		let cur_max = self.id_alloter.cur_max();
		if cur_max >= self.capacity && self.id_alloter.recycle_len() == self.capacity{
			return true;
		}
		false
	}

	pub fn capacity(&self) -> u32 {
		self.capacity
	}
}


/// `OccupiedMark` 结构体用于跟踪一个数据结构的占用情况，其中占用情况由位向量表示。
/// 位向量有一个位的数组，每个位代表一个元素是否被占用，为 0 表示未被占用，为 1 表示已被占用。
pub struct OccupiedMarker {
    occupied_mark: BitVec,
}

impl OccupiedMarker {
    /// 创建一个新的 `OccupiedMark` 结构体实例。
    ///
    /// # Arguments
    ///
    /// * `count` - 需要被跟踪占用情况的元素数量。
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_allocator::occupied_mark::OccupiedMark;
    ///
    /// let mut mark = OccupiedMark::new(10);
    /// ```
    pub fn new(count: usize) -> Self {
        // 创建一个指定容量的 BitVec，BitVec 内部使用一个 Vec<u64> 对象来存储位信息
        let mut occupied_mark = BitVec::with_capacity(count);
        // 设置 BitVec 的长度为 count，因为 BitVec 内部使用一个 Vec<u64> 对象来存储位信息，所以这里必须使用 unsafe，因为实际上，BitVec 内部存储的长度可能远远超过 count
        unsafe { occupied_mark.set_len(count) };
        // 初始化 BitVec 中所有的位都是 false，即没有被占用
        occupied_mark[0..count].fill(false);
        Self {
            occupied_mark,
        }
    }

    /// 分配一个未被占用的元素，将其占用并返回其下标。
    ///
    /// # Returns
    ///
    /// 返回未被占用的元素下标，如果不存在未被占用的元素，则返回 None。
    ///
    /// # Examples
    ///
    /// ```
    /// use pi_render::rhi::dyn_uniform_buffer::OccupiedMark;
    ///
    /// let mut mark = OccupiedMark::new(10);
    /// let index = mark.alloc();
    /// ```
    #[inline]
    pub fn alloc(&mut self) -> Option<usize> {
        // 找到第一个为 0 的位，表示这个位可用，标记为 1，并返回该位的索引
        let r = self.occupied_mark.first_zero();
        if let Some(r) = r {
            self.occupied_mark.set(r, true);
        }
        r
    }

    /// 释放指定下标的元素，将其标记为未被占用。
    ///
    /// # Arguments
    ///
    /// * `index` - 需要被释放的元素下标。
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_allocator::occupied_mark::OccupiedMark;
    ///
    /// let mut mark = OccupiedMark::new(10);
    /// let index = mark.alloc();
    /// mark.free(index.unwrap());
    /// ```
    pub fn free(&mut self, index: usize) {
        self.occupied_mark.set(index, false);
    }

    /// 返回跟踪占用情况的元素数量。
    ///
    /// # Examples
    ///
    /// ```
    /// use memory_allocator::occupied_mark::OccupiedMark;
    ///
    /// let mark = OccupiedMark::new(10);
    /// let count = mark.len();
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.occupied_mark.len() as usize
    }
}