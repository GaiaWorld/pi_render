use derive_deref_rs::Deref;
use pi_assets::{asset::{Asset, Garbageer, Handle}, mgr::{LoadResult, Receiver}};
use pi_atom::Atom;
use pi_futures::BoxFuture;
use pi_hal::{
	image::DynamicImage,
	loader::AsyncLoader,
};

use crate::rhi::{device::RenderDevice, RenderQueue};

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

pub type KeyImageTexture2D = Atom;

pub struct ImageTexture2D {
	pub width: u32,
	pub height: u32,
	pub texture: wgpu::Texture,
	pub is_opacity: bool,
	size: usize,
}

impl ImageTexture2D {
	pub fn new(width: u32, height: u32, size: usize, texture: wgpu::Texture, is_opacity: bool) -> Self {
		Self { width, height, size, texture, is_opacity }
	}
}

impl Asset for ImageTexture2D{
	type Key = KeyImageTexture2D;

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

pub struct ImageTexture2DDesc<'a> {
	pub url: &'a KeyImageTexture2D,
	pub device: &'a RenderDevice,
	pub queue: &'a RenderQueue,
}

impl<'a, G: Garbageer<Self>> AsyncLoader<'a, Self, ImageTexture2DDesc<'a>, G> for ImageTexture2D  {
	fn async_load(desc: ImageTexture2DDesc<'a>, result: LoadResult<'a, Self, G>) -> BoxFuture<'a, std::io::Result<Handle<Self>>> {
		Box::pin(async move { 
			match result {
				LoadResult::Ok(r) => Ok(r),
				LoadResult::Wait(f) => f.await,
				LoadResult::Receiver(recv) => {
					let image = pi_hal::image::load_from_url(&desc.url).await;
					let image = match image {
						Ok(r) => r,
						Err(_e) =>  {
							log::error!("load image fail: {:?}", desc.url.as_str());
							return Err(std::io::Error::new(std::io::ErrorKind::NotFound, ""));
						},
					};

					let texture = create_texture_from_image(&image, &desc.device, &desc.queue, &desc.url, recv).await;
					Ok(texture)
				}
			}
		})
	}
}

pub async fn create_texture_from_image<G: Garbageer<ImageTexture2D>>(
	image: &DynamicImage, 
	device: &RenderDevice, 
	queue: &RenderQueue,
	key: &Atom,
	recv: Receiver<ImageTexture2D, G>
) -> Handle<ImageTexture2D> {
	let buffer_temp;
	// let buffer_temp1;
	let (width, height, buffer, ty, pre_pixel_size, is_opacity) = match image {
		DynamicImage::ImageLuma8(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::R8Unorm, 1, true),
		DynamicImage::ImageRgb8(r) => {
			buffer_temp =  image.to_rgba8();
			(r.width(), r.height(), buffer_temp.as_raw(), wgpu::TextureFormat::Rgba8Unorm, 4, true)
		},
		DynamicImage::ImageRgba8(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::Rgba8Unorm, 4, false),
		// DynamicImage::ImageBgr8(r) => {
		// 	buffer_temp1 =  image.to_bgra8();
		// 	(r.width(), r.height(), buffer_temp1.as_raw(), wgpu::TextureFormat::Bgra8Unorm, 4, true)
		// },
		// DynamicImage::ImageBgra8(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::Bgra8Unorm, 4, false),

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

	recv.receive(key.clone(), Ok(ImageTexture2D::new(width, height, byte_size, texture, is_opacity))).await.unwrap()
}
