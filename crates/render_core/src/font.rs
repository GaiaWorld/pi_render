use std::hash::{Hash, Hasher};

// use crossbeam::queue::SegQueue;
// use pi_hal::font::sdf2_table::TexInfo;
use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_async_rt::prelude::AsyncValue;
use pi_atom::Atom;
use pi_hal::font::sdf2_table::SdfResult;
// use pi_hal::font::svg::SvgTable;
use pi_hash::DefaultHasher;
use pi_share::{Share, ShareMutex};
use wgpu::{Texture, ImageCopyTexture, TextureAspect, ImageDataLayout, Extent3d, Origin3d};
pub use pi_hal::font::font::{FontMgr, FontType, FontId};
pub use pi_hal::font::font::{ Font, Size, GlyphId, FontFamilyId, Glyph};
pub use pi_hal::font::text_split::*;

use crate::rhi::asset::AssetWithId;
use crate::rhi::{asset::TextureRes, device::RenderDevice, RenderQueue};

pub struct FontSheet {
	font_mgr: FontMgr,
	texture_version: Share<ShareMutex<usize>>,
	texture_view: Option<Handle<AssetWithId<TextureRes>>>,
	texture: Option<Share<Texture>>,

	pub sdf_texture_version: Share<ShareMutex<usize>>,
	pub sdf_texture_view: Option<Handle<AssetWithId<TextureRes>>>,
	pub sdf_texture: Option<Share<Texture>>,

	// pub sdf2_texture_version: Share<ShareMutex<usize>>,
	// pub sdf2_index_texture_view: Option<Handle<AssetWithId<TextureRes>>>,
	// pub sdf2_index_texture: Option<Share<Texture>>,
	// pub sdf2_data_texture_view: Option<Handle<AssetWithId<TextureRes>>>,
	// pub sdf2_data_texture: Option<Share<Texture>>,
	// pub sdf2_shadow_texture_view: Option<Handle<AssetWithId<TextureRes>>>,
	// pub sdf2_shadow_texture: Option<Share<Texture>>,
	// pub sdf2_await: Share<SegQueue< Arc<ShareMutex<(usize, Vec<(DefaultKey, TexInfo, Vec<u8>, Vec<u8>)>)>>>>,

	queue: RenderQueue,
	device: RenderDevice,

	texture_asset_mgr: Share<AssetMgr<AssetWithId<TextureRes>>>,
	alloter: Share<pi_key_alloter::KeyAlloter>,
}

pub fn calc_hash2<T: Hash>(v: &T, cur: u64) -> u64 {
    let mut hasher = DefaultHasher::default();
    cur.hash(&mut hasher);
    v.hash(&mut hasher);
    hasher.finish()
}

unsafe impl Send for FontSheet {}
unsafe impl Sync for FontSheet {}

impl FontSheet {
	pub fn new(
		device: &RenderDevice,
		texture_asset_mgr: &Share<AssetMgr<AssetWithId<TextureRes>>>,
		alloter: Share<pi_key_alloter::KeyAlloter>,
		queue: &RenderQueue,
		max_texture_dimension_2d: u32,
		font_type: FontType,
	) -> FontSheet {
		let texture_max = max_texture_dimension_2d.min(4096);
		let width = 4096.min(texture_max);
		let height = texture_max;

		// 宽高可能可变，TODO
		let mut r = Self { 
			font_mgr: FontMgr::new(width as usize, height as usize, font_type, device.0.clone(), queue.clone()),
			texture_view: None, 
			texture: None,
			texture_version: Share::new(ShareMutex::new(0)),

			sdf_texture_view: None, 
			sdf_texture: None,
			sdf_texture_version: Share::new(ShareMutex::new(0)),

			// sdf2_index_texture_view: None, 
			// sdf2_index_texture: None,
			// sdf2_data_texture_view: None, 
			// sdf2_data_texture: None,
			// sdf2_shadow_texture_view: None,
			// sdf2_shadow_texture: None,
			// sdf2_await: Share::new(SegQueue::default()),
			// sdf2_texture_version: Share::new(ShareMutex::new(0)),

			queue: queue.clone(),
			device: device.clone(),
			texture_asset_mgr: texture_asset_mgr.clone(),
			alloter,
 
		};
		match font_type {
			FontType::Bitmap => r.init_texture(),
			FontType::Sdf1 => r.init_sdf_texture(),
			FontType::Sdf2 => {
				r.init_sdf_texture()
				// r.init_sdf_texture()
			},
		};
		let rect = pi_hal::svg::Rect::new(0.0, 0.0, 32.0, 32.0);
		let hash = calc_hash2(&"rect sdf", 0);
		log::debug!("预处理 hash： {}", hash);
        r.font_mgr.table.sdf2_table.add_shape(hash, rect.get_svg_info(), 32, 128, 2);
		let texture = SdfResult::default();
		r.font_mgr.table.sdf2_table.advance_computer_svg(texture.clone());
		let mut result = texture.0.lock().unwrap();
		let sdf_texture = r.sdf_texture.clone();
		let version = r.texture_version.clone();
		let queue = r.queue.clone();
		r.font_mgr.table.sdf2_table.update_svg(move |block, image| {
			// let (texture, pixle_size) = if image.width * image.height * 2 == image.buffer.len() {
			// 	// index
			// 	match &sdf_texture {
			// 		Some(r) => (r, 2),
			// 		None => return,
			// 	}
			// } else {
			// 	// data
			// 	match &sdf_texture {
			// 		Some(r) => (r, 4),
			// 		None => return,
			// 	}
			// };
			let (texture, pixle_size) =	match &sdf_texture {
				Some(r) => (r, 1),
				None => return,
			};
		
			// log::warn!("draw sdf2=-=============={}, {:?}, {:?}, {:?}, {:?}, {:?}", image.buffer.len(), block.x, block.y, &image.width, image.height, pixle_size);
			
			queue.write_texture(
				ImageCopyTexture {
					texture: &texture,
					mip_level: 0,
					origin: Origin3d {
						x: block.x as u32,
						y: block.y as u32,
						z: 0
					},
					aspect: TextureAspect::All
				}, 
				image.buffer.as_slice(),
				ImageDataLayout {
					offset: 0,
					bytes_per_row: if image.width == 0 { None }else { Some(image.width as u32 * pixle_size) }, // 32 * 4
					rows_per_image: None,
				},
				Extent3d {
					width: image.width as u32,
					height: image.height as u32,
					depth_or_array_layers: 1,
				});
			let mut v =  version.lock().unwrap();
			*v = *v + 1;
		}, &mut result.svg_result);
		log::debug!("预处理 hash222： {}", hash);
		r
	}

	pub fn font_mgr(&self) -> &FontMgr {
		&self.font_mgr
	}

	pub fn font_mgr_mut(&mut self) -> &mut FontMgr {
		&mut self.font_mgr
	}

	/// 纹理
	pub fn texture(&self) -> &Option<Share<Texture>> {
		&self.texture
	}

	/// 纹理
	pub fn texture_view(&self) -> &Option<Handle<AssetWithId<TextureRes>>> {
		&self.texture_view
	}

	/// 取到纹理版本
	pub fn texture_version(&self) -> usize {
		*self.texture_version.lock().unwrap()
	}

	/// 纹理宽高
	pub fn texture_size(&self) -> Size<usize> {
		self.font_mgr.size()
	}

	/// 纹理
	pub fn sdf_texture(&self) -> &Option<Share<Texture>> {
		&self.sdf_texture
	}

	/// 纹理
	pub fn sdf_texture_view(&self) -> &Option<Handle<AssetWithId<TextureRes>>> {
		&self.sdf_texture_view
	}

	/// 取到纹理版本
	pub fn sdf_texture_version(&self) -> usize {
		*self.sdf_texture_version.lock().unwrap()
	}

	/// 字体id
	pub fn font_id(&mut self, f: Font) -> FontId {
		self.font_mgr.font_id(f)
	}

	pub fn font_height(&self, f: FontId, font_size: usize) -> f32 {
		self.font_mgr.font_height(f, font_size)
	}

	/// 字形id, 纹理中没有更多空间容纳时，返回None
	pub fn glyph_id(&mut self, f: FontId, char: char) -> Option<GlyphId> {
		self.font_mgr.glyph_id(f, char)
	}

	/// 测量宽度
	pub fn measure_width(&mut self, f: FontId, char: char) -> f32 {
		self.font_mgr.measure_width(f, char)
	}

	// /// 取到字形信息
	// pub fn glyph(&self, id: GlyphId) -> &Glyph {
	// 	self.font_mgr.glyph(id)
	// }

	/// 清理字形
	pub fn clear(&mut self) {
		self.font_mgr.clear()
	}

	pub fn draw_count(&self) -> usize  {
		self.font_mgr.table.sdf2_table.draw_count(&self.font_mgr.sheet.fonts)
	}

	// 绘制等待列表
	pub fn draw_await(&mut self, result: SdfResult,  index: usize, count: usize) -> AsyncValue<()> {
		let font_type = self.font_mgr.font_type();
		
		match font_type {
			FontType::Bitmap => todo!(),
			FontType::Sdf1 => todo!(),
			FontType::Sdf2 => self.font_mgr.table.sdf2_table.draw_await( self.sdf_texture.as_ref().unwrap().clone(), &mut self.font_mgr.sheet, index, result, count)
		}
	}

	pub fn update_sdf2(&mut self, result: SdfResult) {
		let queue = self.queue.clone();
		let version = self.texture_version.clone();
		// let sdf_texture_version = self.sdf_texture_version.clone();
		let font_type = self.font_mgr.font_type();

		let sdf_texture = self.sdf_texture.clone();
		// let sdf2_data_texture = self.sdf2_data_texture.clone();
		// let sdf2_shadow_texture = self.sdf2_shadow_texture.clone();

		// let queue1 = queue.clone();
		match font_type {
			FontType::Bitmap => todo!(),
			FontType::Sdf1 => todo!(),
			FontType::Sdf2 => {
				self.font_mgr.table.sdf2_table.update(move |block, image| {
					// let (texture, pixle_size) = if image.width * image.height * 2 == image.buffer.len() {
					// 	// index
					// 	match &sdf_texture {
					// 		Some(r) => (r, 2),
					// 		None => return,
					// 	}
					// } else {
					// 	// data
					// 	match &sdf_texture {
					// 		Some(r) => (r, 4),
					// 		None => return,
					// 	}
					// };
					let (texture, pixle_size) =	match &sdf_texture {
						Some(r) => (r, 1),
						None => return,
					};
		
					// log::warn!("draw sdf2=-=============={}, {:?}, {:?}, {:?}, {:?}, {:?}", image.buffer.len(), block.x, block.y, &image.width, image.height, pixle_size);
					
					queue.write_texture(
						ImageCopyTexture {
							texture: &texture,
							mip_level: 0,
							origin: Origin3d {
								x: block.x as u32,
								y: block.y as u32,
								z: 0
							},
							aspect: TextureAspect::All
						}, 
						image.buffer.as_slice(),
						ImageDataLayout {
							offset: 0,
							bytes_per_row: if image.width == 0 { None }else { Some(image.width as u32 * pixle_size) }, // 32 * 4
							rows_per_image: None,
						},
						Extent3d {
							width: image.width as u32,
							height: image.height as u32,
							depth_or_array_layers: 1,
						});
					let mut v =  version.lock().unwrap();
					*v = *v + 1;
				}, result)
			},
		}
	}

	// // 绘制等待列表
	// pub fn draw_svg_await(&mut self, result: Arc<ShareMutex<(usize, Vec<(u64, SdfInfo2)>)>>) -> AsyncValue<()> {
	// 	let font_type = self.font_mgr.font_type();
		
	// 	match font_type {
	// 		FontType::Bitmap => todo!(),
	// 		FontType::Sdf1 => todo!(),
	// 		FontType::Sdf2 => self.font_mgr.table.sdf2_table.draw_svg_await(result)
	// 	}
	// }

	// pub fn update_svg_sdf2(&mut self, result: Arc<ShareMutex<(usize, Vec<(u64, SdfInfo2)>)>>) {
	// 	let queue = self.queue.clone();
	// 	let version = self.texture_version.clone();
	// 	// let sdf_texture_version = self.sdf_texture_version.clone();
	// 	let font_type = self.font_mgr.font_type();

	// 	let sdf_texture = self.sdf_texture.clone();
	// 	// let sdf2_data_texture = self.sdf2_data_texture.clone();
	// 	// let sdf2_shadow_texture = self.sdf2_shadow_texture.clone();
	// 	// let queue1 = queue.clone();
	// 	match font_type {
	// 		FontType::Bitmap => todo!(),
	// 		FontType::Sdf1 => todo!(),
	// 		FontType::Sdf2 => {
	// 			self.font_mgr.table.sdf2_table.update_svg(move |block, image| {
	// 				// let (texture, pixle_size) = if image.width * image.height * 2 == image.buffer.len() {
	// 				// 	// index
	// 				// 	match &sdf_texture {
	// 				// 		Some(r) => (r, 2),
	// 				// 		None => return,
	// 				// 	}
	// 				// } else {
	// 				// 	// data
	// 				// 	match &sdf_texture {
	// 				// 		Some(r) => (r, 4),
	// 				// 		None => return,
	// 				// 	}
	// 				// };
	// 				let (texture, pixle_size) =	match &sdf_texture {
	// 					Some(r) => (r, 1),
	// 					None => return,
	// 				};
		
	// 				log::debug!("draw update_svg_sdf2=-=============={}, {:?}, {:?}, {:?}, {:?}, {:?}", block.x, block.y, &image.width, image.height, pixle_size, image.buffer.as_slice());
					
	// 				queue.write_texture(
	// 					ImageCopyTexture {
	// 						texture: &texture,
	// 						mip_level: 0,
	// 						origin: Origin3d {
	// 							x: block.x as u32,
	// 							y: block.y as u32,
	// 							z: 0
	// 						},
	// 						aspect: TextureAspect::All
	// 					}, 
	// 					image.buffer.as_slice(),
	// 					ImageDataLayout {
	// 						offset: 0,
	// 						bytes_per_row: if image.width == 0 { None }else { Some(image.width as u32 * pixle_size) }, // 32 * 1
	// 						rows_per_image: None,
	// 					},
	// 					Extent3d {
	// 						width: image.width as u32,
	// 						height: image.height as u32,
	// 						depth_or_array_layers: 1,
	// 					});
	// 				let mut v =  version.lock().unwrap();
	// 				*v = *v + 1;
	// 			},result)
	// 		},
	// 	}
	// }

	/// 绘制文字
	pub fn update(&mut self) {
		// let texture = self.texture.clone();
		// let sdf_texture = self.sdf_texture.clone();
		// let queue = self.queue.clone();
		// let version = self.texture_version.clone();
		// let sdf_texture_version = self.sdf_texture_version.clone();
		let font_type = self.font_mgr.font_type();

		match font_type {
			FontType::Bitmap => {
				// self.font_mgr.table.bitmap_table.draw(&mut self.font_mgr.sheet.fonts, move |block, image| {
				// 	let (texture, pixle_size) = if image.width * image.height == image.buffer.len() {
				// 		// sdf
				// 		match &sdf_texture {
				// 			Some(r) => (r, 1),
				// 			None => return,
				// 		}
				// 	} else {
				// 		match &texture {
				// 			Some(r) => (r, 4),
				// 			None => return,
				// 		}
				// 	};
		
				// 	// log::warn!("draw=-=============={}, {:?}, {:?}, {:?}, {:?}", image.buffer.len(), block.x, block.y, &image.width, image.height);
					
				// 	queue.write_texture(
				// 		ImageCopyTexture {
				// 			texture: &texture,
				// 			mip_level: 0,
				// 			origin: Origin3d {
				// 				x: block.x as u32,
				// 				y: block.y as u32,
				// 				z: 0
				// 			},
				// 			aspect: TextureAspect::All
				// 		}, 
				// 		image.buffer.as_slice(),
				// 		ImageDataLayout {
				// 			offset: 0,
				// 			bytes_per_row: if image.width == 0 { None }else { Some(image.width as u32 * pixle_size) }, // 32 * 4
				// 			rows_per_image: None,
				// 		},
				// 		Extent3d {
				// 			width: image.width as u32,
				// 			height: image.height as u32,
				// 			depth_or_array_layers: 1,
				// 		});
				// 	let mut v =  version.lock();
				// 	*v = *v + 1;
				// })
			},
			FontType::Sdf1 => todo!(),
			FontType::Sdf2 => todo!(),
		}
		
	}

	// fn draw(&mut self) {
	// }
	fn init_texture(&mut self) {
		let size = self.font_mgr.size();
		let texture = (*self.device).create_texture(&wgpu::TextureDescriptor {
			label: Some("first depth buffer"),
			size: wgpu::Extent3d {
				width: size.width as u32,
				height: size.height as u32,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8Unorm,
			view_formats: &[],
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		});
		let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		// let key = calc_hash(&"text texture view");
		let key = Atom::from("_$text").str_hash() as u64;
		let texture_view = if let Ok(r) = self.texture_asset_mgr.insert(key, AssetWithId::new(TextureRes::new(size.width as u32, size.height as u32, (size.width * size.height * 4) as usize, texture_view, false, wgpu::TextureFormat::Rgba8Unorm), (size.width * size.height * 4) as usize, self.alloter.clone())) {
			r
		} else {
			panic!("insert asset fail");
		};

		self.texture = Some(Share::new(texture));
		self.texture_view = Some(texture_view);
	}

	fn init_sdf_texture(&mut self) {
		let size = self.font_mgr.size();
		let texture = (*self.device).create_texture(&wgpu::TextureDescriptor {
			label: Some("sdf texture"),
			size: wgpu::Extent3d {
				width: size.width as u32,
				height: size.height as u32,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::R8Unorm,
			view_formats: &[wgpu::TextureFormat::R8Unorm],
			usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
		});
		let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let key = Atom::from("_$text_sdf").str_hash() as u64;
		let texture_view = if let Ok(r) = self.texture_asset_mgr.insert(key, AssetWithId::new(TextureRes::new(size.width as u32, size.height as u32, (size.width * size.height) as usize, texture_view, false, wgpu::TextureFormat::R8Unorm), (size.width * size.height) as usize, self.alloter.clone())) {
			r
		} else {
			panic!("insert asset fail");
		};

		log::debug!("sdf texture size: {:?}", size);

		self.sdf_texture = Some(Share::new(texture));
		self.sdf_texture_view = Some(texture_view);
	}

	// fn init_sdf2_texture(&mut self) {
	// 	let index_size = self.font_mgr.table.sdf2_table.index_packer_size();
	// 	let data_size = self.font_mgr.table.sdf2_table.data_packer_size();

	// 	let index_texture = (*self.device).create_texture(&wgpu::TextureDescriptor {
	// 		label: Some("sdf2 index texture"),
	// 		size: wgpu::Extent3d {
	// 			width: index_size.width as u32,
	// 			height: index_size.height as u32,
	// 			depth_or_array_layers: 1,
	// 		},
	// 		mip_level_count: 1,
	// 		sample_count: 1,
	// 		dimension: wgpu::TextureDimension::D2,
	// 		format: wgpu::TextureFormat::Rg8Unorm,
	// 		view_formats: &[],
	// 		usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
	// 	});
	// 	let data_texture = (*self.device).create_texture(&wgpu::TextureDescriptor {
	// 		label: Some("sdf2 index texture"),
	// 		size: wgpu::Extent3d {
	// 			width: data_size.width as u32,
	// 			height: data_size.height as u32,
	// 			depth_or_array_layers: 1,
	// 		},
	// 		mip_level_count: 1,
	// 		sample_count: 1,
	// 		dimension: wgpu::TextureDimension::D2,
	// 		format: wgpu::TextureFormat::Rgba8Unorm,
	// 		view_formats: &[],
	// 		usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
	// 	});
	// 	let shadow_texture = (*self.device).create_texture(&wgpu::TextureDescriptor {
	// 		label: Some("sdf2 shadow texture"),
	// 		size: wgpu::Extent3d {
	// 			width: index_size.width as u32,
	// 			height: index_size.height as u32,
	// 			depth_or_array_layers: 1,
	// 		},
	// 		mip_level_count: 4,
	// 		sample_count: 1,
	// 		dimension: wgpu::TextureDimension::D2,
	// 		format: wgpu::TextureFormat::R8Unorm,
	// 		usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
	// 		view_formats: &[],
	// 	});

	// 	let index_texture_view = index_texture.create_view(&wgpu::TextureViewDescriptor::default());
	// 	let data_texture_view = data_texture.create_view(&wgpu::TextureViewDescriptor::default());
	// 	let shadow_texture_view = shadow_texture.create_view(&wgpu::TextureViewDescriptor::default());
	// 	// let key = calc_hash(&"text texture view");
	// 	let index_key = Atom::from("_$text_index").str_hash() as u64;
	// 	let data_key = Atom::from("_$text_data").str_hash() as u64;
	// 	let shadow_key = Atom::from("_$shadow_data").str_hash() as u64;
	// 	let index_texture_view = if let Ok(r) = self.texture_asset_mgr.insert(index_key, AssetWithId::new(TextureRes::new(index_size.width as u32, index_size.height as u32, (index_size.width * index_size.height * 2) as usize, index_texture_view, false, wgpu::TextureFormat::Rg8Unorm), (index_size.width * index_size.height * 2) as usize, self.alloter.clone())) {
	// 		r
	// 	} else {
	// 		panic!("insert asset fail");
	// 	};
	// 	let data_texture_view = if let Ok(r) = self.texture_asset_mgr.insert(data_key, AssetWithId::new(TextureRes::new(data_size.width as u32, data_size.height as u32, (data_size.width * data_size.height * 4) as usize, data_texture_view, false, wgpu::TextureFormat::Rgba8Unorm), (data_size.width * data_size.height * 4) as usize, self.alloter.clone())) {
	// 		r
	// 	} else {
	// 		panic!("insert asset fail");
	// 	};
	// 	let shadow_texture_view = if let Ok(r) = self.texture_asset_mgr.insert(shadow_key, AssetWithId::new(TextureRes::new(index_size.width as u32, index_size.height as u32, (index_size.width * index_size.height * 1) as usize, shadow_texture_view, false, wgpu::TextureFormat::R8Unorm), (index_size.width * index_size.height * 1) as usize, self.alloter.clone())) {
	// 		r
	// 	} else {
	// 		panic!("insert asset fail");
	// 	};

	// 	self.sdf2_index_texture = Some(Share::new(index_texture));
	// 	self.sdf2_index_texture_view = Some(index_texture_view);

	// 	self.sdf2_data_texture = Some(Share::new(data_texture));
	// 	self.sdf2_data_texture_view = Some(data_texture_view);

	// 	self.sdf2_shadow_texture = Some(Share::new(shadow_texture));
	// 	self.sdf2_shadow_texture_view = Some(shadow_texture_view);
	// }
}

pub fn calc_hash<T: Hash>(v: &T)-> u64 {
	let mut hasher = DefaultHasher::default();
	v.hash(&mut hasher);
	hasher.finish()
}