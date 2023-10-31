use derive_deref_rs::Deref;
use pi_assets::{asset::{Asset,  Garbageer, Handle, Size}, mgr::{LoadResult, Receiver}};
use pi_atom::Atom;
use pi_futures::BoxFuture;
use pi_hal::{
	image::DynamicImage,
	loader::AsyncLoader,
};
use wgpu::{TextureView, AstcBlock, AstcChannel, util::DeviceExt};

use super::{device::RenderDevice, RenderQueue, texture::PiRenderDefault};

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
}

impl<T: 'static> Size for RenderRes<T>{
	fn size(&self) -> usize {
		self.size
	}
}

pub fn calc_texture_size(desc: &wgpu::TextureDescriptor) -> usize {
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

#[derive(Debug)]
pub struct TextureRes {
	pub width: u32,
	pub height: u32,
	pub texture_view: TextureView,
	pub is_opacity: bool,
	pub format: wgpu::TextureFormat,
	size: usize,
}

impl TextureRes {
	pub fn new(width: u32, height: u32, size: usize, texture_view: TextureView, is_opacity: bool, format: wgpu::TextureFormat) -> Self {
		Self { width, height, size, texture_view, is_opacity, format }
	}
}


impl Asset for TextureRes{
	type Key = u64;
    // const TYPE: &'static str = "TextureRes";
}

impl Size for TextureRes{
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

const KTX_SUFF: &'static str = ".ktx";

impl<'a, G: Garbageer<Self>> AsyncLoader<'a, Self, ImageTextureDesc<'a>, G> for TextureRes  {
	fn async_load(desc: ImageTextureDesc<'a>, result: LoadResult<'a, Self, G>) -> BoxFuture<'a, std::io::Result<Handle<Self>>> {
		Box::pin(async move { 
			match result {
				LoadResult::Ok(r) => Ok(r),
				LoadResult::Wait(f) => f.await,
				LoadResult::Receiver(recv) => {
					if desc.url.ends_with(KTX_SUFF) {
						// 加载ktx
						let file = pi_hal::file::load_from_url(&desc.url).await;
						let file = match file {
							Ok(r) => r,
							Err(_e) =>  {
								log::error!("load file fail: {:?}", desc.url.as_str());
								return Err(std::io::Error::new(std::io::ErrorKind::NotFound, ""));
							},
						};
						create_texture_from_ktx(file.as_slice(), &desc.device, &desc.queue, &desc.url, recv).await

					} else {
						// 加载普通图片
						let image = pi_hal::image::load_from_url(&desc.url).await;
						let image = match image {
							Ok(r) => r,
							Err(_e) =>  {
								log::error!("load image fail: {:?}", desc.url.as_str());
								return Err(std::io::Error::new(std::io::ErrorKind::NotFound, ""));
							},
						};

						Ok(create_texture_from_image(&image, &desc.device, &desc.queue, &desc.url, recv).await)
					}
				}
			}
		})
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
	// let buffer_temp1;
	let (width, height, buffer, format, pre_pixel_size, is_opacity) = match image {
		DynamicImage::ImageLuma8(image) => (image.width(), image.height(), image.as_raw(), wgpu::TextureFormat::R8Unorm, 1, true),
		DynamicImage::ImageRgb8(r) => {
			buffer_temp =  image.to_rgba8();
			(r.width(), r.height(), buffer_temp.as_raw(), if <wgpu::TextureFormat as PiRenderDefault>::is_srgb() { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm }, 4, true)
		},
		DynamicImage::ImageRgba8(image) => (image.width(), image.height(), image.as_raw(), if <wgpu::TextureFormat as PiRenderDefault>::is_srgb() { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm }, 4, false),
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

	// log::warn!("create_texture==========={:?}, {:?}", key, std::thread::current().id());
	let texture = (**device).create_texture(&wgpu::TextureDescriptor {
		label: Some("image texture"),
		size: texture_extent,
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format,
		usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		view_formats: &[],
	});
	let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

	queue.write_texture(
		texture.as_image_copy(),
		buffer,
		wgpu::ImageDataLayout {
			offset: 0,
			bytes_per_row: Some(width * pre_pixel_size),
			rows_per_image: None,
		},
		texture_extent,
	);

	recv.receive(key.get_hash() as u64, Ok(TextureRes::new(width, height, byte_size, texture_view, is_opacity, format))).await.unwrap()
}


pub async fn create_texture_from_ktx<G: Garbageer<TextureRes>>(
	buffer: &[u8], 
	device: &RenderDevice, 
	queue: &RenderQueue,
	key: &Atom,
	recv: Receiver<TextureRes, G>
) -> std::io::Result<Handle<TextureRes>> {

	use ktx::KtxInfo;

	let decoder = ktx::Decoder::new(buffer)?;
	let format = convert_format(decoder.gl_internal_format());

	let texture_extent = wgpu::Extent3d {
		width: decoder.pixel_width(),
		height: decoder.pixel_height(),
		depth_or_array_layers: 1,
	}.physical_size(format);
	log::warn!("width====={:?}, height==={:?}", texture_extent.width, texture_extent.height);

	// let byte_size = buffer.len();
	let mut textures = decoder.read_textures();
	let data = textures.next().unwrap(); // TODO

	let texture = (**device).create_texture_with_data(queue, &wgpu::TextureDescriptor {
		label: Some("first depth buffer"),
		size: texture_extent,
		mip_level_count: 1, // TODO
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format,
		usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		view_formats: &[],
	}, data.as_slice());
	let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

	Ok(recv.receive(key.get_hash() as u64, Ok(TextureRes::new(texture_extent.width, texture_extent.height, data.len(), texture_view, true/*TODO*/, format))).await.unwrap())
}

fn convert_format(v: u32) -> wgpu::TextureFormat {
	match v {
		// 0x83f0 => wgpu::TextureFormat::Bc1RgbUnorm,// GL_COMPRESSED_RGB_S3TC_DXT1_EXT	0x83f0     GL_COMPRESSED_RGB_S3TC_DXT1_EXT	Bc1RgbUnorm
		0x83f1 => wgpu::TextureFormat::Bc1RgbaUnorm,// GL_COMPRESSED_RGBA_S3TC_DXT1_EXT	0x83f1     GL_COMPRESSED_RGBA_S3TC_DXT1_EXT	Bc1RgbaUnorm
		0x83f2 => wgpu::TextureFormat::Bc2RgbaUnorm,// GL_COMPRESSED_RGBA_S3TC_DXT3_EXT	0x83f2     GL_COMPRESSED_RGBA_S3TC_DXT3_EXT	Bc2RgbaUnorm
		0x83f3 => wgpu::TextureFormat::Bc3RgbaUnorm,// GL_COMPRESSED_RGBA_S3TC_DXT5_EXT	0x83f3     GL_COMPRESSED_RGBA_S3TC_DXT5_EXT	Bc3RgbaUnorm
		0x9274 => wgpu::TextureFormat::Etc2Rgb8Unorm,// GL_COMPRESSED_RGB8_ETC2	0x9274             GL_COMPRESSED_RGB8_ETC2	Etc2Rgb8Unorm
		0x9278 => wgpu::TextureFormat::Etc2Rgba8Unorm,// GL_COMPRESSED_RGBA8_ETC2_EAC	0x9278         GL_COMPRESSED_RGBA8_ETC2_EAC	Etc2Rgba8Unorm

		// 0x8c00 => wgpu::TextureFormat::Bc1RgbaUnorm,// GL_COMPRESSED_RGB_PVRTC_4BPPV1_IMG	0x8c00  GL_COMPRESSED_RGB_PVRTC_4BPPV1_IMG	PvrtcRgb4bppUnorm 
		// 0x8c01 => wgpu::TextureFormat::Bc1RgbaUnorm,// GL_COMPRESSED_RGB_PVRTC_2BPPV1_IMG	0x8c01 GL_COMPRESSED_RGB_PVRTC_2BPPV1_IMG	PvrtcRgb2bppUnorm 
		// 0x8c02 => wgpu::TextureFormat::Bc1RgbaUnorm,// GL_COMPRESSED_RGBA_PVRTC_4BPPV1_IMG	0x8c02 UnormGL_COMPRESSED_RGBA_PVRTC_4BPPV1_IMG	PvrtcRgba4bppUnorm
		// 0x8c03 => wgpu::TextureFormat::Bc1RgbaUnorm,// GL_COMPRESSED_RGBA_PVRTC_2BPPV1_IMG	0x8c03 GL_COMPRESSED_RGBA_PVRTC_2BPPV1_IMG	PvrtcRgba2bppUnorm 

		0x93b0 => wgpu::TextureFormat::Astc { block: AstcBlock::B4x4, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_4x4_KHR	0x93b0     GL_COMPRESSED_RGBA_ASTC_4x4_KHR	Astc4x4Unorm 
		0x93b1 => wgpu::TextureFormat::Astc { block: AstcBlock::B5x4, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_5x4_KHR	0x93b1     GL_COMPRESSED_RGBA_ASTC_5x4_KHR	Astc5x4Unorm 
		0x93b2 => wgpu::TextureFormat::Astc { block: AstcBlock::B5x5, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_5x5_KHR	0x93b2     GL_COMPRESSED_RGBA_ASTC_5x5_KHR	Astc5x5Unorm
		0x93b3 => wgpu::TextureFormat::Astc { block: AstcBlock::B6x5, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_6x5_KHR	0x93b3     GL_COMPRESSED_RGBA_ASTC_6x5_KHR	Astc6x5Unorm 
		0x93b4 => wgpu::TextureFormat::Astc { block: AstcBlock::B6x6, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_6x6_KHR	0x93b4     GL_COMPRESSED_RGBA_ASTC_6x6_KHR	Astc6x6Unorm 
		0x93b5 => wgpu::TextureFormat::Astc { block: AstcBlock::B8x5, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_8x5_KHR	0x93b5     GL_COMPRESSED_RGBA_ASTC_8x5_KHR	Astc8x5Unorm 
		0x93b6 => wgpu::TextureFormat::Astc { block: AstcBlock::B8x6, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_8x6_KHR	0x93b6     GL_COMPRESSED_RGBA_ASTC_8x6_KHR	Astc8x6Unorm 
		0x93b7 => wgpu::TextureFormat::Astc { block: AstcBlock::B8x8, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_8x8_KHR	0x93b7     GL_COMPRESSED_RGBA_ASTC_8x8_KHR	Astc8x8Unorm 
		0x93b8 => wgpu::TextureFormat::Astc { block: AstcBlock::B10x5, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_10x5_KHR	0x93b8     GL_COMPRESSED_RGBA_ASTC_10x5_KHR	Astc10x5Unorm 
		0x93b9 => wgpu::TextureFormat::Astc { block: AstcBlock::B10x6, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_10x6_KHR	0x93b9     GL_COMPRESSED_RGBA_ASTC_10x6_KHR	Astc10x6Unorm 
		0x93ba => wgpu::TextureFormat::Astc { block: AstcBlock::B10x8, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_10x8_KHR	0x93ba GL_COMPRESSED_RGBA_ASTC_10x8_KHR	Astc10x8Unorm  
		0x93bb => wgpu::TextureFormat::Astc { block: AstcBlock::B10x10, channel: AstcChannel::Unorm },//  GL_COMPRESSED_RGBA_ASTC_10x10_KHR	0x93bb     GL_COMPRESSED_RGBA_ASTC_10x10_KHR	Astc10x10Unorm 
		0x93bc => wgpu::TextureFormat::Astc { block: AstcBlock::B12x10, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_12x10_KHR	0x93bc     GL_COMPRESSED_RGBA_ASTC_12x10_KHR	Astc12x10 
		0x93bd => wgpu::TextureFormat::Astc { block: AstcBlock::B12x12, channel: AstcChannel::Unorm },// GL_COMPRESSED_RGBA_ASTC_12x12_KHR	0x93bd     GL_COMPRESSED_RGBA_ASTC_12x12_KHR	Astc12x12Unorm
		_ => panic!("not suport fomat： {}", v),
	}
}
