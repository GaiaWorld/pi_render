use derive_deref_rs::Deref;
use pi_assets::{asset::{Asset,  Garbageer, Handle, Size}, mgr::{LoadResult, Receiver}};
use pi_atom::Atom;
use pi_futures::BoxFuture;
use pi_hal::{
	image_texture_load,
	loader::AsyncLoader,
};
use pi_key_alloter::KeyData;
use pi_share::Share;
use wgpu::{TextureView, AstcBlock, AstcChannel, util::{DeviceExt, TextureDataOrder}};

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

#[derive(Debug, Deref)]
pub struct TextureRes {
	pub width: u32,
	pub height: u32,
	#[deref]
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

impl<'a, G: Garbageer<Self>> AsyncLoader<'a, Self, ImageTextureDesc<'a>, G> for TextureRes  {
	fn async_load(desc: ImageTextureDesc<'a>, result: LoadResult<'a, Self, G>) -> BoxFuture<'a, std::io::Result<Handle<Self>>> {
		Box::pin(async move { 
			match result {
				LoadResult::Ok(r) => Ok(r),
				LoadResult::Wait(f) => f.await,
				LoadResult::Receiver(recv) => {
					let r = match image_texture_load::load_from_url(&pi_hal::texture::ImageTextureDesc::new(desc.url.clone()), &desc.device, &desc.queue).await {
						Ok(r) => r,
						Err(_e) =>  {
							log::error!("load texture fail: {:?}", desc.url.as_str());
							return Err(std::io::Error::new(std::io::ErrorKind::NotFound, ""));
						},
					};
					let texture_view = r.texture.create_view(&wgpu::TextureViewDescriptor::default());
					let r = TextureRes::new(r.width, r.height, r.size, texture_view, r.is_opacity, r.format);
					recv.receive(desc.url.str_hash() as u64, Ok(r)).await
				}
			}
		})
	}
}

pub struct TextureAssetDesc<'a> {
	pub alloter: &'a Share<pi_key_alloter::KeyAlloter>,
	pub url: &'a Atom,
	pub device: &'a RenderDevice,
	pub queue: &'a RenderQueue,
}


impl<'a, G: Garbageer<Self>> AsyncLoader<'a, Self, TextureAssetDesc<'a>, G> for AssetWithId<TextureRes>  {
	fn async_load(desc: TextureAssetDesc<'a>, result: LoadResult<'a, Self, G>) -> BoxFuture<'a, std::io::Result<Handle<Self>>> {
		Box::pin(async move { 
			match result {
				LoadResult::Ok(r) => Ok(r),
				LoadResult::Wait(f) => f.await,
				LoadResult::Receiver(recv) => {
					let r = match image_texture_load::load_from_url(&pi_hal::texture::ImageTextureDesc::new(desc.url.clone()), &desc.device, &desc.queue).await {
						Ok(r) => r,
						Err(_e) =>  {
							log::error!("load texture fail: {:?}", desc.url.as_str());
							return Err(std::io::Error::new(std::io::ErrorKind::NotFound, ""));
						},
					};
					
					let texture_view = r.texture.create_view(&wgpu::TextureViewDescriptor::default());
					let texture = TextureRes::new(r.width, r.height, r.size, texture_view, r.is_opacity, r.format);
					recv.receive(desc.url.str_hash() as u64, Ok(AssetWithId::new(texture, r.size, desc.alloter.clone()))).await
				}
			}
		})
	}
}


#[derive(Debug)]
pub struct AssetWithId<T> {
	pub id: KeyData,
	pub value: T,
	pub size: usize,
	alloter: Share<pi_key_alloter::KeyAlloter>,
}

impl<T> AssetWithId<T> {
    pub fn new(value: T, size: usize, alloter: Share<pi_key_alloter::KeyAlloter>) -> Self {
		let r = alloter.alloc(1, 1);
        Self {
            id: r,
            value,
            alloter,
			size,
        }
    }
}

impl<T> std::ops::Deref for AssetWithId<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}


impl<T> Drop for AssetWithId<T> {
    fn drop(&mut self) {
		self.alloter.recycle(self.id);
    }
}

impl<T: 'static> Asset for AssetWithId<T>{
	type Key = u64;
    // const TYPE: &'static str = "TextureRes";
}

impl<T: 'static> Size for AssetWithId<T> {
	fn size(&self) -> usize {
		self.size
	}
}
