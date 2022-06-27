use derive_deref_rs::Deref;
use pi_assets::asset::Asset;
use wgpu::TextureView;

#[derive(Debug, Deref)]
pub struct RenderRes<T> {
	#[deref]
	value: T,
	size: usize,
}

impl<T> RenderRes<T>  {
	#[inline]
	pub fn new(value: T, size: usize) -> Self {
		Self { value, size }
	}
}

impl<T: 'static> Asset for RenderRes<T>{
	type Key = u64;

	fn size(&self) -> usize {
		self.size
	}
}

pub fn calc_texture_size(desc: wgpu::TextureDescriptor) -> usize {
	let size = (desc.size.width * desc.size.height * desc.size.depth_or_array_layers) as usize;
	// TODO
	match desc.format {
		wgpu::TextureFormat::Bgra8UnormSrgb | 
		wgpu::TextureFormat::Bgra8Unorm | 
		wgpu::TextureFormat::Rgba8Uint |
		wgpu::TextureFormat::Depth32Float => size * 4,
		wgpu::TextureFormat::Depth24Plus => size * 3,
		_ => size,
	}
}

pub struct TextureRes {
	pub width: u32,
	pub height: u32,
	pub texture_view: TextureView,
	size: usize,
}

impl TextureRes {
	pub fn new(width: u32, height: u32, size: usize, texture_view: TextureView) -> Self {
		Self { width, height, size, texture_view }
	}
}


impl Asset for TextureRes{
	type Key = u64;

	fn size(&self) -> usize {
		self.size
	}
}
