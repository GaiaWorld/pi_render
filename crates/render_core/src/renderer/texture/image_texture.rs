
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
    pub fn texture(&self) -> &wgpu::Texture {
        &self.data.texture
    }
    pub fn create_data_texture(
        device: &RenderDevice, queue: &RenderQueue, key: &KeyImageTexture, width: u32, height: u32,
        format: wgpu::TextureFormat, dimension: wgpu::TextureViewDimension, is_opacity: bool, depth_or_array_layers: u32, aspect: Option<wgpu::TextureAspect>,
        data: Option<&[u8]>, dataoffset: u64
    ) -> Self {
        let texture = ResImageTexture::create_texture(device, key, width, height, format, dimension.compatible_texture_dimension(), depth_or_array_layers);

        let (block_width, block_height) = format.block_dimensions();
        let mut extent_width    = width / block_width;
        let mut extent_height   = height / block_height;
        if extent_width * block_width < width {
            extent_width += 1;
        }
        if extent_height * block_height < height {
            extent_height += 1;
        }
        let bytes_per_row = if let Some(pre_pixel_size) = format.block_copy_size(aspect) {
            Some(extent_width * pre_pixel_size)
        } else { None };

        if let Some(data) = data {
            let offset = dataoffset;

            let texture_extent = wgpu::Extent3d {
                width: extent_width,
                height: extent_height,
                depth_or_array_layers,
            };
            queue.write_texture(
                texture.as_image_copy(),
                data,
                wgpu::ImageDataLayout {
                    offset,
                    bytes_per_row,
                    rows_per_image: None,
                },
                texture_extent,
            );
        }

        // log::error!("{:?}", (key, width, height, format, dimension, depth_or_array_layers, block_width, block_height, extent_width, extent_height, bytes_per_row));
        let size = if let Some(bytes_per_row) = bytes_per_row { extent_height * bytes_per_row } else { extent_width * extent_height * 4 };
        let data: ImageTexture = ImageTexture {
            width, height, size: size as usize, texture, format, view_dimension: dimension, is_opacity
        };
        Self {
            data, extend: vec![]
        }
    }

    pub fn update(&self, queue: &RenderQueue, xoffset: u32, yoffset: u32, width: u32, height: u32, depth_or_array_layers: u32, aspect: Option<wgpu::TextureAspect>, data: &[u8], dataoffset: u64) {
        ResImageTexture::update_texture(&self.data.texture, queue, xoffset, yoffset, width, height, depth_or_array_layers, aspect, data, dataoffset);
    }
    pub fn create_texture(
        device: &RenderDevice, key: &KeyImageTexture, width: u32, height: u32,
        format: wgpu::TextureFormat, dimension: wgpu::TextureDimension, depth_or_array_layers: u32
    ) -> wgpu::Texture {
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };

        let texture = (**device).create_texture(&wgpu::TextureDescriptor {
            label: Some(key.url.as_str()),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension,
            format,
            usage: key.useage,
            view_formats: &[],
        });

        texture
    }
    pub fn update_texture(texture: &wgpu::Texture, queue: &RenderQueue, xoffset: u32, yoffset: u32, width: u32, height: u32, depth_or_array_layers: u32, aspect: Option<wgpu::TextureAspect>, data: &[u8], dataoffset: u64) {
        let offset = dataoffset;
        let (mut extent_width, mut _extent_height) = texture.format().block_dimensions();
        extent_width    = width / extent_width;
        // extent_height   = height / extent_height;
        let bytes_per_row = if let Some(pre_pixel_size) = texture.format().block_copy_size(aspect) {
            Some(extent_width * pre_pixel_size)
        } else { None };

        let mut temp = texture.as_image_copy();
        temp.origin.x = xoffset;
        temp.origin.y = yoffset;
        queue.write_texture(temp, data, wgpu::ImageDataLayout { offset, bytes_per_row, rows_per_image: None  }, wgpu::Extent3d { width, height, depth_or_array_layers });
    }
    pub fn update_sub(
        texture: &wgpu::Texture, queue: &RenderQueue,
        origin: wgpu::Origin3d,
        width: u32, height: u32, depth_or_array_layers: u32,
        aspect: Option<wgpu::TextureAspect>, data: &[u8], dataoffset: u64
    ) {
        let offset = dataoffset;
        let (mut extent_width, mut _extent_height) = texture.format().block_dimensions();
        extent_width    = width / extent_width;
        // extent_height   = height / extent_height;
        let bytes_per_row = if let Some(pre_pixel_size) = texture.format().block_copy_size(aspect) {
            Some(extent_width * pre_pixel_size)
        } else { None };

        let mut temp = texture.as_image_copy();
        temp.origin = origin;
        queue.write_texture(temp, data, wgpu::ImageDataLayout { offset, bytes_per_row, rows_per_image: None  }, wgpu::Extent3d { width, height, depth_or_array_layers });
    }

    pub fn width(&self) -> u32 {
        self.data.width
    }
    pub fn height(&self) -> u32 {
        self.data.height
    }
    pub fn image(&self) -> &ImageTexture {
        &self.data
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
