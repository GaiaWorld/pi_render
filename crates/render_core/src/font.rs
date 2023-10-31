use std::hash::{Hash, Hasher};

use pi_assets::{asset::Handle, mgr::AssetMgr};
use pi_atom::Atom;
use pi_hash::DefaultHasher;
use pi_share::{Share, ShareMutex};
use wgpu::{Texture, ImageCopyTexture, TextureAspect, ImageDataLayout, Extent3d, Origin3d};
use pi_hal::font::font::FontMgr;
pub use pi_hal::font::font::{ Font, Size, GlyphId, FontId, Glyph};
pub use pi_hal::font::text_split::*;

use crate::rhi::{asset::TextureRes, device::RenderDevice, RenderQueue};

pub struct FontSheet {
	font_mgr: FontMgr,
	texture_version: Share<ShareMutex<usize>>,
	texture_view: Handle<TextureRes>,
	texture: Share<Texture>,
	queue: RenderQueue,
}

impl FontSheet {
	pub fn new(
		device: &RenderDevice,
		texture_asset_mgr: &Share<AssetMgr<TextureRes>>,
		queue: &RenderQueue,
		max_texture_dimension_2d: u32,
	) -> FontSheet {
		let texture_max = max_texture_dimension_2d.min(4096);
		let width = 4096.min(texture_max);
		let height = texture_max;
		let texture = (**device).create_texture(&wgpu::TextureDescriptor {
			label: Some("first depth buffer"),
			size: wgpu::Extent3d {
				width,
				height,
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
		let key = Atom::from("_$text").get_hash() as u64;
		let texture_view = if let Ok(r) = texture_asset_mgr.insert(key, TextureRes::new(width, height, (width * height * 4) as usize, texture_view, false, wgpu::TextureFormat::Rgba8Unorm)) {
			r
		} else {
			panic!("insert asset fail");
		};

		// 宽高可能可变，TODO
		Self { 
			font_mgr: FontMgr::new(width as usize, height as usize),
			texture_view: texture_view, 
			texture: Share::new(texture),
			texture_version: Share::new(ShareMutex::new(0)),
			queue: queue.clone(),
		}
	}

	/// 纹理
	pub fn texture(&self) -> &Share<Texture> {
		&self.texture
	}

	/// 纹理
	pub fn texture_view(&self) -> &Handle<TextureRes> {
		&self.texture_view
	}

	/// 取到纹理版本
	pub fn texture_version(&self) -> usize {
		*self.texture_version.lock()
	}

	/// 纹理宽高
	pub fn texture_size(&self) -> Size<usize> {
		self.font_mgr.size()
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

	/// 取到字形信息
	pub fn glyph(&self, id: GlyphId) -> &Glyph {
		self.font_mgr.glyph(id)
	}

	/// 清理字形
	pub fn clear(&mut self) {
		self.font_mgr.clear()
	}

	/// 绘制文字
	pub fn draw(&mut self) {
		let texture = self.texture.clone();
		let queue = self.queue.clone();
		let version = self.texture_version.clone();
		self.font_mgr.draw(move |block, image| {
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
					bytes_per_row: if image.width == 0 { None }else { Some(image.width as u32 * 4) }, // 32 * 4
					rows_per_image: None,
				},
				Extent3d {
					width: image.width as u32,
					height: image.height as u32,
					depth_or_array_layers: 1,
				});
			let mut v = version.lock();
			*v = *v + 1;
		})
	}

	// fn draw(&mut self) {
	// }
}

pub fn calc_hash<T: Hash>(v: &T)-> u64 {
	let mut hasher = DefaultHasher::default();
	v.hash(&mut hasher);
	hasher.finish()
}