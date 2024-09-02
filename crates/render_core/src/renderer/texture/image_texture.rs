
use pi_assets::asset::{Asset, Size};
use pi_atom::Atom;
use pi_hal::texture::ImageTexture;
use pi_hash::XHashMap;

use crate::{rhi::{device::RenderDevice, RenderQueue}, asset::TAssetKeyU64, renderer::buildin_data::DefaultTexture};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TexturePath {
    /// 路径
    pub url: Atom,
    /// 是否 sRGB
    pub srgb: bool,
    /// 是否从文件加载
    pub file: bool,
    /// 是否压缩纹理
    pub compressed: bool,
    pub depth_or_array_layers: u8,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyImageTexture {
    /// 路径
    pub url: Atom,
    /// 是否 sRGB
    pub srgb: bool,
    /// 是否从文件加载
    pub file: bool,
    /// 是否压缩纹理
    pub compressed: bool,
    pub depth_or_array_layers: u8,
    pub useage: wgpu::TextureUsages,
}
impl Default for KeyImageTexture {
    fn default() -> Self {
        Self { url: Atom::from(DefaultTexture::BLACK_2D), srgb: false, file: false, compressed: false, depth_or_array_layers: 1, useage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING }
    }
}
impl TAssetKeyU64 for KeyImageTexture {}
impl std::ops::Deref for KeyImageTexture {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.url.as_str()
    }
}


pub struct ResImageTexture {
    pub(crate) data: ImageTexture,
    pub extend: Vec<u8>,
}

impl ResImageTexture {
    pub fn new(data: ImageTexture) -> Self {
        Self { data, extend: vec![] }
    }
    pub fn create_data_texture(device: &RenderDevice, queue: &RenderQueue, key: &KeyImageTexture, data: &[u8], width: u32, height: u32, format: wgpu::TextureFormat, dimension: wgpu::TextureViewDimension, pre_pixel_size: u32, is_opacity: bool) -> Self {
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = (**device).create_texture(&wgpu::TextureDescriptor {
            label: Some(key.url.as_str()),
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

        let data: ImageTexture = ImageTexture {
            width, height, size, texture, format, view_dimension: dimension, is_opacity
        };
        Self::new(data)
    }

    pub fn update(&self, queue: &RenderQueue, data: &[u8], xoffset: u32, yoffset: u32, width: u32, height: u32) {
        let offset = (yoffset * self.data.width + xoffset) as u64;
        let temp = self.data.texture.as_image_copy();
        queue.write_texture(temp, data, wgpu::ImageDataLayout { offset, bytes_per_row: None, rows_per_image: None  }, wgpu::Extent3d { width, height, depth_or_array_layers: 1 });
    }

    pub fn width(&self) -> u32 {
        self.data.width
    }
    pub fn height(&self) -> u32 {
        self.data.height
    }
}

impl Asset for ResImageTexture {
    type Key = KeyImageTexture;
    // const TYPE: &'static str = "ImageTexture";
}
impl Size for ResImageTexture {
    fn size(&self) -> usize {
        self.data.size
    }
}

#[derive(Debug,Clone, Copy)]
pub enum ErrorImageTexture {
    LoadFail,
    CacheError,
    FormatError,
    CreateError,
}

pub struct ImageTextureErrorMap(pub XHashMap<KeyImageTexture, ErrorImageTexture>);

#[derive(Clone)]
pub struct ImageTexture2DDesc {
    pub url: KeyImageTexture,
    pub device: RenderDevice,
    pub queue: RenderQueue,
}