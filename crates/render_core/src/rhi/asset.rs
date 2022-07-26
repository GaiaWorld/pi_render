use std::io::ErrorKind;

use async_trait::async_trait;
use derive_deref_rs::Deref;
use pi_assets::{asset::{Asset, Garbageer, Handle}, mgr::{LoadResult, Receiver}};
use pi_atom::Atom;
use pi_hal::{
	image::DynamicImage,
	loader::AsyncLoader,
};
use wgpu::TextureView;

use super::{device::RenderDevice, RenderQueue};

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
	pub is_opacity: bool,
	size: usize,
}

impl TextureRes {
	pub fn new(width: u32, height: u32, size: usize, texture_view: TextureView, is_opacity: bool) -> Self {
		Self { width, height, size, texture_view, is_opacity }
	}
}


impl Asset for TextureRes{
	type Key = u64;

	fn size(&self) -> usize {
		self.size
	}
}

// match result {
// 	LoadResult::Ok(r) => Ok(r),
// 	LoadResult::Wait(f) => f.await,
// 	LoadResult::Receiver(recv) => {
// 		let image = pi_hal::image::load_from_path(&image_assets_mgr, url).await;
// 		let image = match image {
// 			Ok(r) => r,
// 			Err(e) =>  {
// 				log::error!("load image fail: {:?}", key.as_str());
// 				return Err(std::io::Error::from(""));
// 			},
// 		};

// 		let texture = create_texture_from_image(&image, device, queue, url, recv).await;
// 		Some(texture)
// 	}
// }

pub struct ImageTextureDesc<'a> {
	pub url: &'a Atom,
	pub device: &'a RenderDevice,
	pub queue: &'a RenderQueue,
}

#[async_trait]
impl<'a, G: Garbageer<Self>> AsyncLoader<'a, Self, ImageTextureDesc<'a>, G> for TextureRes  {
	async fn async_load(desc: ImageTextureDesc<'a>, result: LoadResult<'a, Self, G>) -> std::io::Result<Handle<Self>> {
		match result {
			LoadResult::Ok(r) => Ok(r),
			LoadResult::Wait(f) => f.await,
			LoadResult::Receiver(recv) => {
				let image = pi_hal::image::from_path_or_url(desc.url.as_str()).await;
				// let image = match image {
				// 	Ok(r) => r,
				// 	Err(_e) =>  {
				// 		log::error!("load image fail: {:?}", desc.url.as_str());
				// 		return Err(std::io::Error::new(ErrorKind::NotFound, ""));
				// 	},
				// };

				let texture = create_texture_from_image(&image, &desc.device, &desc.queue, &desc.url, recv).await;
				Ok(texture)
			}
		}
	}
}

pub async fn create_texture_from_image<G: Garbageer<TextureRes>>(
	image: &DynamicImage, 
	device: &RenderDevice, 
	queue: &RenderQueue,
	key: &Atom,
	recv: Receiver<TextureRes, G>
) -> Handle<TextureRes> {
	let buffer_temp;
	let buffer_temp1;
	let (width, height, buffer, ty, pre_pixel_size, is_opacity) = match image {
		DynamicImage::ImageLuma8(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::R8Unorm, 1, true),
		DynamicImage::ImageRgb8(r) => {
			buffer_temp =  image.to_rgba8();
			(r.width(), r.height(), buffer_temp.as_raw(), wgpu::TextureFormat::Rgba8Unorm, 4, true)
		},
		DynamicImage::ImageRgba8(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::Rgba8Unorm, 4, false),
		DynamicImage::ImageBgr8(r) => {
			buffer_temp1 =  image.to_bgra8();
			(r.width(), r.height(), buffer_temp1.as_raw(), wgpu::TextureFormat::Bgra8Unorm, 4, true)
		},
		DynamicImage::ImageBgra8(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::Bgra8Unorm, 4, false),

		_ => panic!("不支持的图片格式"),

		// DynamicImage::ImageLumaA8(image) => panic!("不支持的图片格式: DynamicImage::ImageLumaA8"),
		// DynamicImage::ImageLuma16(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::Bgra8Unorm),
		// DynamicImage::ImageLumaA16(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::Bgra8Unorm),

		// DynamicImage::ImageRgb16(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::Bgra8Unorm),
		// DynamicImage::ImageRgba16(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::Bgra8Unorm),
	};
	let texture_extent = wgpu::Extent3d {
		width,
		height,
		depth_or_array_layers: 1,
	};
	let byte_size = buffer.len();

	let texture = (**device).create_texture(&wgpu::TextureDescriptor {
		label: Some("first depth buffer"),
		size: texture_extent,
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: ty,
		usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
	});
	let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
	queue.write_texture(
		texture.as_image_copy(),
		buffer,
		wgpu::ImageDataLayout {
			offset: 0,
			bytes_per_row: Some(std::num::NonZeroU32::new(width * pre_pixel_size).unwrap()),
			rows_per_image: None,
		},
		texture_extent,
	);

	recv.receive(key.get_hash() as u64, Ok(TextureRes::new(width, height, byte_size, texture_view, is_opacity))).await.unwrap()
}
