//! 渲染目标分配器模块
//!
//! 该模块负责管理和分配渲染目标（如颜色附件、深度附件）的纹理资源，
//! 通过Atlas分配算法高效复用纹理内存，支持动态调整纹理尺寸和重用。

use std::{hash::{Hash, Hasher}, collections::hash_map::Entry, intrinsics::transmute, mem::size_of, sync::atomic::AtomicBool};

use derive_deref_rs::Deref;
use guillotiere::{Size, Allocation, Rectangle, Point};
use pi_assets::{asset::{Handle, Droper}, mgr::AssetMgr, homogeneous::HomogeneousMgr};
use pi_null::Null;
use pi_share::{Share, ShareRwLock};
use pi_slotmap::{DefaultKey, SlotMap, SecondaryMap};
use pi_hash::{DefaultHasher, XHashMap};
use pi_atom::Atom;
use smallvec::SmallVec;
use wgpu::{TextureAspect, TextureDimension, TextureFormat, TextureUsages, TextureViewDimension};

use crate::rhi::{asset::{calc_texture_size, AssetWithId, RenderRes, TextureRes}, device::RenderDevice};

lazy_static!{
	pub static ref DEPTH_TEXTURE: Atom = Atom::from("DEPTH_TEXTURE");
}

/// 纹理描述
#[derive(Debug, Hash, Clone, Copy)]
pub struct TextureDescriptor {
	pub mip_level_count: u32,
	pub sample_count: u32,
	pub dimension: TextureDimension,
	pub format: TextureFormat,
	pub usage: TextureUsages,

	pub base_mip_level: u32,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
	pub view_dimension: Option<TextureViewDimension>,
}


/// 渲染目标描述
#[derive(Debug, Hash, Clone)]
pub struct TargetDescriptor {
	/// 颜色纹理描述
	pub colors_descriptor: SmallVec<[TextureDescriptor; 1]>,
	/// 是否需要深度附件
	pub need_depth: bool,
	/// 深度纹理描述， 如果为None，则使用默认值，默认值为：
	/// TextureDescriptor {
	///		mip_level_count: 1,
	///		sample_count: 1,
	///		dimension: TextureDimension::D2,
	///		format: TextureFormat::Depth32Float,
	///		usage: TextureUsages::COPY_SRC | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,

	///		base_mip_level: 0,
	///		base_array_layer: 0,
	///		array_layer_count: None,
	///		view_dimension: None,
	///	}
	/// 可选的深度附件描述，None时使用默认值
	pub depth_descriptor: Option<TextureDescriptor>,
	/// 默认宽度（如果分配纹理宽度小于default_width，则会直接使用default_width）
	pub default_width: u32,
	/// 默认高度（如果分配纹理高度小于default_height，则会直接使用default_height）
	pub default_height: u32,
}

/// 帧缓冲对象（Framebuffer Object），包含实际纹理资源
#[derive(Debug)]
pub struct Fbo {
	pub depth: Option<(Handle<AssetWithId<TextureRes>>, Share<wgpu::Texture>)>, // 深度附件
	pub colors: SmallVec<[(Handle<AssetWithId<TextureRes>>, Share<wgpu::Texture>);1]>, // 颜色附件
	pub width: u32,  // 纹理实际宽度
	pub height: u32, // 纹理实际高度
}

// TODO Send问题， 临时解决
unsafe impl Send for Fbo {}
unsafe impl Sync for Fbo {}

/// 渲染目标视图，表示分配到的具体纹理区域
#[derive(Debug)]
pub struct TargetView {
    ty_index: DefaultKey,       // 目标类型索引
    index: DefaultKey,          // 在分配器中的索引
    info: Allocation,           // 分配信息（包含边框）
    rect: Rectangle,            // 实际可用区域（不含边框）
    target: Share<Fbo>,         // 关联的FBO
    is_hold: AtomicBool,        // 是否持有该分配区域
}

impl TargetView {
	/// 拿到渲染目标
	pub fn target(&self) -> &Share<Fbo> {
		&self.target
	}
	/// 拿到分配的矩形信息
	pub fn rect_with_border(&self) -> &Rectangle {
		&self.info.rectangle
	}

	pub fn rect(&self) -> &Rectangle {
		&self.rect
	}
	/// 拿到分配的uv
	pub fn uv(&self) -> [f32;8] {
		let (xmin, xmax, ymin, ymax) = (
			self.rect.min.x as f32/self.target.width as f32,
			self.rect.max.x as f32/self.target.width as f32,
			self.rect.min.y as f32/self.target.height as f32,
			self.rect.max.y as f32/self.target.height as f32,
		);
		// [xmin, ymax, xmin, ymin, xmax, ymin, xmax, ymax]
		[xmin, ymin, xmin, ymax, xmax, ymax, xmax, ymin]
	}
	/// 拿到分配的uv
	pub fn uv_box(&self) -> [f32; 4] {
		[
			(self.rect.min.x as f32 + 0.5)/self.target.width as f32,
			(self.rect.min.y as f32 + 0.5)/self.target.height as f32,
			(self.rect.max.x as f32 - 0.5)/self.target.width as f32,
			
			(self.rect.max.y as f32 - 0.5)/self.target.height as f32,
		]
	}

	pub fn rect_normalize(&self) -> [f32; 4] {
		[
			self.rect.min.x as f32 /self.target.width as f32,
			self.rect.min.y as f32 /self.target.height as f32,
			self.rect.max.x as f32 /self.target.width as f32,
			
			self.rect.max.y as f32 /self.target.height as f32,
		]
	}
	/// 渲染目标类型id
	pub fn ty_index(&self) -> DefaultKey {
		self.ty_index
	}
	/// 纹理index
	pub fn target_index(&self) -> DefaultKey {
		self.index
	}
}

#[derive(Debug, Clone, Copy, Hash)]
pub struct TargetType(DefaultKey);

/// 安全的目标视图包装，自动管理生命周期
/// 当SafeTargetView销毁时， 会从纹理分配器中自动释放
#[derive(Deref)]
pub struct SafeTargetView {
	#[deref]
	value: TargetView, // 内嵌的目标视图
	allotor: SafeAtlasAllocator // 关联的分配器（用于自动释放）
}

impl SafeTargetView {
	#[inline]
	pub fn size(&self) -> usize {
		self.allotor.targetview_size(&self.value)
	}

	// /// 丢弃空间占用句柄
	// #[inline]
	// pub fn dicard_hold(&self) {
	// 	self.allotor.0.write().unwrap().dicard_hold(&self.value);
	// }
	
	// 为该TargetView创建一个弱的（不占用空间）SafeTargetView
	pub fn downgrade(&self) -> Self {
		Self {
			value: self.allotor.0.write().unwrap().to_not_hold(&self.value),
			allotor: self.allotor.clone()
		}
	}
}

impl Drop for SafeTargetView {
	/// 析构时自动释放分配的区域
    fn drop(&mut self) {
		self.allotor.0.write().unwrap().deallocate(&self.value);
    }
}

impl std::fmt::Debug for SafeTargetView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SafeTargetView").field(&self.value).finish()
    }
}

pub type ShareTargetView = Share<SafeTargetView>;

pub trait GetTargetView {
	fn get_target_view(&self) -> Option<&TargetView>;
}

impl GetTargetView for TargetView {
    fn get_target_view(&self) -> Option<&TargetView>{
        Some(self)
    }
}

impl<T: GetTargetView + 'static, O: std::ops::Deref<Target=T>> GetTargetView for O {
    fn get_target_view(&self) -> Option<&TargetView>{
		self.deref().get_target_view()
    }
}

/// 线程安全的纹理分配器
#[derive(Clone)]
pub struct SafeAtlasAllocator(pub(crate) Share<ShareRwLock<AtlasAllocator>>);
impl SafeAtlasAllocator {
	pub fn size(&self) -> usize {
		self.0.as_ref().read().unwrap().size()
	}
	/// 创建分配器
	pub fn new(
		device: RenderDevice, 
		texture_assets_mgr: Share<AssetMgr<AssetWithId<TextureRes>>>,
		unuse_textures: Share<HomogeneousMgr<RenderRes<UnuseTexture>>>,
		key_alloter: Share<pi_key_alloter::KeyAlloter>,
	) -> Self {
		Self (
			Share::new(
				ShareRwLock::new(
					AtlasAllocator::new(device, texture_assets_mgr, unuse_textures, key_alloter))))
	}


	pub fn get_or_create_type(&self, descript: TargetDescriptor) -> TargetType {
		self.0.write().unwrap().get_or_create_type(descript)
	}

	/// 创建一个渲染目标类型，并且不共享（get_or_create_type无法通过hash命中该类型）
	#[inline]
	pub fn create_type(&self, descript: TargetDescriptor) -> TargetType {
		self.0.write().unwrap().create_type(descript)
	}

	/// 分配矩形区域
	#[inline]
	pub fn allocate<G: GetTargetView, T: Iterator<Item=G>>(&self, width: u32, height: u32, target_type: TargetType, exclude: T) -> ShareTargetView {
		Share::new(self.allocate_not_share(width, height, target_type, exclude, true))
	}

	/// 分配矩形区域, 但不占用该区域
	#[inline]
	pub fn allocate_not_hold<G: GetTargetView, T: Iterator<Item=G>>(&self, width: u32, height: u32, target_type: TargetType, exclude: T) -> ShareTargetView {
		Share::new(self.allocate_not_share(width, height, target_type, exclude, false))
	}

	/// 分配矩形区域
	#[inline]
	pub fn allocate_not_share<G: GetTargetView, T: Iterator<Item=G>>(&self, width: u32, height: u32, target_type: TargetType, exclude: T, is_hold: bool) -> SafeTargetView {
		SafeTargetView{
			value: self.0.write().unwrap().allocate(width, height, target_type, exclude, is_hold, false),
			allotor: self.clone()
		}
	}

	/// 分配矩形区域, 但不占用该区域
	#[inline]
	pub fn allocate_alone<G: GetTargetView, T: Iterator<Item=G>>(&self, width: u32, height: u32, target_type: TargetType, exclude: T) -> ShareTargetView {
		Share::new(self.allocate_alone_not_share(width, height, target_type, exclude, false))
	}

	/// 分配矩形区域
	#[inline]
	pub fn allocate_alone_not_share<G: GetTargetView, T: Iterator<Item=G>>(&self, width: u32, height: u32, target_type: TargetType, exclude: T, is_hold: bool) -> SafeTargetView {
		SafeTargetView{
			value: self.0.write().unwrap().allocate(width, height, target_type, exclude, is_hold, true),
			allotor: self.clone()
		}
	}
	
	#[inline]
	pub fn targetview_size(&self, target: &TargetView) -> usize {
		self.0.read().unwrap().targetview_size(target)
	}
}

/// 线程不安全的渲染目标分配器
pub(crate) struct AtlasAllocator {
	// 渲染目标类型索引（每种不同的描述，对应一种渲染目标）
	type_map: XHashMap<u64/*TargetDescriptor hash */, DefaultKey/*self.all_allocator index */>,
	// 所有的AllocatorGroup（一个TargetDescriptor对应一个AllocatorGroup）
	pub(crate) all_allocator: SlotMap<DefaultKey, AllocatorGroup>,
	// 深度纹理描述，当前为内置固定描述，是否需要扩展？TODO
	default_depth_descript: TextureDescriptor,
	default_depth_hash: u64,
	// // 未使用的纹理缓冲
	// // 预计纹理格式和尺寸都不会有太大的差距（通常是屏幕大小、rgba格式），所以将所有的未使用纹理放在一起，而不分类
	// unuse_textures: Vec<UnuseTexture>,
	
	unuse_textures: Share<HomogeneousMgr<RenderRes<UnuseTexture>>>,
	// 纹理资源管理器，将纹理资源放入资源管理器，未使用的纹理不立即销毁
	texture_assets_mgr: Share<AssetMgr<AssetWithId<TextureRes>>>,
	key_alloter: Share<pi_key_alloter::KeyAlloter>,
	// 递增的数字，用于缓存纹理创建的纹理（纹理本身描述会重复，不能以描述的hash值作为key，而是以描述hash+ texture_cur_index作为纹理的key）
	texture_cur_index: usize,
	// 渲染设备
	device: RenderDevice,

	// 当前分配需要排除的纹理
	excludes: SecondaryMap<DefaultKey, bool>,
}

const PADDING: i32 = 1;
const DOUBLE_PADDING: u32 = 2;

impl AtlasAllocator {
	pub fn size(&self) -> usize {
		let mut result = 0;
		self.all_allocator.iter().for_each(|(_, item)| {
			result += item.list.capacity() * (168 + 8);
		});
		result += self.type_map.capacity() * 16
		+ self.all_allocator.capacity() * 182
		+ 48
		+ 8
		+ 8
		+ 8
		+ 8
		+ 8
		+ 8
		+ self.excludes.capacity() * 8;
		result
	}
	fn new(
		device: RenderDevice, 
		texture_assets_mgr: Share<AssetMgr<AssetWithId<TextureRes>>>,
		unuse_textures: Share<HomogeneousMgr<RenderRes<UnuseTexture>>>,
		key_alloter: Share<pi_key_alloter::KeyAlloter>,
	) -> Self {
		let d = create_default_depth_descriptor();
		Self {
			type_map: XHashMap::default(),
			all_allocator: SlotMap::default(),
			default_depth_hash: calc_hash(&d),
			default_depth_descript: d,
			unuse_textures,
			texture_cur_index: 0,
			device,
			texture_assets_mgr,
			key_alloter,
			excludes: SecondaryMap::new(),
		}
	}

	/// 获取或创建渲染目标类型
	fn get_or_create_type(&mut self, descript: TargetDescriptor) -> TargetType {
		match self.type_map.entry(calc_hash(&descript)) {
			Entry::Vacant(r) => {
				TargetType(r.insert(Self::create_type_inner(&mut self.all_allocator, descript, self.default_depth_hash)).clone())
			},
			Entry::Occupied(r) => TargetType(r.get().clone())
		}
	}

	/// 创建一个渲染目标类型，并且不共享（get_or_create_type无法通过hash命中该类型）
	#[inline]
	fn create_type(&mut self, descript: TargetDescriptor) -> TargetType {
		TargetType(Self::create_type_inner(&mut self.all_allocator, descript, self.default_depth_hash))
	}

	

	/// 分配TargetView
	/// -is_hold是否占有分配的空间， 如果是true， 将独占该空间， 如果是false， 则与其他分配共享该空间
	fn allocate<G: GetTargetView, T: Iterator<Item=G>>(&mut self, width: u32, height: u32, target_type: TargetType, exclude: T, is_hold: bool, is_alone: bool) -> TargetView {
		let list = match self.all_allocator.get_mut(target_type.0) {
			Some(r) => r,
			None => panic!("TargetType is not exist: {:?}", target_type),
		};
		self.excludes.clear();
		// 将需要排除的渲染目标插入到slotmap中，后续可以更快的判断一个纹理是否需要排除
		for i in exclude {
			let i = i.get_target_view();
			if let Some(i) = i {
				if i.ty_index == target_type.0 {
					self.excludes.insert(i.index, true);
				}
			}
		}
		for (index, item) in list.list.iter_mut(){
			let (offset, width, height) = if is_alone && item.hold_count == 0 && item.target.width == width && item.target.height == height { 
				(0, width, height)
			} else {
				// 不在需要排除的渲染目标上分配
				if self.excludes.get(index).is_some() {
					continue;
				}

				// 数量等于0，保持原大小，否则需要padding
				// 原因是，为了重用屏幕渲染使用的深度缓冲区，通常，fbo的大小与屏幕等大
				// 同时，需要分配的矩形，也很可能与屏幕等大，如果这里不判断item.count == 0，大部分fbo无法容纳与屏幕等大的矩形
				if item.hold_count == 0 {
					(0, width, height)
				} else {
					(PADDING, width + DOUBLE_PADDING, height + DOUBLE_PADDING)
				}
			};
			

			match item.allocator.allocate(Size::new(width as i32, height as i32)) {
				Some(allocation) => {
					// log::warn!("alloct========================{:?}, {:?}, {}, {:?}, {:?}, \n{:?}", std::thread::current().id(), &self.excludes.len(), ii, index, self.excludes.get(index).is_some(), &self.excludes);
					// 在已有的rendertarget中分配成功，直接返回
					item.count += 1;
					let rectangle = &allocation.rectangle;
					let rect = Rectangle::new(
						Point::new(rectangle.min.x + offset, rectangle.min.y + offset),
						Point::new(rectangle.max.x - offset, rectangle.max.y - offset)
					);
					// 如果不需要占有该空间， 则立即释放该空间
					if !is_hold {
						item.allocator.deallocate(allocation.id);
					} else {
						item.hold_count += 1;
					}
					log::trace!("allocate1, is_hold: {:?}, ty_index: {:?}, target_index: {:?}, key: {:?}, rect: {:?}", is_hold, target_type.0, index, allocation.id, rect);

					return TargetView {
						info: allocation,
						rect,
						ty_index: target_type.0,
						index,
						target: item.target.clone(),
						is_hold: AtomicBool::new(is_hold),
					};
				},
				None => (),
			};
		}

		let target = if is_alone {
			Share::new(self.create_target(width, height, width, height, target_type))
		} else {
			Share::new(self.create_target(width, height, Null::null(), Null::null(), target_type))
		};

		// self.debugList.push(Cmd::Create(self.cur_allocator_index, w , h));
		let mut atlas_allocator= guillotiere::AtlasAllocator::new(
			guillotiere::Size::new(target.width as i32, target.height as i32));
		// self.debugList.push(Cmd::Allocate(self.cur_allocator_index, width as i32 , height as i32));
		let allocation= match atlas_allocator.allocate(guillotiere::Size::new(width as i32, height as i32)) {
			Some(r) => r,
			None => panic!("AtlasAllocator allocate first fail, width: {:?}, height: {:?}, target_width : {:?}, target_height: {:?}", width, height, target.width, target.height),
		};
		// 如果不需要占有该空间， 则立即释放该空间
		let hold_count = if !is_hold {
			atlas_allocator.deallocate(allocation.id);
			0
		} else {
			1
		};

		let list = &mut self.all_allocator[target_type.0];
		let index = list.list.insert(SingleAllocator {
			allocator:atlas_allocator,
			target: target.clone(),
			count: 1,
			hold_count: hold_count,
		});
		let rect = allocation.rectangle.clone();
		log::trace!("allocate2, is_hold: {:?}, ty_index: {:?}, target_index: {:?}, key: {:?}, rect: {:?}", is_hold, target_type.0, index, allocation.id, rect);
		return TargetView {
			info: allocation,
			rect,
			target,
			index,
			ty_index: target_type.0,
			is_hold: AtomicBool::new(is_hold),
		}
	}

	// /// 丢弃占用空间（空间可被接下来的分配占用）
	// fn dicard_hold(&mut self, view: &TargetView) {
	// 	let r = view.is_hold.swap(false, std::sync::atomic::Ordering::Relaxed);
	// 	if !r {
	// 		return;
	// 	}
	// 	let alloctor = &mut self.all_allocator[view.ty_index].list[view.index];
	// 	alloctor.allocator.deallocate(view.info.id);
	// }

	/// 取消TargetView分配
	fn deallocate(&mut self, view: &TargetView) {
		let alloctor = &mut self.all_allocator[view.ty_index].list[view.index];
		// 如果TargetView中独占该空间， 则在此时释放分配空间
		let is_hold = view.is_hold.load(std::sync::atomic::Ordering::Relaxed);
		if view.is_hold.load(std::sync::atomic::Ordering::Relaxed) {
			alloctor.allocator.deallocate(view.info.id);
			alloctor.hold_count -= 1;
		}
		alloctor.count -= 1;
		log::trace!("deallocate, count: {:?}, ty_index: {:?}, target_index: {:?}, is_hold: {:?}, key: {:?}, rect: {:?}", alloctor.count, view.ty_index, view.index, is_hold, view.info.id, &view.rect);

		if alloctor.count == 0 {
			let t = self.all_allocator[view.ty_index].list.remove(view.index).unwrap();
			// 缓冲深度纹理
			if let Some(r) = &t.target.depth {
				// log::warn!("drop depth====={:?}, {:?}, ty: {:?}, {:?},", t.target.width, t.target.height, view.ty_index, self.all_allocator[view.ty_index].info.depth_hash);
				self.unuse_textures.create(RenderRes::new(UnuseTexture { 
					view: r.0.clone(),
					texture: r.1.clone(),
					// weak: Share::downgrade(&r.0), 
					// weak_texture: Share::downgrade(&r.1),
					width: t.target.width, 
					height: t.target.height, 
					hash: self.all_allocator[view.ty_index].info.depth_hash, // 深度hash为0，是否需要修改为其他数字，TODO 
				}, size_of::<UnuseTexture>())); 
				// self.unuse_textures.push(
				// 	UnuseTexture { 
				// 		view: (**r.0).clone(),
				// 		texture: (**r.1).clone(),
				// 		// weak: Share::downgrade(&r.0), 
				// 		// weak_texture: Share::downgrade(&r.1),
				// 		width: t.target.width, 
				// 		height: t.target.height, 
				// 		hash: 0, // 深度hash为0，是否需要修改为其他数字，TODO 
				// 	});
			}

			// 缓冲颜色纹理
			for color_index in 0..t.target.colors.len() {
				log::trace!("drop=====ty_index:{:?}, view_index: {:?}, width: {:?}, height: {:?}, target: {:?}", view.ty_index, view.index, t.target.width, t.target.height, self.all_allocator[view.ty_index].info.texture_hash[color_index]);
				self.unuse_textures.create(RenderRes::new(UnuseTexture { 
					view: t.target.colors[color_index].0.clone(),
					texture: t.target.colors[color_index].1.clone(), 
					width: t.target.width, 
					height: t.target.height, 
					hash: self.all_allocator[view.ty_index].info.texture_hash[color_index],
				}, size_of::<UnuseTexture>()));
				// self.unuse_textures.push(
				// 	UnuseTexture { 
				// 		weak: Share::downgrade(&t.target.colors[color_index].0), 
				// 		weak_texture: Share::downgrade(&t.target.colors[color_index].1),
				// 		width: t.target.width, 
				// 		height: t.target.height, 
				// 		hash: self.all_allocator[view.ty_index].info.texture_hash[color_index],
				// 	});
			}
		}
	}

	/// 变为一个不占用空间的targetview
	fn to_not_hold(&mut self, view: &TargetView) -> TargetView {
		let alloctor = &mut self.all_allocator[view.ty_index].list[view.index];
		alloctor.count += 1;
		TargetView {
			ty_index: view.ty_index,
			index: view.index,
			info: view.info,
			rect: view.rect,
			target: view.target.clone(),
			is_hold: AtomicBool::new(false),
		}
	}

	// 渲染目标视图的二进制大小
	fn targetview_size(&self, target: &TargetView) -> usize {
		let info = &self.all_allocator[target.ty_index].info;
		let len = info.descript.colors_descriptor.len();
		let rect = target.rect_with_border();
		let (width, height) = (rect.max.x - rect.min.x, rect.max.y - rect.min.y);

		let mut size = 0;
		for i in 0..len {
			let descriptor = &info.descript.colors_descriptor[i];
			let desc = wgpu::TextureDescriptor {
				label: None,
				size: wgpu::Extent3d {width: width as u32, height: height as u32, depth_or_array_layers: 1},
				mip_level_count: descriptor.mip_level_count,
				sample_count: descriptor.sample_count,
				dimension: descriptor.dimension,
				format: descriptor.format,
				usage: descriptor.usage,
				view_formats: &[],
			};
			size += calc_texture_size(&desc);
		}

		if info.descript.need_depth {
			let descriptor = if let Some(depth_descript) = &info.descript.depth_descriptor {
				depth_descript
			} else {
				&self.default_depth_descript
			};
			let desc = wgpu::TextureDescriptor {
				label: None,
				size: wgpu::Extent3d {width: width as u32, height: height as u32, depth_or_array_layers: 1},
				mip_level_count: descriptor.mip_level_count,
				sample_count: descriptor.sample_count,
				dimension: descriptor.dimension,
				format: descriptor.format,
				usage: descriptor.usage,
				view_formats: &[],
			};
			size += calc_texture_size(&desc);
		}
		size
	}

	fn create_target(
		&mut self, 
		min_width: u32, 
		min_height: u32, 
		width: u32, 
		height: u32, 
		target_type: TargetType,
	)-> Fbo {
		let info: &AllocatorGroupInfo = unsafe { transmute(&self.all_allocator[target_type.0].info) };
		let mut width = if width.is_null() {
			info.descript.default_width.max(min_width)
		} else {
			width
		};
		let mut height = if height.is_null() {
			info.descript.default_height.max(min_height)
		} else {
			height
		};
		// let mut width = info.descript.default_width.max(min_width);
		// let mut height = info.descript.default_height.max(min_height);
		let len = info.descript.colors_descriptor.len();

		let mut target = Fbo {
			depth: None,
			colors: SmallVec::new(),
			width,
			height,
		};

		for i in 0..len {
			let descriptor = &info.descript.colors_descriptor[i];
			let r = self.get_or_create_texture(
				width, 
				height, 
				descriptor,
				TextureAspect::All,
				info.texture_hash[i],
				len,
			);
			if len == 1 {
				width = r.2;
				height = r.3;
				target.width = width;
				target.height = height;
			}
			
			target.colors.push((r.0, r.1));
		}

		if info.descript.need_depth {
			let (descript, depth_hash) = if let Some(depth_descript) = &info.descript.depth_descriptor {
				(depth_descript, info.depth_hash)
			} else {
				(&self.default_depth_descript, self.default_depth_hash)
			};
			let r = self.get_or_create_texture(
				width,
				height,
				unsafe{ transmute(descript)}, // SAFE： 生命周期问题， 这里是安全的，get_or_create_texture内部不会修改descript
				wgpu::TextureAspect::DepthOnly,
				depth_hash,
				2, // 

			);
			target.depth = Some((r.0, r.1));
		}

		return target;
	}

	// 返回纹理和纹理宽高
	fn get_or_create_texture(
		&mut self, 
		width: u32, 
		height: u32, 
		descript: &TextureDescriptor,
		aspect: TextureAspect,
		hash: u64,
		len: usize,
	) -> (Handle<AssetWithId<TextureRes>>, Share<wgpu::Texture>, u32, u32) {
		// 找到一个匹配的纹理，直接返回
		let unuse =  self.unuse_textures.pop_by_filter(|t| {
			if t.hash == hash && 
				(( // 只需要一张纹理，则只要该纹理的大小大于等于要求的大小即可
					len == 1 &&
					t.width >= width &&
					t.height >= height) ||
				( // 需要多张纹理，该纹理的大小必须等于要求的大小（如果大于等于就可以，后续如果找不到缓冲的纹理，则需要创建比要求的大小更大的纹理）
					len > 1 && 
					t.width == width &&
					t.height == height)) {
				return true;
			}
			return false;
		});
		
		if let Some(r) = unuse {
			return (r.view.clone(), r.texture.clone(), r.width, r.height);
		}

		let desc = wgpu::TextureDescriptor {
			label: None,
			size: wgpu::Extent3d {width: width as u32, height: height as u32, depth_or_array_layers: 1},
			mip_level_count: descript.mip_level_count,
			sample_count: descript.sample_count,
			dimension: descript.dimension,
			format: descript.format,
			usage: descript.usage,
			view_formats: &[],
		};
		log::trace!("create target=================={:?}", (width, height));
		// 缓存中不存在，则创建纹理
		let texture: wgpu::Texture = (*self.device).create_texture(&desc);
		let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
			label: None,
			format: Some(descript.format),
			dimension: descript.view_dimension,
			aspect,
			base_mip_level: descript.base_mip_level,
			mip_level_count: if descript.mip_level_count == 0 {None}else {Some(descript.mip_level_count)},
			base_array_layer: descript.base_array_layer,
			array_layer_count: descript.array_layer_count,
		});

		self.texture_cur_index += 1;
		let key = calc_hash(&(hash, self.texture_cur_index, width, height));
		let s = calc_texture_size(&desc);
		(
			match AssetMgr::insert(
				&self.texture_assets_mgr, 
				key, 
				 AssetWithId::new(TextureRes::new(width, height, calc_texture_size(&desc), texture_view, true, descript.format), s, self.key_alloter.clone())) {
					Ok(r) => r,
					_ => panic!("alloc fbo key is exist: {:?}", key),
				},
			Share::new(texture),
			width,
			height
		)
	}

	
	fn create_type_inner(all_allocator: &mut SlotMap<DefaultKey, AllocatorGroup>, descript: TargetDescriptor, mut default_depth_hash: u64) -> DefaultKey {
		let mut texture_hashs = SmallVec::with_capacity(descript.colors_descriptor.len());
		for i in descript.colors_descriptor.iter() {
			texture_hashs.push(calc_hash(i));
		}
		if let Some(r) = &descript.depth_descriptor {
			default_depth_hash = calc_hash(r);
		}
		let ty = all_allocator.insert(
			AllocatorGroup { 
				info: AllocatorGroupInfo { 
					descript: descript, 
					texture_hash: texture_hashs, 
					depth_hash: default_depth_hash,
					// hash: 0, // TODO
				}, 
				list: SlotMap::new() });
		ty
	}
}

pub(super) struct SingleAllocator {
	pub(super) allocator: guillotiere::AtlasAllocator,
	pub(super) target: Share<Fbo>,
	pub(super) count: usize,
	pub(super) hold_count: usize,
}

#[derive(Debug)]
pub struct UnuseTexture {
	// weak: ShareWeak<Droper<AssetWithId<TextureRes>>>,
	// weak_texture: ShareWeak<wgpu::Texture>,
	view: Share<Droper<AssetWithId<TextureRes>>>,
	texture: Share<wgpu::Texture>,
	width: u32,
	height: u32,
	hash: u64,
}

pub struct AllocatorGroup {
	pub(super) info: AllocatorGroupInfo,
	pub(super) list: SlotMap<DefaultKey, SingleAllocator>,
}

pub struct AllocatorGroupInfo {
	pub(super) descript: TargetDescriptor,
	pub(super) texture_hash: SmallVec<[u64;1]>,
	pub(super) depth_hash: u64,
	// hash: u64,
}

fn calc_hash<T: Hash>(v: &T)-> u64 {
	let mut hasher = DefaultHasher::default();
	v.hash(&mut hasher);
	hasher.finish()
}

fn create_default_depth_descriptor() -> TextureDescriptor {
	TextureDescriptor {
		mip_level_count: 1,
		sample_count: 1,
		dimension: TextureDimension::D2,
		format: TextureFormat::Depth32Float,
		usage: TextureUsages::COPY_SRC | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,

		base_mip_level: 0,
		base_array_layer: 0,
		array_layer_count: None,
		view_dimension: None,
	}
}

#[test]
fn test() {
	use guillotiere::Size;
	let mut rr = guillotiere::AtlasAllocator::new(Size::new(1024, 1024));

	let xx = rr.allocate(Size::new(50, 100));

	let zz = rr.allocate(Size::new(300, 200));

	let yy = rr.allocate(Size::new(600, 20));

	rr.deallocate(zz.unwrap().id);

	let aa = rr.allocate(Size::new(20, 20));
	let bb = rr.allocate(Size::new(300, 200));

	println!("xx: {:?}, \nzz: {:?}, \nyy: {:?}, \naa: {:?}, \nbb: {:?}", xx, zz, yy, aa, bb);

}

