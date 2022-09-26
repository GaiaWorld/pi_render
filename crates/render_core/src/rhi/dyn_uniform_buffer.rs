
use bitvec::vec::BitVec;
use derive_deref_rs::Deref;
use pi_share::{ShareMutex, Share};
use wgpu::{util::BufferInitDescriptor, BufferUsages};

use super::{device::RenderDevice, RenderQueue, buffer::Buffer, bind_group_layout::BindGroupLayout, bind_group::BindGroup};

pub struct DynUniformBuffer {
	cache_buffer: Share<ShareMutex<DynBuffer>>,
	buffer: Option<Buffer>,
    capacity: usize,
    label: Option<String>,
	is_change: bool,
}

impl DynUniformBuffer {
	pub fn new(label: Option<String>, alignment: u32) -> Self {
		Self {
			cache_buffer: Share::new(ShareMutex::new(DynBuffer::new(alignment))),
			buffer: None,
			capacity: 0,
			label,
			is_change: false,
		}
	}

	#[inline]
	pub fn buffer(&self) -> Option<&Buffer> {
		self.buffer.as_ref()
	}

	#[inline]
	pub fn alloc_binding<T: Bind>(&mut self) -> BindOffset {
		BindOffset{
			offset: self.cache_buffer.lock().alloc::<T>(),
			context: self.cache_buffer.clone()
		}
		
	}

	#[inline]
	pub fn set_uniform<T: Uniform>(&mut self, binding_offset: &BindOffset, t: &T) {
		self.cache_buffer.lock().full::<T>(**binding_offset, t);
		self.is_change = true;
	}

	/// 写入buffer到现存，返回是否重新创建了buffer
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) -> bool {
        let size = self.cache_buffer.lock().buffer().len();

        if self.capacity < size {
            self.buffer = Some(device.create_buffer_with_data(&BufferInitDescriptor {
                label: self.label.as_deref(),
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                contents: self.cache_buffer.lock().buffer(),
            }));
            self.capacity = size;
			self.is_change = true;
			return true;
        } else if let Some(buffer) = &self.buffer {
			if self.is_change {
				queue.write_buffer(buffer, 0, self.cache_buffer.lock().buffer());
				self.is_change = false
			}
		}
		false
	}

	pub fn capacity(&self) -> usize {
		self.capacity
	}	
}

#[derive(Debug)]
pub struct BindOffset {
	offset: u32,
	context: Share<ShareMutex<DynBuffer>>,
}

impl std::ops::Deref for BindOffset {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.offset
    }
}

impl Drop for BindOffset {
    fn drop(&mut self) {
        self.context.lock().remove(self.offset);
    }
}

pub trait Uniform {
	// 将自身写入buffer缓冲区，假定buffer的容量足够，否则崩溃
	fn write_into(&self, index: u32, buffer: &mut [u8]);
}

#[derive(Clone, Copy, Deref)]
pub struct BindIndex(usize);

impl BindIndex {
	pub fn new(index: usize) -> Self {
		Self(index)
	}
}

pub type GroupId = u32;

pub type BindId = u32;

pub trait Bind {
	fn min_size() -> usize;

	// 在bindings数组中的位置
	fn index() -> BindIndex;
}

pub trait Group {
	fn id() -> GroupId;
	fn create_layout(device: &RenderDevice, has_dynamic_offset: bool) -> BindGroupLayout;
}

pub trait BufferGroup {
	fn create_bind_group(device: &RenderDevice, layout: &BindGroupLayout, buffer: &Buffer) -> BindGroup;
}

#[derive(Debug)]
pub struct DynBuffer {
	tail: usize,
	buffer: Vec<u8>,
	alignment: usize,
	
	full_bits: BitVec, // 填充位标识

	end_bits: BitVec, // 结束索引
}

impl DynBuffer {
	#[inline]
	pub fn buffer(&self) -> &[u8] {
		self.buffer.as_slice()
	}

	#[inline]
	pub fn new(alignment: u32) -> Self {
		Self { 
			buffer: Vec::default(),  
			alignment: alignment as usize,
			full_bits: BitVec::default(),
			end_bits: BitVec::default(),
			tail: 0,
		}
	}

	pub fn alloc<T: Bind>(&mut self) -> u32 {
		let count = ( T::min_size() as f32 / self.alignment as f32).ceil() as usize;

		let mut iter = self.full_bits.iter_zeros();
		let mut item = iter.next();

		let len = self.full_bits.len();
		loop {
			let i = match item {
				Some(i) => i,
				None => self.full_bits.len(),
			};
			
			let end = i + count;
			let mut cur_end = end.min(len);

			if self.full_bits[i..cur_end].not_any() {
				self.full_bits[i..cur_end].fill(true);
				while cur_end < end {
					self.full_bits.push(true);
					cur_end += 1;
				}

				// 设置结束标识
				let l = self.end_bits.len();
				if l < end {
					self.end_bits.reserve(end - l);
					unsafe { self.end_bits.set_len(end) };
					self.end_bits[l..end - 1].fill(false);
				}
				self.end_bits.set(end - 1, true);

				self.tail = self.tail.max((i + count) * self.alignment);
				return (i * self.alignment) as u32;
			}
			item = iter.next();
		}
	}

	/// 移除buffer分配
	pub fn remove(&mut self, mut i: u32) {
		i = i / self.alignment as u32;
		let len = self.end_bits.len();
		while (i as usize) < len {
			self.full_bits.set(i as usize, false);
			i = i + 1;
			if unsafe { self.end_bits.replace_unchecked((i - 1) as usize, false) } {
				return;
			}
		}
	}

	pub fn full<T: Uniform>(&mut self, index: u32, t: &T){
		if self.buffer.len() < self.tail {
			self.buffer.reserve(self.tail - self.buffer.len());
			unsafe {self.buffer.set_len(self.tail)}
		}

		t.write_into(index, self.buffer.as_mut_slice());
	}
}