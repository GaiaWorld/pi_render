use ktx::KtxInfo;
use pi_hal::image::DynamicImage;
use pi_hash::XHashMap;
use crate::{renderer::texture::{CombineAtlas2DMgr, ImageTextureFrame, KeyImageTextureFrame}, rhi::{device::RenderDevice, RenderQueue}};

pub const COMPRESSED_RGB_S3TC_DXT1_EXT : u32          = 0x83F0;
pub const COMPRESSED_RGBA_S3TC_DXT1_EXT: u32          = 0x83F1;
pub const COMPRESSED_RGBA_S3TC_DXT3_EXT: u32          = 0x83F2;
pub const COMPRESSED_RGBA_S3TC_DXT5_EXT: u32          = 0x83F3;
pub const COMPRESSED_SRGB_S3TC_DXT1_EXT      : u32    = 0x8C4C;
pub const COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT: u32    = 0x8C4D;
pub const COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT: u32    = 0x8C4E;
pub const COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT: u32    = 0x8C4F;
pub const COMPRESSED_RGBA_ASTC_4X4_KHR          : u32 = 0x93B0;
pub const COMPRESSED_RGBA_ASTC_5X4_KHR          : u32 = 0x93B1;
pub const COMPRESSED_RGBA_ASTC_5X5_KHR          : u32 = 0x93B2;
pub const COMPRESSED_RGBA_ASTC_6X5_KHR          : u32 = 0x93B3;
pub const COMPRESSED_RGBA_ASTC_6X6_KHR          : u32 = 0x93B4;
pub const COMPRESSED_RGBA_ASTC_8X5_KHR          : u32 = 0x93B5;
pub const COMPRESSED_RGBA_ASTC_8X6_KHR          : u32 = 0x93B6;
pub const COMPRESSED_RGBA_ASTC_8X8_KHR          : u32 = 0x93B7;
pub const COMPRESSED_RGBA_ASTC_10X5_KHR         : u32 = 0x93B8;
pub const COMPRESSED_RGBA_ASTC_10X6_KHR         : u32 = 0x93B9;
pub const COMPRESSED_RGBA_ASTC_10X8_KHR         : u32 = 0x93BA;
pub const COMPRESSED_RGBA_ASTC_10X10_KHR        : u32 = 0x93BB;
pub const COMPRESSED_RGBA_ASTC_12X10_KHR        : u32 = 0x93BC;
pub const COMPRESSED_RGBA_ASTC_12X12_KHR        : u32 = 0x93BD;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_4X4_KHR  : u32 = 0x93D0;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_5X4_KHR  : u32 = 0x93D1;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_5X5_KHR  : u32 = 0x93D2;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_6X5_KHR  : u32 = 0x93D3;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_6X6_KHR  : u32 = 0x93D4;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_8X5_KHR  : u32 = 0x93D5;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_8X6_KHR  : u32 = 0x93D6;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_8X8_KHR  : u32 = 0x93D7;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_10X5_KHR : u32 = 0x93D8;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_10X6_KHR : u32 = 0x93D9;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_10X8_KHR : u32 = 0x93DA;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_10X10_KHR: u32 = 0x93DB;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_12X10_KHR: u32 = 0x93DC;
pub const COMPRESSED_SRGB8_ALPHA8_ASTC_12X12_KHR: u32 = 0x93DD;

pub fn compressed_texture_format(val: u32) -> Option<wgpu::TextureFormat> {
    match val {
        COMPRESSED_RGBA_S3TC_DXT1_EXT         => { Some(wgpu::TextureFormat::Bc1RgbaUnorm) },
        COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT   => { Some(wgpu::TextureFormat::Bc1RgbaUnormSrgb) },
        COMPRESSED_RGBA_S3TC_DXT3_EXT         => { Some(wgpu::TextureFormat::Bc2RgbaUnorm) },
        COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT   => { Some(wgpu::TextureFormat::Bc2RgbaUnormSrgb) },
        COMPRESSED_RGBA_S3TC_DXT5_EXT         => { Some(wgpu::TextureFormat::Bc3RgbaUnorm) },
        COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT   => { Some(wgpu::TextureFormat::Bc3RgbaUnormSrgb) },
        COMPRESSED_RGBA_ASTC_4X4_KHR          => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B4x4  , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_5X4_KHR          => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B5x4  , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_5X5_KHR          => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B5x5  , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_6X5_KHR          => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B6x5  , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_6X6_KHR          => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B6x6  , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_8X5_KHR          => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B8x5  , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_8X6_KHR          => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B8x6  , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_8X8_KHR          => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B8x8  , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_10X5_KHR         => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B10x5 , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_10X6_KHR         => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B10x6 , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_10X8_KHR         => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B10x8 , channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_10X10_KHR        => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B10x10, channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_12X10_KHR        => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B12x10, channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_RGBA_ASTC_12X12_KHR        => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B12x12, channel: wgpu::AstcChannel::Unorm }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_4X4_KHR  => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B4x4  , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_5X4_KHR  => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B5x4  , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_5X5_KHR  => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B5x5  , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_6X5_KHR  => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B6x5  , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_6X6_KHR  => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B6x6  , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_8X5_KHR  => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B8x5  , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_8X6_KHR  => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B8x6  , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_8X8_KHR  => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B8x8  , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_10X5_KHR => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B10x5 , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_10X6_KHR => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B10x6 , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_10X8_KHR => { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B10x8 , channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_10X10_KHR=> { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B10x10, channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_12X10_KHR=> { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B12x10, channel: wgpu::AstcChannel::UnormSrgb }) },
        COMPRESSED_SRGB8_ALPHA8_ASTC_12X12_KHR=> { Some(wgpu::TextureFormat::Astc { block: wgpu::AstcBlock::B12x12, channel: wgpu::AstcChannel::UnormSrgb }) },
        _ => { None }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyAtlasDesc {
    pub format: wgpu::TextureFormat,
}

/// 纹理图集管理器, 用于合并图块
#[derive(Default)]
pub struct TextureCombineAtlas2DMgr {
    pub(crate) map: XHashMap<KeyAtlasDesc, CombineAtlas2DMgr>,
}
impl TextureCombineAtlas2DMgr {
    // 拓展指定纹理格式的图集管理器
    pub fn append_desc(&mut self, desc: KeyAtlasDesc, device: &RenderDevice, maxlayer: u32, maxsize: u32, maxcount: usize) -> bool {
        if self.map.contains_key(&desc) == false {
            let format: wgpu::TextureFormat = desc.format;
            self.map.insert(desc, CombineAtlas2DMgr::new(device, format, maxlayer, maxsize, maxcount));
            true
        } else { false }
    }
    // 查询是否支持对应纹理格式的图集
    pub fn query_desc(&self, desc: &KeyAtlasDesc) -> bool {
        // log::error!("query_desc {}", self.map.contains_key(desc));
        self.map.contains_key(desc)
    }
    // 尝试合并压缩纹理图块
    pub fn combine_ktx(&mut self, _keyimage: &KeyImageTextureFrame, data: &[u8], device: &RenderDevice, queue: &RenderQueue) -> Option<ImageTextureFrame> {
        let ktx = ktx::Ktx::new(data);
        if let Some(format) = compressed_texture_format(ktx.gl_internal_format()) {
            let key = KeyAtlasDesc { format };
            let _mipmaps = ktx.mipmap_levels();
            
            if ktx.textures().count() == 0 || ktx.textures().count() > 1 || ktx.faces() > 1 || ktx.pixel_depth() > 1 {
                return None;
            }
            
            if let Some(atlas) = self.map.get_mut(&key) {
                if let Some(texture) = atlas.combine(format, ktx.pixel_width(), ktx.pixel_height(), device, queue) {
                    // log::error!("Combine: {:?}", (&keyimage.url));
                    for data in ktx.textures() {
                        texture.update_texture(queue, data);
                    }
                    return Some(texture);
                }
            }
        }
        return None;
    }
    // 尝试合并普通图片纹理图块
    pub fn combine_image(&mut self, _keyimage: &KeyImageTextureFrame, data: &DynamicImage, device: &RenderDevice, queue: &RenderQueue) -> Option<ImageTextureFrame> {

        match &data {
            pi_hal::image::DynamicImage::ImageLuma8(image_buffer) => {
                let data = image_buffer.as_raw();
                let format = wgpu::TextureFormat::R8Unorm;
                let key = KeyAtlasDesc { format };
                if let Some(atlas) = self.map.get_mut(&key) {
                    if let Some(texture) = atlas.combine(format, image_buffer.width(), image_buffer.height(), device, queue) {
                        texture.update_texture(queue, data);
                        return Some(texture);
                    }
                }
            },
            pi_hal::image::DynamicImage::ImageRgb8(image_buffer) => {
                let data = data.to_rgba8();
                let data = data.as_raw();
                let format = wgpu::TextureFormat::Rgba8Unorm;
                let key = KeyAtlasDesc { format };
                if let Some(atlas) = self.map.get_mut(&key) {
                    if let Some(texture) = atlas.combine(format, image_buffer.width(), image_buffer.height(), device, queue) {
                        texture.update_texture(queue, data);
                        return Some(texture);
                    }
                }
            },
            pi_hal::image::DynamicImage::ImageRgba8(image_buffer) => {
                let data = image_buffer.as_raw();
                let format = wgpu::TextureFormat::Rgba8Unorm;
                let key = KeyAtlasDesc { format };
                if let Some(atlas) = self.map.get_mut(&key) {
                    if let Some(texture) = atlas.combine(format, image_buffer.width(), image_buffer.height(), device, queue) {
                        // log::error!("ImageRgb8 Combine");
                        texture.update_texture(queue, data);
                        return Some(texture);
                    }
                }
            },
            pi_hal::image::DynamicImage::ImageLuma16(_image_buffer) => {},
            pi_hal::image::DynamicImage::ImageLumaA16(_image_buffer) => {},
            pi_hal::image::DynamicImage::ImageRgb16(_image_buffer) => {},
            pi_hal::image::DynamicImage::ImageRgba16(_image_buffer) => {},
            pi_hal::image::DynamicImage::ImageRgb32F(_image_buffer) => {},
            pi_hal::image::DynamicImage::ImageRgba32F(_image_buffer) => {},
            _ => {},
        }
        return None;
    }
}