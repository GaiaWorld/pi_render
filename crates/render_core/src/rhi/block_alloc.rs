use std::{mem::size_of};

use bitvec::prelude::BitVec;

/// 块分配器
#[derive(Debug)]
pub struct BlockAlloter {
	datas: Vec<Data>, // 数据列表
	each_data_max_count: usize, // 每个数据的最大块数量
	each_data_min_count: usize, // 每个数据的最小数量
	block_size: usize, // 块大小
	full_bits: BitVec, // 填充标识
	change_bits: BitVec, // 修改标识
}

impl BlockAlloter {

	pub fn new(block_size: usize, each_data_min_count: usize, each_data_max_count: usize) -> Self {
		Self {
			datas: Vec::new(),
			each_data_max_count,
			each_data_min_count,
			block_size,
			full_bits: BitVec::new(),
			change_bits: BitVec::new(),
		}
	}

	/// 分配块count为块数量
	pub fn alloc(&mut self, count: usize) -> BlockIndex {
		loop {
			let index = match self.full_bits.first_zero() {
				Some(r) => r,
				None => {
					if self.each_data_max_count < count {
						panic!("alloc fail, each_data_max_count < count");
					}
					self.full_bits.push(false);
					self.change_bits.push(false);
					self.datas.push(Data::width_capacity(self.each_data_min_count));
					self.full_bits.len() - 1
				},
			};
			
			if let Some(offset) = self.datas[index].alloc(self.each_data_max_count, count) {
				self.change_bits.set(index, true);
				return BlockIndex {
					index,
					offset,
					count,
				};
			} else {
				// 如果分配块数量为1也无法分配，这设置full_bits为true
				if count == 1 {
					self.full_bits.set(index, true);
				}
			}
		}
	}

	pub fn remove(&mut self, index: &BlockIndex) {
		if self.full_bits.len() <= index.index {
			return ;
		}
		self.full_bits.replace(index.index, false); // 设置为未填满状态
		self.datas[index.index].remove(index.offset, index.count);
		// 这里不需要清理change_bits,change_bits是一个二级缓冲，只删除change_bits下对应索引中的一个块，而不是对应索引中所有的块
	}

	// 填充数据
	pub fn full(&mut self, alloc_index: &BlockIndex, offset: usize, data: &[u8]) -> Result<(), String> {
		// 数据内容超出块边界，则不填充
		if offset + data.len() > self.block_size {
			return Err(format!("data content overflow block, block_size: {}, offset: {}, datalen: {}", self.block_size, offset, data.len()));
		}
		// 索引不存在
		if let Some(data1) = self.datas.get_mut(alloc_index.index) {
			data1.write_data(alloc_index.offset, offset, self.block_size, data);
			Ok(())
		} else {
			return Err(format!("full fail, index , block_size: {}, offset: {}, datalen: {}", self.block_size, offset, data.len()));
		}
	}
}

/// 块索引
#[derive(Debug)]
pub struct BlockIndex {
	index: usize, // 在data中的索引
	offset: usize, // 在data中指定Vec的偏移
	count: usize,
}

impl BlockIndex {
	pub fn index(&self) -> usize {
		self.index
	}

	pub fn offset(&self) -> usize {
		self.offset
	}
}

#[derive(Default, Debug)]
struct Data {
	value: Vec<u8>, // 数据内容
	full_bits: BitVec, // 填充位标识
	change_bits: BitVec, // 修改标识
}

impl Data {
	fn width_capacity(capacity: usize) -> Self {
		let len = (capacity as f32 /size_of::<usize>() as f32).ceil() as usize;
		Self {
			value: Vec::new(),
			full_bits: BitVec::with_capacity(len),
			change_bits: BitVec::with_capacity(len),
		}
	}

	fn alloc(&mut self, max_size: usize, count: usize) -> Option<usize> {
		// 长度等于max_size，表示已经满了,发，返回None
		if count == 0 {
			return None;
		}

		let mut iter = self.full_bits.iter_zeros();
		let mut item = iter.next();
		let len = self.full_bits.len();

		loop {
			let i = match item {
				Some(i) => i,
				None => self.full_bits.len(),
			};
			
			if i + count <= max_size {
				let end = i + count;
				let mut cur_end = end.min(len);

				if self.full_bits[i..cur_end].not_any() {
					self.full_bits[i..cur_end].fill(true);
					self.change_bits[i..cur_end].fill(true);
					while cur_end < end {
						self.full_bits.push(true);
						self.change_bits.push(true);
						cur_end += 1;
					}
					return Some(i);
				}
			} else {
				return None;
			}

			if item.is_none() {
				return None;
			}

			item = iter.next();
		}
	}

	fn remove(&mut self, index: usize, count: usize) {
		let end = index + count;
		if self.full_bits.len() < end {
			return;
		}

		self.full_bits[index..end].fill(false);
		self.change_bits[index..end].fill(false);
	}

	fn write_data(&mut self, index: usize, offset: usize, block: usize, data: &[u8]) {
		let len = self.full_bits.len() * block;
		let index = block * index + offset;
		if self.value.len() < len {
			self.value.reserve(len - self.value.len());
			unsafe {self.value.set_len(len)}
		}
		let len = data.len();
		unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), self.value.as_mut_ptr().add(index), len) }
	}
}


#[test]
fn test() {
	use pi_share::{Share, ShareMutex};
	let mut block_alloter = BlockAlloter::new(256, 200, 8000);
	let size = 8; // 位

	let mut blocks = Vec::new();
	for _i in 0..size {
		let time = std::time::Instant::now();
		blocks.push(block_alloter.alloc(1));
		println!("alloc====={:?}", std::time::Instant::now() - time);
	}
	println!("blocks====={:?}", blocks);
	println!(" block_alloter: {:?}", block_alloter);

	block_alloter.remove(&blocks[size - 1]);// 移除最后一个
	println!(" block_alloter: {:?}", block_alloter);
	blocks[size - 1] = block_alloter.alloc(2);
	println!("blocks====={:?}", blocks);
	println!(" block_alloter: {:?}", block_alloter);
	blocks.push(block_alloter.alloc(1));
	println!("blocks====={:?}", blocks);
	println!(" block_alloter: {:?}", block_alloter);

	let block_alloter = Share::new(ShareMutex::new(block_alloter));
	for i in blocks.iter() {
		let vec:Vec<u8> = Vec::with_capacity(256);
		let time = std::time::Instant::now();
		block_alloter.lock().full(i, 0, vec.as_slice()).unwrap();
		println!("time====={:?}", std::time::Instant::now() - time);
	}

	// let mut vec:Vec<u32> = Vec::with_capacity(16);
	// unsafe { vec.set_len(16)};
	// for i in 0..10 {
	// 	let time = std::time::Instant::now();
	// 	let mut hasher = DefaultHasher::default();
	// 	vec.hash(&mut hasher);
	// 	let v = hasher.finish();
	// 	println!("timexxx====={:?}, hash: {:?}", std::time::Instant::now() - time,v);
	// }
	// println!(" block_alloter: {:?}", block_alloter);
	
}

// #[test]
// fn test1() {
// 	let mut block_alloter = BlockAlloter::new(256, 200, 1000);
// 	let size = 8; // 位

// 	let mut blocks = Vec::new();
// 	for _i in 0..size {
// 		blocks.push(block_alloter.alloc(1));
// 	}
// 	println!("blocks====={:?}", blocks);
// 	println!(" block_alloter: {:?}", block_alloter);

// 	block_alloter.remove(&blocks[size - 2]);// 移除最后一个
// 	println!(" block_alloter: {:?}", block_alloter);
// 	blocks[size - 2] = block_alloter.alloc(2);
// 	println!("blocks====={:?}", blocks);
// 	println!(" block_alloter: {:?}", block_alloter);
// 	blocks.push(block_alloter.alloc(1));
// 	println!("blocks====={:?}", blocks);
// 	println!(" block_alloter: {:?}", block_alloter);
// }

// #[test]
// fn test2() {
// 	let mut block_alloter = BlockAlloter::new(256, 200, 1000);
// 	let size = 8; // 位

// 	let mut blocks = Vec::new();
// 	for _i in 0..size {
// 		blocks.push(block_alloter.alloc(1));
// 	}
// 	println!("blocks====={:?}", blocks);
// 	println!(" block_alloter: {:?}", block_alloter);

// 	block_alloter.remove(&blocks[size - 2]);
// 	block_alloter.remove(&blocks[size - 3]);
// 	println!(" block_alloter: {:?}", block_alloter);
// 	blocks[size - 2] = block_alloter.alloc(2);
// 	println!("blocks====={:?}", blocks);
// 	println!(" block_alloter: {:?}", block_alloter);
// 	blocks.push(block_alloter.alloc(1));
// 	println!("blocks====={:?}", blocks);
// 	println!(" block_alloter: {:?}", block_alloter);
// }