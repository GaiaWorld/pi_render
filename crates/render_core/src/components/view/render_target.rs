use std::sync::atomic::AtomicBool;

use guillotiere::{Allocation, Rectangle};
use pi_assets::asset::Handle;
use pi_share::Share;
use pi_slotmap::DefaultKey;
use smallvec::SmallVec;

use crate::rhi::asset::{AssetWithId, TextureRes};

pub struct RenderTargetDescriptor {
    pub color_format: SmallVec<[wgpu::TextureFormat; 1]>,
    pub depth_format: Option<wgpu::TextureFormat>,
	/// 默认宽度（如果分配纹理宽度小于default_width，则会直接使用default_width）
	pub default_width: u32,
	/// 默认高度（如果分配纹理高度小于default_height，则会直接使用default_height）
	pub default_height: u32,
}

pub struct FrameBuffer {
	pub depth: Option<(Handle<AssetWithId<TextureRes>>, Share<wgpu::Texture>)>,
	pub colors: SmallVec<[(Handle<AssetWithId<TextureRes>>, Share<wgpu::Texture>);1]>,
	pub width: u32,
	pub height: u32,
}
// TODO Send问题， 临时解决
unsafe impl Send for FrameBuffer {}
unsafe impl Sync for FrameBuffer {}


/// 渲染目标视图
pub struct RenderTargetView {
	ty_index: DefaultKey,
	index: DefaultKey, // 第几张纹理
	info: Allocation,
	rect: Rectangle, // target的宽高（不包含边框）
	target: Share<FrameBuffer>,
	is_hold: AtomicBool,
}
impl RenderTargetView {
	/// 拿到渲染目标
	pub fn target(&self) -> &Share<FrameBuffer> {
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
	/// 渲染目标类型id
	pub fn ty_index(&self) -> DefaultKey {
		self.ty_index
	}
	/// 纹理index
	pub fn target_index(&self) -> DefaultKey {
		self.index
	}
}

