
use pi_slotmap::DefaultKey;
use serde::{Serialize, Deserialize};

use super::target_alloc::SafeAtlasAllocator;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AtlasAllocatorDebuger(pub Vec<TargetDescriptor>);


#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TargetDescriptor {
    pub key: DefaultKey,
    pub count: usize,
    pub targets: Vec<Target>,
    pub colors_descriptor: Vec<TextureDescriptor>,
	pub need_depth: bool,
	pub depth_descriptor: Option<TextureDescriptor>,
	pub default_width: u32,
	pub default_height: u32,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Target {
    key: DefaultKey,
    rect_count: usize,
    width: usize,
    height: usize,
}

impl AtlasAllocatorDebuger {
	pub fn debug_info(alloter: &SafeAtlasAllocator) -> Self {
        let alloter = alloter.0.write().unwrap();
        let mut list = Self::default();
		for (key, group) in alloter.all_allocator.iter() {
            let info = &group.info;
            let mut desc = TargetDescriptor::default();
            desc.key = key;
            desc.count = group.list.len();
            for (key, target) in group.list.iter() {
                desc.targets.push(Target {
                    key,
                    width: target.target.width as usize, 
                    height: target.target.height as usize,
                    rect_count: target.count,
                });
            }
            desc.colors_descriptor = info.descript.colors_descriptor.iter().map(|r| {TextureDescriptor::from(r)}).collect();
            desc.need_depth = info.descript.need_depth;
            desc.depth_descriptor = info.descript.depth_descriptor.map(|r| {TextureDescriptor::from(&r)});
            desc.default_width = info.descript.default_width;
            desc.default_height= info.descript.default_height;

            list.0.push(desc);
		}
		list
	}
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct TextureDescriptor {
	pub mip_level_count: u32,
	pub sample_count: u32,
	pub dimension: String,
	pub format: String,
	pub usage: String,
	pub base_mip_level: u32,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
	pub view_dimension: Option<String>,
}

impl TextureDescriptor {
    pub fn from(v: &super::target_alloc::TextureDescriptor) -> Self {
        Self {
            mip_level_count: v.mip_level_count,
            sample_count: v.sample_count,
            dimension: format!("{:?}", v.dimension),
            format: format!("{:?}", v.format),
            usage: format!("{:?}", v.usage),
            base_mip_level: v.base_mip_level,
            base_array_layer: v.base_array_layer,
            array_layer_count: v.array_layer_count,
            view_dimension: v.view_dimension.map(|v| format!("{:?}", v)),
        }
    }
}