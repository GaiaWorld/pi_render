use std::{fmt::Debug, ops::Range};

use derive_deref_rs::Deref;
use pi_share::{Share, ShareMutex};
use smallvec::{smallvec, SmallVec};
use wgpu::{util::BufferInitDescriptor, BufferUsages, BufferDescriptor};

use super::{
    buffer::Buffer,
    device::RenderDevice,
    shader::WriteBuffer,
    RenderQueue, id_alloter::{IdAlloterWithCountLimit, Index},
};

// Buffer索引
pub enum BufferIndex {
	Align {index: AlignBufferIndex, level: u32, len: u32, align_buffer_alloter: Share<BufferContainer>}, // 对齐的buffer，与其他数据共享buffer， 延迟更新到显存，与显存对应的，在内存中也会存在一份
	Alone {buffer: Buffer, range: Range<usize>} // 独立的buffer，不延迟更新到显存，不存在对应的内存， 只存在显存
}

impl BufferIndex {
	pub fn range(&self) -> Range<wgpu::BufferAddress> {
		match self {
			BufferIndex::Align { level, len, index, .. } => {
				let offset = (index.index.index() * calc_blocksize_from_level(*level)) as wgpu::BufferAddress;
				offset..offset + *len as wgpu::BufferAddress
			},
			BufferIndex::Alone { range, .. } => range.start as wgpu::BufferAddress..range.end as wgpu::BufferAddress,
		}
	}

	pub fn buffer(&self) -> &wgpu::Buffer {
		match self {
			BufferIndex::Align { index, level, align_buffer_alloter, .. } => {
				align_buffer_alloter[*level as usize].buffer_layers[index.layer].wgpu_buffer().unwrap()
			},
			BufferIndex::Alone { buffer, .. } => buffer,
		}
	}
}

impl Debug for BufferIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Align { index, level, .. } => f.debug_struct("Align").field("index", index).field("level", level).finish(),
            Self::Alone { buffer, range } => f.debug_struct("Alone").field("buffer", buffer).field("range", range).finish(),
        }
    }
}

impl Drop for BufferIndex {
    fn drop(&mut self) {
		// 如果是对齐buffer的索引，需要在对齐分配器中释放
        if let BufferIndex::Align { index, level, align_buffer_alloter, .. } = self {
			align_buffer_alloter[*level as usize].buffer_layers[index.layer].recycle(index.index);
		}
    }
}

/// buffer分配器
pub struct BufferAlloter {
	// 预备在栈上的分配器， 最小MIN_ALIGN， 最大 MIN_ALIGN * Math.pow(2, 6)，即4k
	align_buffer_alloter: Share<BufferContainer>,

	// 最大对齐，长度大于该值的buffer，不会在align_buffer_alloter中分配，而是创建一个单独的buffer
	max_align: u32,
	usages: BufferUsages,
	init_size: u32,
	label: Option<String>,

	mutex: ShareMutex<()>,
	device: RenderDevice,
	queue: RenderQueue,
}

impl BufferAlloter {
	/// 创建buffer分配器，并指定最大对齐（超过该对齐的buffer， 会被分配为独立的buffer，而不会被分配在一个共享的大buffer中）
	pub fn new(device: RenderDevice, queue: RenderQueue, max_align: u32, usages: BufferUsages) -> Self {
		let r = max_align.next_power_of_two();
		Self { 
			mutex: ShareMutex::new(()),
			align_buffer_alloter: Share::new(SmallVec::new()),
			max_align: r,
			usages,
			init_size: 512,
			device,
			queue,
			label: Some("BufferAlloter buffer".to_string()),
		}
	}

	/// 分配buffer
	pub fn alloc(&self, data: &[u8]) -> BufferIndex {
		debug_assert!(calc_level(data.len()) < 32);
		// 不存在旧的索引，则直接创建
		if data.len() <= self.max_align as usize {
			// 长度在最大对齐范围内，则在align_buffer_alloter中分配
			let new_level = calc_level(data.len());
			let index = self.alloc_align(new_level);
			self.update_buffer_to_align(&index, new_level, data);
			BufferIndex::Align {
				index, 
				level: new_level,
				len: data.len() as u32,
				align_buffer_alloter: self.align_buffer_alloter.clone()
			}
		} else {
			// 长度超出最大对齐，则创建单独的buffer(是否使用队列提交buffer？TODO)
			let buffer = self.device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
				label: self.label.as_deref(),
				contents: data,
				usage: self.usages,
			});
			BufferIndex::Alone { range: 0..data.len() as usize, buffer, }
		}
	}

	/// 更新buffer
	pub fn update(&self, old: &mut BufferIndex, data: &[u8]) -> bool {
		debug_assert!(calc_level(data.len()) < 32);
		// 如果存在旧的索引， 检查当前buffer的长度是否小于等于旧的buffer长度
		match old {
			BufferIndex::Align{index, level, align_buffer_alloter, ..} => {
				if data.len() <= self.max_align as usize {
					let new_level = calc_level(data.len());
					if new_level <= *level {
						// 如果就索引能容纳新buffer， 则直接更新buffer
						self.update_buffer_to_align(index, *level, data);
						false
					} else {
						// 释放旧的索引
						align_buffer_alloter[*level as usize].buffer_layers[index.layer].recycle(index.index);
						// 否则，重新分配索引
						let new_index = self.alloc_align(new_level);
						self.update_buffer_to_align(&new_index, new_level, data);
						*index = new_index;
						*level = new_level;
						true
					}
				} else {
					// 长度超出最大对齐，则创建单独的buffer(是否使用队列提交buffer？TODO)
					let buffer = self.device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
						label: self.label.as_deref(),
						contents: data,
						usage: self.usages,
					});
					*old = BufferIndex::Alone { range: 0..data.len() as usize, buffer, };
					true
				}
			},
			BufferIndex::Alone{buffer, range} => {
				if data.len() > buffer.size() as usize {
					// 旧的buffer不能容纳新的数据， 则创建新的buffer(是否使用队列提交buffer？TODO)
					let new_buffer = self.device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
						label: self.label.as_deref(),
						contents: data,
						usage: self.usages,
					});
					*old = BufferIndex::Alone { range: 0..data.len() as usize, buffer: new_buffer, };
					true
				} else {
					// 旧的buffer能容纳新的数据，则更新原有buffer(这里更新修改为maped buffer？TODO)
					self.queue.write_buffer(
						&buffer,
						0,
						data,
					);
					*range = 0..data.len() as usize;
					false
				}
			},
		}
	}

	/// 更新buffer到显存
	/// 通常每帧调用一次
	#[inline]
    pub fn write_buffer(&self) {
		let _lock = self.mutex.lock();
		// 迭代所有对齐类型的buffer，写入显存
		for level_alloter in self.align_buffer_alloter.iter() {
			for layer in level_alloter.iter() {
				// SAFE： 需要保证没有其他场景使用align_buffer_alloter
				let layer = unsafe {&mut *(layer as *const SingleBufferAlloter as usize as *mut SingleBufferAlloter)};
				layer.write_buffer(&self.device, &self.queue, self.label.as_deref());
			}
		}
	}

	// 分配对齐的buffer
	fn alloc_align(&self, level: u32) -> AlignBufferIndex {
		let mut lock = None;
		let align_buffer_alloter = unsafe { &mut *(Share::as_ptr(&self.align_buffer_alloter) as usize as *mut BufferContainer) };

		if align_buffer_alloter.len() as u32 <= level {
			lock = Some(self.mutex.lock());
			for i in align_buffer_alloter.len() as u32..=level {
				let block_size = calc_blocksize_from_level(i);
				let mut init_block_count = self.init_size / block_size;
				if init_block_count == 0 {
					init_block_count = 1;
				}

				align_buffer_alloter.push(AlignBufferAlloter { 
					buffer_layers: smallvec![SingleBufferAlloter::new(init_block_count as usize, block_size, self.usages)], 
					lately_use_buffer: 0 
				});
			}
		}

		let block_size = calc_blocksize_from_level(level);
		let index = align_buffer_alloter[level as usize].alloc(|block_count| {
			SingleBufferAlloter::new(block_count, block_size, self.usages)
		}, match lock {
			Some(_) => None,
			None => Some(&self.mutex)
		});

		index
	}

	// 更新buffer到对齐buffer中
	fn update_buffer_to_align(&self, index: &AlignBufferIndex, level: u32, data: &[u8]) {
		let align_buffer_alloter = unsafe { &mut *(Share::as_ptr(&self.align_buffer_alloter) as usize as *mut BufferContainer) };
		align_buffer_alloter[level as usize].buffer_layers[index.layer].fill(index.index.index() * calc_blocksize_from_level(level), data);
	}
}

// buffer最小对齐字节数
// const MIN_ALIGN: u32 = 64;
// 最小等级（2^6 = MIN_ALIGN）
const MIN_LEVEL: usize = 6;

// 计算等级
#[inline]
fn calc_level(size: usize) -> u32 {
	debug_assert!(size != 0);
	(((size - 1) >> MIN_LEVEL << 1_usize).next_power_of_two()).trailing_zeros()
}

// 根据level计算块大小
#[inline]
fn calc_blocksize_from_level(level: u32) -> u32 {
	1<<(level + 6)
}

pub trait Alloter {
	fn alloc(&self) -> Option<Index>;
	
	fn capacity(&self) -> u32;
}

/// 对齐类型的buffer的索引
#[derive(Debug)]
pub struct AlignBufferIndex {
	layer: usize, // 层
	index: Index,
}

/// 对齐的buffer分配器
/// 用于分配指定对齐值的buffer（如用于分配对齐值为256的buffer）
#[derive(Deref)]
pub struct AlignBufferAlloter<Inner: Alloter> {
    // mutex: ShareMutex<()>,
	// 层列表，用于存储所有的buffer，保证已分配的buffer容量不变，当前所有层对应的buffer都不足以分配时，会创建新的层，
    #[deref]
	buffer_layers: SmallVec<[Inner; 1]>,
    // 最近使用的buffer索引（在buffers字段中的索引）
    lately_use_buffer: usize,
}

impl<Inner: Alloter> AlignBufferAlloter<Inner> {
	// 分配， 返回分配索引和层索引
	fn alloc<CF: Fn(usize)-> Inner>(&self, create_fn: CF, lock: Option<&ShareMutex<()>>) -> AlignBufferIndex {
		 // 如果最近分配过的buffer能继续分配，则直接返回分配结果
		 if let Some(r) = self.buffer_layers[self.lately_use_buffer].alloc() {
			return AlignBufferIndex{index: r, layer: self.lately_use_buffer};
		}

		// 否则，解锁，找到一个有空位的buffer分配，会创建新的buffer，使用新的buffer分配
		// SAFE: 这里转为可变，立即解锁，保证alloc在多线程下不冲突
		let buffers =
		unsafe { &mut *(self as *const Self as usize as *mut Self) };

		// 这里，锁由外部传入，如果传入None， 外部应该保证有更大范围的锁保护
		let _lock;
		if let Some(l) = lock {
			_lock = l.lock();
		}
		

		// 再次尝试分配（多线程结构下需要重新检查）
		if let Some(r) = self.buffer_layers[self.lately_use_buffer].alloc() {
			return AlignBufferIndex{index: r, layer: self.lately_use_buffer};
		}

		// 找到一个存在空闲位置的buffer组
		for (index, buffer) in buffers.buffer_layers.iter_mut().enumerate() {
			if let Some(r) = buffer.alloc() {
				buffers.lately_use_buffer = index;
				return AlignBufferIndex{index: r, layer: index} ;
			}
		}

		// 如果未找到，则创建新的
		let buffer_maps = create_fn(buffers.buffer_layers.last().unwrap().capacity() as usize * 2);
		let alloc_index = buffer_maps.alloc().unwrap();
		buffers.buffer_layers.push(buffer_maps);
		buffers.lately_use_buffer = self.buffer_layers.len() - 1;
		AlignBufferIndex{index: alloc_index, layer: self.lately_use_buffer}
	}
}

/// 单个buffer的分配器
#[derive(Deref)]
pub struct SingleBufferAlloter {
    buffer_map: BufferMap,
	// 空位标识
	#[deref]
    id_alloter: IdAlloterWithCountLimit,
}

impl SingleBufferAlloter {
	/// 创建 BindGroup Buffer分配器
	/// -block_count：块数量
	/// -block_size： 每块的大小（单位：字节）
    pub fn new(block_count: usize, block_size: u32, usage: wgpu::BufferUsages) -> Self {
        Self {
            buffer_map: BufferMap::new(block_size as usize * block_count, usage),
            id_alloter: IdAlloterWithCountLimit::new(block_count as u32),
        }
    }

	/// 在buffer中填充数据
	#[inline]
    pub fn fill<T: WriteBuffer + ?Sized>(&mut self, offset: u32, value: &T) {
        self.buffer_map.cache_buffer.full(offset, value);
    }

	/// 更新代码到显存
	#[inline]
    pub fn write_buffer(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        label: Option<&str>,
    ) -> bool {
        self.buffer_map.write_buffer(device, queue, label)
    }

	#[inline]
	pub fn wgpu_buffer(&self) -> Option<&Buffer> {
		self.buffer_map.wgpu_buffer()
	}
}

impl Alloter for SingleBufferAlloter {
	#[inline]
    fn alloc(&self) -> Option<Index> {
        self.id_alloter.alloc()
    }

	#[inline]
    fn capacity(&self) -> u32 {
        self.id_alloter.capacity() as u32
    }
}

/// buffer映射
/// 内存buffer到wgpu buffer的映射
#[derive(Deref)]
pub struct BufferMap {
	#[deref]
    cache_buffer: BufferCache,
    buffer: Option<Buffer>,
	usage: wgpu::BufferUsages,
}

impl BufferMap {
    pub fn new(len: usize, usage: wgpu::BufferUsages) -> Self {
        Self {
            cache_buffer: BufferCache::new(len),
            buffer: None,
			usage,
        }
    }

    #[inline]
    pub fn wgpu_buffer(&self) -> Option<&Buffer> {
        self.buffer.as_ref()
    }

    /// 写入buffer到显存，返回是否重新创建了buffer
    pub fn write_buffer(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        label: Option<&str>,
    ) -> bool {
        // 什么也没改变， 直接返回
        if self.cache_buffer.change_range.len() == 0 {
			if let None = &self.buffer {
				self.buffer = Some(device.create_buffer(&BufferDescriptor {
                    label,
                    usage: self.usage,
					size: self.cache_buffer.buffer().len() as wgpu::BufferAddress,
					mapped_at_creation: false,
                }));
				return true;
			}
            return false;
        }

        let r = match &self.buffer {
            Some(buffer) => {
				let mut start = self.cache_buffer.change_range.start as wgpu::BufferAddress ;
				let mut end = self.cache_buffer.change_range.end as wgpu::BufferAddress ;
				start = start / wgpu::COPY_BUFFER_ALIGNMENT * wgpu::COPY_BUFFER_ALIGNMENT;
				end = (end + wgpu::COPY_BUFFER_ALIGNMENT - 1) / wgpu::COPY_BUFFER_ALIGNMENT * wgpu::COPY_BUFFER_ALIGNMENT;
				// if start == 0 && end * 2 >= self.cache_buffer.buffer.len() as wgpu::BufferAddress {
				// 	end = self.cache_buffer.buffer.len() as wgpu::BufferAddress ;
				// }
                queue.write_buffer(
                    buffer,
                    start as u64,
                    &self.cache_buffer.buffer()[start as usize
                        ..end as usize],
                );

                false
            }
            None => {
                self.buffer = Some(device.create_buffer_with_data(&BufferInitDescriptor {
                    label,
                    usage: self.usage,
                    contents: self.cache_buffer.buffer(),
                }));
                // self.old_len = size;
                true
            }
        };
		self.cache_buffer.reset_change_range();
        r
    }
}

/// buffer缓存
/// 包含一个内存的buffer和这段buffer的修改范围
#[derive(Debug)]
pub struct BufferCache {
    buffer: Vec<u8>, // buffer
    change_range: Range<u32>, // 修改范围
}

impl BufferCache {
    #[inline]
    pub fn new(len: usize) -> Self {
        let mut buffer = Vec::with_capacity(len);
        unsafe { buffer.set_len(len) };
        Self {
            buffer,
            change_range: 0..0, //
        }
    }

    /// 在已分配位置上填充buffer
    pub fn full<T: WriteBuffer + ?Sized>(&mut self, index: u32, t: &T) {
        debug_assert!(self.buffer.len() >= (index + t.byte_len()) as usize);

        t.write_into(index, self.buffer.as_mut_slice());

        // 设置数据变化范围
        let start = index + t.offset();
        let end = start + t.byte_len();
        if self.change_range.len() == 0 {
            self.change_range.start = start;
            self.change_range.end = end;
            // println!(
            //     "full1======{:?}, {:?}",
            //     self.change_range,
            //     bytemuck::cast_slice::<_, f32>(&self.buffer)
            // );
            return;
        }
        if self.change_range.start > start {
            self.change_range.start = start;
        }
        if self.change_range.end < end {
            self.change_range.end = end;
        }

        // println!(
        //     "full======{:?}, {:?}, {}, {:?}",
        //     self.change_range,
        //     start,
        //     end,
        //     bytemuck::cast_slice::<_, f32>(&self.buffer),
        // );
    }

    #[inline]
    pub fn buffer(&self) -> &[u8] {
        self.buffer.as_slice()
    }

	#[inline]
	pub fn reset_change_range(&mut self) {
		self.change_range = 0..0;
	}
}

type BufferContainer = SmallVec<[AlignBufferAlloter<SingleBufferAlloter>; 7]>;

#[cfg(test)]
mod test {
    use std::sync::{Arc, atomic::AtomicBool};
    use pi_async_rt::rt::AsyncRuntime;
    use wgpu::{Gles3MinorVersion, InstanceFlags};
    use winit::{event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows};

    use crate::rhi::{device::initialize_renderer, options::RenderOptions};

    use super::BufferAlloter;

	#[test]
	fn test_alloc() {
		let is_end = Arc::new(AtomicBool::new(false));
		let is_end1 = is_end.clone();
		
		let options = RenderOptions::default();
		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
			// Which `Backends` to enable.
			backends: options.backends,
			// Which DX12 shader compiler to use.
			// dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
			flags: InstanceFlags::DEBUG,
			// gles_minor_version: Gles3MinorVersion::Automatic,
    		backend_options: wgpu::BackendOptions::default(),
		});
		let event_loop =  EventLoopBuilder::new().with_any_thread(true).build();
		let window = winit::window::Window::new(&event_loop).unwrap();

		

		pi_hal::runtime::MULTI_MEDIA_RUNTIME.spawn(async move {
			let surface = instance.create_surface(&window).unwrap();
			let request_adapter_options = wgpu::RequestAdapterOptions {
				power_preference: options.power_preference,
				compatible_surface: Some(&surface),
				..Default::default()
			};

			let mut alloter = pi_assets::allocator::Allocator::new(32 * 1024 * 1024);
			
			let (device, queue, _adapter_info) =
			initialize_renderer(&instance, &options, &request_adapter_options, &mut alloter).await;
			
			let _alloter = BufferAlloter::new(device, queue, 128, wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX);
			let mut level1_buffer = Vec::new();
			for _i in 0..68 {
				level1_buffer.push(1)
			}

			// level2， 由于max_align为128， 该buffer应该创建独立的buffer
			let mut level2_buffer = Vec::new();
			for _i in 0..132 {
				level2_buffer.push(1)
			}
			
			// level3， 由于max_align为128， 该buffer应该创建独立的buffer
			let mut level3_buffer = Vec::new();
			for _i in 0..136 {
				level3_buffer.push(1)
			}

			// // 测试分配在level0的情况
			// let mut r = alloter.alloc_or_update(None, &[0]);
			// println!("====测试分配在level0的情况====={:?}", r);
			// // 测试更新level0到level0的情况
			// let r = alloter.alloc_or_update(Some(r), &[1]);
			// println!("====测试更新level0到level0的情况====={:?}", r);
			// // 测试更新level0到level1的情况
			// let r = alloter.alloc_or_update(Some(r), level1_buffer.as_slice());
			// println!("====测试更新level0到level1的情况====={:?}", r);
			// // 测试更新level1到level2的情况
			// let r = alloter.alloc_or_update(Some(r), level2_buffer.as_slice());
			// println!("====测试更新level1到level2的情况====={:?}", r);
			// // 测试更新level2到level2的情况
			// let r = alloter.alloc_or_update(Some(r), level2_buffer.as_slice());
			// println!("====测试更新level2到level2的情况====={:?}", r);
			// // 测试更新level2到level3的情况
			// let r = alloter.alloc_or_update(Some(r), level3_buffer.as_slice());
			// println!("====测试更新level2到level3的情况====={:?}", r);
			is_end1.store(true, std::sync::atomic::Ordering::Relaxed);
		}).unwrap();

		loop {
			if is_end.load(std::sync::atomic::Ordering::Relaxed) {
				break;
			}
		}
	}
}



// #[test]
// fn tt() {
// 	// println!("{:?}", 0_u32.trailing_zeros());
// 	// println!("{:?}", 2_u32.trailing_zeros());
// 	// println!("{:?}", 1_u32.trailing_zeros());
// 	// println!("{:?}", 1_u32.next_power_of_two());
// 	// println!("{:?}", calc_level(0));
// 	// println!("{:?}",( 1_u32 >> 6 << 1).next_power_of_two().trailing_zeros());
// 	// println!("{:?}",( 64_u32 >> 6 << 1).next_power_of_two().trailing_zeros());
// 	println!("{:?}", calc_level(1));
// 	println!("{:?}", calc_level(64));
// 	println!("{:?}", calc_level(65));
// 	println!("{:?}", calc_level(129));
// 	println!("{:?}", calc_level(130));
// 	// (std::mem::size_of::<usize>() * 8) as u32 - (size >> 6_usize).next_power_of_two().leading_zeros()

// 	println!("{:?}", calc_blocksize_from_level(0));
// 	println!("{:?}", calc_blocksize_from_level(1));
// 	println!("{:?}", calc_blocksize_from_level(2));
// 	println!("{:?}", calc_blocksize_from_level(3));
// 	// println!("{:?}", std::mem::size_of::<BufferIndex>());
	
// }

