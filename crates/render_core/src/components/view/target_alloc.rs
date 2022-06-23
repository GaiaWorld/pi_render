//! 渲染目标分配器

use std::{hash::{Hash, Hasher}, collections::hash_map::Entry, intrinsics::transmute, num::NonZeroU32, ops::Deref};

use guillotiere::{Size, Allocation, Rectangle};
use pi_assets::{asset::{Handle, Droper}, mgr::AssetMgr};
use pi_share::{Share, ShareWeak, ShareRwLock};
use pi_slotmap::{DefaultKey, SlotMap, SecondaryMap};
use pi_hash::{DefaultHasher, XHashMap};
use pi_atom::Atom;
use smallvec::SmallVec;
use wgpu::{TextureAspect, TextureDimension, TextureFormat, TextureUsages, TextureViewDimension};

use crate::rhi::{asset::{RenderRes, calc_texture_size}, device::RenderDevice};

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
    pub array_layer_count: Option<NonZeroU32>,
	pub view_dimension: Option<TextureViewDimension>,
}

/// 渲染目标描述
#[derive(Debug, Hash, Clone)]
pub struct TargetDescriptor {
	pub texture_descriptor: SmallVec<[TextureDescriptor; 1]>,
	pub need_depth: bool,
	pub default_width: u32,
	pub default_height: u32,
}

/// 渲染目标
pub struct Fbo {
	pub depth: Option<(Handle<RenderRes<wgpu::TextureView>>, Share<wgpu::Texture>)>,
	pub colors: SmallVec<[(Handle<RenderRes<wgpu::TextureView>>, Share<wgpu::Texture>);1]>,
	pub width: u32,
	pub height: u32,
}

/// 渲染目标视图
pub struct TargetView {
	ty_index: DefaultKey,
	index: DefaultKey, // 第几张纹理
	info: Allocation,
	target: Share<Fbo>,
}

impl TargetView {
	/// 拿到渲染目标
	pub fn target(&self) -> &Share<Fbo> {
		&self.target
	}
	/// 拿到分配的矩形信息
	pub fn rect(&self) -> &Rectangle {
		&self.info.rectangle
	}
	/// 拿到分配的uv
	pub fn uv(&self) -> [f32;8] {
		let (xmin, xmax, ymin, ymax) = (
			self.info.rectangle.min.x as f32/self.target.width as f32,
			self.info.rectangle.max.x as f32/self.target.width as f32,
			self.info.rectangle.min.y as f32/self.target.height as f32,
			self.info.rectangle.max.y as f32/self.target.height as f32,
		);
		// [xmin, ymax, xmin, ymin, xmax, ymin, xmax, ymax]
		[xmin, ymin, xmin, ymax, xmax, ymax, xmax, ymin]
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

#[derive(Clone)]
pub struct ShareTargetView(Share<(TargetView, SafeAtlasAllocator)>);

impl Deref for ShareTargetView {
    type Target = TargetView;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
}

impl Drop for ShareTargetView {
    fn drop(&mut self) {
		if Share::strong_count(&self.0) == 1 {
			self.0.1.0.write().unwrap().deallocate(&self.0.0);
		}
    }
}

impl GetTargetView for ShareTargetView {
    fn get_target_view(&self) -> Option<&TargetView>{
        Some(&self.0.0)
    }
}

impl GetTargetView for &ShareTargetView {
    fn get_target_view(&self) -> Option<&TargetView>{
        Some(&self.0.0)
    }
}

impl GetTargetView for Option<ShareTargetView> {
    fn get_target_view(&self) -> Option<&TargetView>{
		match self {
			Some(r) => Some(&r.0.0),
			None => None
		}
	}
}

impl GetTargetView for &Option<ShareTargetView> {
    fn get_target_view(&self) -> Option<&TargetView>{
		match self {
			Some(r) => Some(&r.0.0),
			None => None
		}
	}
}

pub trait GetTargetView {
	fn get_target_view(&self) -> Option<&TargetView>;
}

/// 线程安全的纹理分配器
#[derive(Clone)]
pub struct SafeAtlasAllocator(Share<ShareRwLock<AtlasAllocator>>);
impl SafeAtlasAllocator {
	/// 创建分配器
	pub fn new(device: RenderDevice, texture_assets_mgr: Share<AssetMgr<RenderRes<wgpu::TextureView>>>) -> Self {
		Self (
			Share::new(
				ShareRwLock::new(
					AtlasAllocator::new(device, texture_assets_mgr))))
	}

	pub fn get_or_create_type(&self, descript: TargetDescriptor) -> TargetType {
		self.0.write().unwrap().get_or_create_type(descript)
	}

	/// 创建一个渲染目标类型，并且不共享（get_or_create_type无法通过hash命中该类型）
	#[inline]
	pub fn create_type(&mut self, descript: TargetDescriptor) -> TargetType {
		self.0.write().unwrap().create_type(descript)
	}

	/// 分配矩形区域
	pub fn allocate<G: GetTargetView, T: Iterator<Item=G>>(&self, width: u32, height: u32, target_type: TargetType, exclude: T) -> ShareTargetView {
		ShareTargetView(
			Share::new(
				(self.0.write().unwrap().allocate(width, height, target_type, exclude), self.clone())
			)
		)
	}
}

/// 线程不安全的渲染目标分配器
struct AtlasAllocator {
	// 渲染目标类型索引（每种不同的描述，对应一种渲染目标）
	type_map: XHashMap<u64/*TargetDescriptor hash */, DefaultKey/*self.all_allocator index */>,
	// 所有的AllocatorGroup（一个TargetDescriptor对应一个AllocatorGroup）
	all_allocator: SlotMap<DefaultKey, AllocatorGroup>,
	// 深度纹理描述，当前为内置固定描述，是否需要扩展？TODO
	depth_descript: TextureDescriptor,
	// 未使用的纹理缓冲
	// 预计纹理格式和尺寸都不会有太大的差距（通常是屏幕大小、rgba格式），所以将所有的未使用纹理放在一起，而不分类
	unuse_textures: Vec<UnuseTexture>,
	// 纹理资源管理器，将纹理资源放入资源管理器，未只用的纹理不立即销毁
	texture_assets_mgr: Share<AssetMgr<RenderRes<wgpu::TextureView>>>,
	// 递增的数字，用于缓存纹理创建的纹理（纹理本身描述会重复，不能以描述的hash值作为key，而是以描述hash+ texture_cur_index作为纹理的key）
	texture_cur_index: usize,
	// 渲染设备
	device: RenderDevice,

	// 当前分配需要排除的纹理
	excludes: SecondaryMap<DefaultKey, bool>,
}

impl AtlasAllocator {
	fn new(device: RenderDevice, texture_assets_mgr: Share<AssetMgr<RenderRes<wgpu::TextureView>>>) -> Self {
		Self {
			type_map: XHashMap::default(),
			all_allocator: SlotMap::default(),
			depth_descript: TextureDescriptor {
				mip_level_count: 1,
				sample_count: 1,
				dimension: TextureDimension::D2,
				format: TextureFormat::Depth32Float,
				usage: TextureUsages::COPY_SRC | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,

				base_mip_level: 0,
				base_array_layer: 0,
				array_layer_count: None,
				view_dimension: None,
			},
			unuse_textures: Vec::new(),
			texture_cur_index: 0,
			device,
			texture_assets_mgr,
			excludes: SecondaryMap::new(),
		}
	}

	/// 获取或创建渲染目标类型
	fn get_or_create_type(&mut self, descript: TargetDescriptor) -> TargetType {
		match self.type_map.entry(calc_hash(&descript)) {
			Entry::Vacant(r) => {
				TargetType(r.insert(Self::create_type_inner(&mut self.all_allocator, descript)).clone())
			},
			Entry::Occupied(r) => TargetType(r.get().clone())
		}
	}

	/// 创建一个渲染目标类型，并且不共享（get_or_create_type无法通过hash命中该类型）
	#[inline]
	fn create_type(&mut self, descript: TargetDescriptor) -> TargetType {
		TargetType(Self::create_type_inner(&mut self.all_allocator, descript))
	}

	/// 分配TargetView
	fn allocate<G: GetTargetView, T: Iterator<Item=G>>(&mut self, width: u32, height: u32, target_type: TargetType, exclude: T) -> TargetView {
		let list = self.all_allocator.get_mut(target_type.0).unwrap();
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
			// 不在需要排除的渲染目标上分配
			if self.excludes.get(index).is_some() {
				continue;
			}

			match item.allocator.allocate(Size::new(width as i32, height as i32)) {
				Some(allocation) => {
					// 在已有的rendertarget中分配成功，直接返回
					item.count += 1;
					return TargetView {
						info: allocation,
						ty_index: target_type.0,
						index,
						target: item.target.clone(),
					};
				},
				None => (),
			};
		}
		self.excludes.clear();

		let target = Share::new(self.create_target(width, height, target_type));

		// self.debugList.push(Cmd::Create(self.cur_allocator_index, w , h));
		let mut atlas_allocator= guillotiere::AtlasAllocator::new(
			guillotiere::Size::new(target.width as i32, target.height as i32));
		// self.debugList.push(Cmd::Allocate(self.cur_allocator_index, width as i32 , height as i32));
		let allocation= atlas_allocator.allocate(guillotiere::Size::new(width as i32, height as i32)).unwrap();
		let list = &mut self.all_allocator[target_type.0];
		let index = list.list.insert(SingleAllocator {
			allocator:atlas_allocator,
			target: target.clone(),
			count: 1,
		});

		return TargetView {
			info: allocation,
			target,
			index,
			ty_index: target_type.0,
		}
	}

	/// 取消TargetView分配
	fn deallocate(&mut self, view: &TargetView) {
		let mut alloctor = &mut self.all_allocator[view.ty_index].list[view.index];
		alloctor.allocator.deallocate(view.info.id);
		alloctor.count -= 1;

		if alloctor.count == 0 {
			let t = self.all_allocator[view.ty_index].list.remove(view.index).unwrap();
			// 缓冲深度纹理
			if let Some(r) = &t.target.depth {
				self.unuse_textures.push(
					UnuseTexture { 
						weak: Share::downgrade(&r.0), 
						weak_texture: Share::downgrade(&r.1),
						width: t.target.width, 
						height: t.target.height, 
						hash: 0, // 深度hash为0，是否需要修改为其他数字，TODO 
					});
			}

			// 缓冲颜色纹理
			for color_index in 0..t.target.colors.len() {
				self.unuse_textures.push(
					UnuseTexture { 
						weak: Share::downgrade(&t.target.colors[color_index].0), 
						weak_texture: Share::downgrade(&t.target.colors[color_index].1),
						width: t.target.width, 
						height: t.target.height, 
						hash: self.all_allocator[view.ty_index].info.texture_hash[color_index],
					});
			}
		}
	}

	fn create_target(
		&mut self, 
		min_width: u32, 
		min_height: u32, 
		target_type: TargetType,
	)-> Fbo {
		let info: &AllocatorGroupInfo = unsafe { transmute(&self.all_allocator[target_type.0].info) };
		let width = info.descript.default_width.max(min_width);
		let height = info.descript.default_height.max(min_height);
		let len = info.descript.texture_descriptor.len();

		let mut target = Fbo {
			depth: None,
			colors: SmallVec::new(),
			width,
			height,
		};
		for i in 0..len {
			let descriptor = &info.descript.texture_descriptor[i];
			target.colors.push(self.get_or_create_texture(
				width, 
				height, 
				descriptor,
				TextureAspect::All,
				info.texture_hash[i],
				1,
			));
		}

		if info.descript.need_depth {
			target.depth = Some(self.get_or_create_texture(
				width,
				height,
				unsafe{ transmute(&self.depth_descript)},
				wgpu::TextureAspect::DepthOnly,
				0, // 深度纹理共用，默认hash为0
				1,
			));
		}

		return target;
	}

	fn get_or_create_texture(
		&mut self, 
		width: u32, 
		height: u32, 
		descript: &TextureDescriptor,
		aspect: TextureAspect,
		hash: u64,
		len: usize) -> (Handle<RenderRes<wgpu::TextureView>>, Share<wgpu::Texture>) {
		if self.unuse_textures.len() > 0 {
			let mut i = 0;
			while i < self.unuse_textures.len() {
				let t = &self.unuse_textures[i];
				if t.hash == hash && 
					(( // 只需要一张纹理，则只要该纹理的大小大于等于要求的大小即可
						len == 1 &&
						t.width >= width &&
						t.height >= height) ||
					( // 需要多张纹理，该纹理的大小必须等于要求的大小（如果大于等于就可以，后续如果找不到缓冲的纹理，则需要创建比要求的大小更大的纹理）
						len > 1 && 
						t.width == width &&
						t.height == height)) {

					let unuse_texture = self.unuse_textures.swap_remove(i);
					match unuse_texture.weak.upgrade() {
						Some(r) => {
							return (r, unuse_texture.weak_texture.upgrade().unwrap());
						},
						None => {
							continue
						},
					};
				} else {
					i += 1;
				}
			}
		}

		let desc = wgpu::TextureDescriptor {
			label: None,
			size: wgpu::Extent3d {width: width as u32, height: height as u32, depth_or_array_layers: 1},
			mip_level_count: descript.mip_level_count,
			sample_count: descript.sample_count,
			dimension: descript.dimension,
			format: descript.format,
			usage: descript.usage,
		};
		// 缓存中不存在，则创建纹理
		let texture: wgpu::Texture = (*self.device).create_texture(&desc);
		let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
			label: None,
			format: Some(descript.format),
			dimension: descript.view_dimension,
			aspect,
			base_mip_level: descript.base_mip_level,
			mip_level_count:  NonZeroU32::new(descript.mip_level_count),
			base_array_layer: descript.base_array_layer,
			array_layer_count: descript.array_layer_count,
		});

		self.texture_cur_index += 1;
		let key = calc_hash(&(hash, self.texture_cur_index, width, height));
		(AssetMgr::insert(
			&self.texture_assets_mgr, 
			key, 
			RenderRes::new(texture_view, calc_texture_size(desc))).unwrap(),
			Share::new(texture)
		)
	}

	
	fn create_type_inner(all_allocator: &mut SlotMap<DefaultKey, AllocatorGroup>, descript: TargetDescriptor) -> DefaultKey {
		let mut texture_hashs = SmallVec::with_capacity(descript.texture_descriptor.len());
		for i in descript.texture_descriptor.iter() {
			texture_hashs.push(calc_hash(i));
		}
		let ty = all_allocator.insert(
			AllocatorGroup { 
				info: AllocatorGroupInfo { 
					descript: descript, 
					texture_hash: texture_hashs, 
					// hash: 0, // TODO
				}, 
				list: SlotMap::new() });
		ty
	}
}

struct SingleAllocator {
	allocator: guillotiere::AtlasAllocator,
	target: Share<Fbo>,
	count: usize,
}

struct UnuseTexture {
	weak: ShareWeak<Droper<RenderRes<wgpu::TextureView>>>,
	weak_texture: ShareWeak<wgpu::Texture>,
	width: u32,
	height: u32,
	hash: u64,
}

pub struct AllocatorGroup {
	info: AllocatorGroupInfo,
	list: SlotMap<DefaultKey, SingleAllocator>,
}

pub struct AllocatorGroupInfo {
	descript: TargetDescriptor,
	texture_hash: SmallVec<[u64;1]>,
	// hash: u64,
}

fn calc_hash<T: Hash>(v: &T)-> u64 {
	let mut hasher = DefaultHasher::default();
	v.hash(&mut hasher);
	hasher.finish()
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

