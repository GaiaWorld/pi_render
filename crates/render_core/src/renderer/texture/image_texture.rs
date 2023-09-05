
use pi_assets::{asset::{Asset, Garbageer, Handle, Size}, mgr::{LoadResult, Receiver}};
use pi_atom::Atom;
use pi_futures::BoxFuture;
use pi_hal::{
	image::DynamicImage,
	loader::AsyncLoader,
};
use pi_hash::XHashMap;

use crate::{rhi::{device::RenderDevice, RenderQueue}, asset::TAssetKeyU64};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum KeyImageTexture {
	File(Atom, bool),
	Data(Atom, bool),
}
impl KeyImageTexture {
    pub fn as_str(&self) -> &str {
		match self {
			KeyImageTexture::File(val, _) => val.as_str(),
			KeyImageTexture::Data(val, _) => val.as_str(),
		}
	}
    pub fn as_srgb(&self) -> bool {
		match self {
			KeyImageTexture::File(_val, srgb) => *srgb,
			KeyImageTexture::Data(_val, srgb) => *srgb,
		}
	}
}
impl TAssetKeyU64 for KeyImageTexture {}
impl std::ops::Deref for KeyImageTexture {
    type Target = str;

    fn deref(&self) -> &Self::Target {
		match self {
			KeyImageTexture::File(val, _) => val.as_str(),
			KeyImageTexture::Data(val, _) => val.as_str(),
		}
    }
}


#[derive(Debug)]
pub struct ImageTexture {
	pub(crate) width: u32,
	pub(crate) height: u32,
	pub(crate) texture: wgpu::Texture,
	#[allow(dead_code)]
	pub(crate) is_opacity: bool,
	pub(crate) size: usize,
	pub(crate) format: wgpu::TextureFormat,
	pub(crate) dimension: wgpu::TextureViewDimension,
}

impl ImageTexture {
	pub fn new(width: u32, height: u32, size: usize, texture: wgpu::Texture, format: wgpu::TextureFormat, dimension: wgpu::TextureViewDimension, is_opacity: bool) -> Self {
		Self { width, height, size, texture, is_opacity, format, dimension }
	}
	pub fn create_data_texture(device: &RenderDevice, queue: &RenderQueue, key: &KeyImageTexture, data: &[u8], width: u32, height: u32, format: wgpu::TextureFormat, dimension: wgpu::TextureViewDimension, pre_pixel_size: u32, is_opacity: bool) -> Self {
		let texture_extent = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};

		let texture = (**device).create_texture(&wgpu::TextureDescriptor {
			label: Some(key.as_str()),
			size: texture_extent,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: format,
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
			view_formats: &[],
		});

		queue.write_texture(
			texture.as_image_copy(),
			data,
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(width * pre_pixel_size),
				rows_per_image: None,
			},
			texture_extent,
		);

		let size = data.len();

		Self::new(width, height, size, texture, format, dimension, is_opacity)
	}

	pub fn update(&self, queue: &RenderQueue, data: &[u8], xoffset: u32, yoffset: u32, width: u32, height: u32) {
		let offset = (yoffset * self.width + xoffset) as u64;
		let temp = self.texture.as_image_copy();
		queue.write_texture(temp, data, wgpu::ImageDataLayout { offset, bytes_per_row: None, rows_per_image: None  }, wgpu::Extent3d { width, height, depth_or_array_layers: 1 });
	}

	pub fn width(&self) -> u32 {
		self.width
	}
	pub fn height(&self) -> u32 {
		self.height
	}
}

impl Asset for ImageTexture {
	type Key = KeyImageTexture;
    // const TYPE: &'static str = "ImageTexture";
}
impl Size for ImageTexture {
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

pub enum ErrorImageTexture {
	LoadFail,
	CacheError,
}

pub struct ImageTextureErrorMap(pub XHashMap<KeyImageTexture, ErrorImageTexture>);

#[derive(Clone)]
pub struct ImageTexture2DDesc {
	pub url: KeyImageTexture,
	pub device: RenderDevice,
	pub queue: RenderQueue,
}

impl<'a, G: Garbageer<Self>> AsyncLoader<'a, Self, ImageTexture2DDesc, G> for ImageTexture  {
	fn async_load(desc: ImageTexture2DDesc, result: LoadResult<'a, Self, G>) -> BoxFuture<'a, std::io::Result<Handle<Self>>> {
		Box::pin(async move { 
			match result {
				LoadResult::Ok(r) => Ok(r),
				LoadResult::Wait(f) => f.await,
				LoadResult::Receiver(recv) => {
					let image = pi_hal::image::load_from_url(&Atom::from(desc.url.as_str()) ).await;
					let image = match image {
						Ok(r) => r,
						Err(_e) =>  {
							log::debug!("load image fail: {:?}", desc.url.as_str());
							return Err(std::io::Error::new(std::io::ErrorKind::NotFound, ""));
						},
					};

					let texture = create_texture_from_image(&image, desc, recv).await;
					Ok(texture)
				}
			}
		})
	}
}

pub async fn create_texture_from_image<G: Garbageer<ImageTexture>>(
	image: &DynamicImage, 
	desc: ImageTexture2DDesc,
	recv: Receiver<ImageTexture, G>
) -> Handle<ImageTexture> {
	let buffer_temp;
	// let buffer_temp1;
	let (width, height, buffer, ty, pre_pixel_size, is_opacity) = match image {
		DynamicImage::ImageLuma8(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::R8Unorm, 1, true),
		DynamicImage::ImageRgb8(r) => {
			let format = if desc.url.as_srgb() { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm };
			buffer_temp =  image.to_rgba8();
			(r.width(), r.height(), buffer_temp.as_raw(), format, 4, false)
		},
		DynamicImage::ImageRgba8(image) => {
			let format = if desc.url.as_srgb() { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm };
			(image.width(), image.height(), image.as_raw(), format, 4, true)
		},
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

	let texture = (**desc.device).create_texture(&wgpu::TextureDescriptor {
		label: Some(desc.url.as_str()),
		size: texture_extent,
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: ty,
		usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
		view_formats: &[],
	});

	desc.queue.write_texture(
		texture.as_image_copy(),
		buffer,
		wgpu::ImageDataLayout {
			offset: 0,
			bytes_per_row: Some(width * pre_pixel_size),
			rows_per_image: None,
		},
		texture_extent,
	);

	recv.receive(desc.url, Ok(ImageTexture::new(width, height, (width * height * pre_pixel_size) as usize, texture, ty, wgpu::TextureViewDimension::D2, is_opacity))).await.unwrap()
}
