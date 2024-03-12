
use pi_assets::{asset::{Asset, Garbageer, Handle, Size}, mgr::{LoadResult, Receiver}};
use pi_atom::Atom;
use pi_futures::BoxFuture;
use pi_hal::{
    image::DynamicImage,
    loader::AsyncLoader,
};
use pi_hash::XHashMap;
use ktx::KtxInfo;
use wgpu::{AstcBlock, AstcChannel, util::DeviceExt};

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


#[derive(Debug)]
pub struct ImageTexture {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) texture: wgpu::Texture,
    #[allow(dead_code)]
    pub(crate) is_opacity: bool,
    pub(crate) size: usize,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) dimension: wgpu::TextureViewDimension,
    pub extend: Vec<u8>,
}

impl ImageTexture {
    pub fn new(width: u32, height: u32, size: usize, texture: wgpu::Texture, format: wgpu::TextureFormat, dimension: wgpu::TextureViewDimension, is_opacity: bool) -> Self {
        Self { width, height, size, texture, is_opacity, format, dimension, extend: vec![] }
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

        Self::new(width, height, size, texture, format, dimension, is_opacity)
    }

    pub fn update(&self, queue: &RenderQueue, data: &[u8], xoffset: u32, yoffset: u32, width: u32, height: u32) {
        let offset = (yoffset * self.width + xoffset) as u64;
        let temp = self.texture.as_image_copy();
        queue.write_texture(temp, data, wgpu::ImageDataLayout { offset, bytes_per_row: None, rows_per_image: None  }, wgpu::Extent3d { width, height, depth_or_array_layers: 1 });
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Asset for ImageTexture {
    type Key = KeyImageTexture;
    // const TYPE: &'static str = "ImageTexture";
}
impl Size for ImageTexture {
    fn size(&self) -> usize {
        self.size
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

impl ImageTexture {
    pub fn async_load_image<'a, G: Garbageer<Self>>(desc: ImageTexture2DDesc, result: LoadResult<'a, Self, G>) -> BoxFuture<'a, Result<Handle<Self>, ErrorImageTexture>> {
        Box::pin(async move { 
            match result {
                LoadResult::Ok(r) => Ok(r),
                LoadResult::Wait(f) => match f.await {
                    Ok(result) => Ok(result),
                    Err(_err) => Err(ErrorImageTexture::LoadFail),
                },
                LoadResult::Receiver(recv) => {
                    match pi_hal::image::load_from_url( &desc.url.url ).await {
                        Ok(image) => create_image_texture_from_image(&image, desc, recv).await,
                        Err(_e) => Err(ErrorImageTexture::LoadFail),
                    }
                }
            }
        })
    }
    pub fn async_load_compressed<'a, G: Garbageer<Self>>(desc: ImageTexture2DDesc, result: LoadResult<'a, Self, G>) -> BoxFuture<'a, Result<Handle<Self>, ErrorImageTexture>> {
        log::error!("{:?}", desc.url.url);
        Box::pin(async move { 
            match result {
                LoadResult::Ok(r) => Ok(r),
                LoadResult::Wait(f) => match f.await {
                    Ok(result) => Ok(result),
                    Err(_err) => Err(ErrorImageTexture::LoadFail),
                },
                LoadResult::Receiver(recv) => {
                    match pi_hal::file::load_from_url( &desc.url.url ).await {
                        Ok(data) => create_texture_compressed(desc, &data, recv).await ,
                        Err(_) => Err(ErrorImageTexture::LoadFail),
                    }
                }
            }
        })
    }
}

pub async fn create_image_texture_from_image<G: Garbageer<ImageTexture>>(
    image: &DynamicImage, 
    desc: ImageTexture2DDesc,
    recv: Receiver<ImageTexture, G>
) -> Result<Handle<ImageTexture>, ErrorImageTexture> {
    let temp_rgba8;
    let temp_rgba16;
    let temp_rgba32;
    let width;
    let height;
    let buffer;
    let format;
    let dimension = wgpu::TextureDimension::D2;
    let pre_pixel_size;
    let is_opacity = true;
    
    match image {
        DynamicImage::ImageLuma8(data)     => {
            width = data.width(); height = data.height(); buffer = bytemuck::cast_slice(data.as_raw()); pre_pixel_size = 1 * 1; format = wgpu::TextureFormat::R8Unorm;
        },
        DynamicImage::ImageLumaA8(data)     => {
            width = data.width(); height = data.height(); buffer = bytemuck::cast_slice(data.as_raw()); pre_pixel_size = 2 * 1; format = wgpu::TextureFormat::Rg8Unorm;
        },
        DynamicImage::ImageRgb8(data)         => {
            temp_rgba8 = image.to_rgba8();
            width = data.width(); height = data.height(); buffer = bytemuck::cast_slice(temp_rgba8.as_raw()); pre_pixel_size = 4 * 1; format = if desc.url.srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm };
        },
        DynamicImage::ImageRgba8(data)     => {
            width = data.width(); height = data.height(); buffer = bytemuck::cast_slice(data.as_raw()); pre_pixel_size = 4 * 1; format = if desc.url.srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm };
        },
        DynamicImage::ImageLuma16(data)     => {
            width = data.width(); height = data.height(); buffer = bytemuck::cast_slice(data.as_raw()); pre_pixel_size = 1 * 2; format = wgpu::TextureFormat::R16Unorm;
        },
        DynamicImage::ImageLumaA16(data)     => {
            width = data.width(); height = data.height(); buffer = bytemuck::cast_slice(data.as_raw()); pre_pixel_size = 2 * 2; format = wgpu::TextureFormat::Rg16Unorm;
        },
        DynamicImage::ImageRgb16(data)     => {
            temp_rgba16 = image.to_rgba16();
            width = data.width(); height = data.height(); buffer = bytemuck::cast_slice(temp_rgba16.as_raw()); pre_pixel_size = 4 * 2; format = wgpu::TextureFormat::Rgba16Unorm;
        },
        DynamicImage::ImageRgba16(data)     => {
            width = data.width(); height = data.height(); buffer = bytemuck::cast_slice(data.as_raw()); pre_pixel_size = 4 * 2; format = wgpu::TextureFormat::Rgba16Unorm;
        },
        DynamicImage::ImageRgb32F(data)     => {
            temp_rgba32 = image.to_rgba32f();
            width = data.width(); height = data.height(); buffer = bytemuck::cast_slice(temp_rgba32.as_raw()); pre_pixel_size = 4 * 4; format = wgpu::TextureFormat::Rgba32Float;
        },
        DynamicImage::ImageRgba32F(data)     => {
            width = data.width(); height = data.height(); buffer = bytemuck::cast_slice(data.as_raw()); pre_pixel_size = 4 * 4; format = wgpu::TextureFormat::Rgba32Float;
        },
        _ => return Err(ErrorImageTexture::FormatError),
    };

    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = (**desc.device).create_texture(&wgpu::TextureDescriptor {
        label: Some(desc.url.url.as_str()),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension,
        format,
        usage: desc.url.useage,
        view_formats: &[],
    });

    desc.queue.write_texture(
        texture.as_image_copy(),
        buffer,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(width * pre_pixel_size),
            rows_per_image: None,
        },
        texture_extent,
    );

    let dimension = wgpu::TextureViewDimension::D2;

    match recv.receive(desc.url, Ok(ImageTexture::new(width, height, (width * height * pre_pixel_size) as usize, texture, format, dimension, is_opacity))).await {
        Ok(result) => Ok(result),
        Err(_) => Err(ErrorImageTexture::CreateError),
    }
}

pub async fn create_texture_compressed<G: Garbageer<ImageTexture>>(
    desc: ImageTexture2DDesc,
    buffer: &[u8], 
    recv: Receiver<ImageTexture, G>
) -> Result<Handle<ImageTexture>, ErrorImageTexture> {

    match ktx::Decoder::new(buffer) {
        Ok(decoder) => {
            match convert_format(decoder.gl_internal_format()) {
                Ok(format) => {
                    let mut depth_or_array_layers = decoder.header().pixel_depth();
                    let textures = decoder.read_textures();
                    let mut texturelen = 0;
                    let mut data = vec![];
                    textures.into_iter().for_each(|v| {
                        texturelen += 1;
                        v.into_iter().for_each(|v| { data.push(v) });
                    });
                    let mut dimension = wgpu::TextureDimension::D3;
                    let mut viewdimension = wgpu::TextureViewDimension::D3;
                    if depth_or_array_layers == 0 {
                        dimension = wgpu::TextureDimension::D2;
                        depth_or_array_layers = decoder.header().faces();
                        viewdimension = wgpu::TextureViewDimension::D2;
                        if 1 < texturelen {
                            viewdimension = wgpu::TextureViewDimension::D2Array;
                        }
                        if depth_or_array_layers == 6 {
                            viewdimension = wgpu::TextureViewDimension::Cube;
                            if 1 < texturelen {
                                viewdimension = wgpu::TextureViewDimension::CubeArray;
                            }
                        }
                    }
                    let mip_level_count = decoder.mipmap_levels();
                    let width = decoder.pixel_width();
                    let height = decoder.pixel_height();
                    let texture_extent = wgpu::Extent3d { width, height, depth_or_array_layers, }.physical_size(format);
                    let is_opacity = true;


                    let texture = (**desc.device).create_texture_with_data(&desc.queue, &wgpu::TextureDescriptor {
                        label: Some(desc.url.url.as_str()),
                        size: texture_extent,
                        mip_level_count, // TODO
                        sample_count: 1,
                        dimension,
                        format,
                        usage: desc.url.useage,
                        view_formats: &[],
                    }, Default::default(), data.as_slice());
                    
                    match recv.receive(desc.url, Ok(ImageTexture::new(width, height, buffer.len() as usize, texture, format, viewdimension, is_opacity))).await {
                        Ok(result) => Ok(result),
                        Err(_) => Err(ErrorImageTexture::CreateError),
                    }
                },
                Err(_) => Err(ErrorImageTexture::FormatError),
            }
        },
        Err(_) => Err(ErrorImageTexture::LoadFail),
    }
}

fn convert_format(v: u32) -> Result<wgpu::TextureFormat, ErrorImageTexture> {
    match v {
        // // GL_COMPRESSED_RGB_S3TC_DXT1_EXT    0x83f0     GL_COMPRESSED_RGB_S3TC_DXT1_EXT    Bc1RgbUnorm
        // 0x83f0 => Ok(wgpu::TextureFormat::Bc1RgbUnorm),
        // GL_COMPRESSED_RGBA_S3TC_DXT1_EXT    0x83f1     GL_COMPRESSED_RGBA_S3TC_DXT1_EXT    Bc1RgbaUnorm
        0x83f1 => Ok(wgpu::TextureFormat::Bc1RgbaUnorm),
        // GL_COMPRESSED_RGBA_S3TC_DXT3_EXT    0x83f2     GL_COMPRESSED_RGBA_S3TC_DXT3_EXT    Bc2RgbaUnorm
        0x83f2 => Ok(wgpu::TextureFormat::Bc2RgbaUnorm),
        // GL_COMPRESSED_RGBA_S3TC_DXT5_EXT    0x83f3     GL_COMPRESSED_RGBA_S3TC_DXT5_EXT    Bc3RgbaUnorm
        0x83f3 => Ok(wgpu::TextureFormat::Bc3RgbaUnorm),
        // GL_COMPRESSED_RGB8_ETC2    0x9274             GL_COMPRESSED_RGB8_ETC2    Etc2Rgb8Unorm
        0x9274 => Ok(wgpu::TextureFormat::Etc2Rgb8Unorm),
        // GL_COMPRESSED_RGBA8_ETC2_EAC    0x9278         GL_COMPRESSED_RGBA8_ETC2_EAC    Etc2Rgba8Unorm
        0x9278 => Ok(wgpu::TextureFormat::Etc2Rgba8Unorm),
            
        // // GL_COMPRESSED_RGB_PVRTC_4BPPV1_IMG    0x8c00  GL_COMPRESSED_RGB_PVRTC_4BPPV1_IMG    PvrtcRgb4bppUnorm 
        // 0x8c00 => wgpu::TextureFormat::Bc1RgbaUnorm,
        // // GL_COMPRESSED_RGB_PVRTC_2BPPV1_IMG    0x8c01 GL_COMPRESSED_RGB_PVRTC_2BPPV1_IMG    PvrtcRgb2bppUnorm 
        // 0x8c01 => wgpu::TextureFormat::Bc1RgbaUnorm,
        // // GL_COMPRESSED_RGBA_PVRTC_4BPPV1_IMG    0x8c02 UnormGL_COMPRESSED_RGBA_PVRTC_4BPPV1_IMG    PvrtcRgba4bppUnorm
        // 0x8c02 => wgpu::TextureFormat::Bc1RgbaUnorm,
        // // GL_COMPRESSED_RGBA_PVRTC_2BPPV1_IMG    0x8c03 GL_COMPRESSED_RGBA_PVRTC_2BPPV1_IMG    PvrtcRgba2bppUnorm 
        // 0x8c03 => wgpu::TextureFormat::Bc1RgbaUnorm,
        
        // GL_COMPRESSED_RGBA_ASTC_4x4_KHR    0x93b0     GL_COMPRESSED_RGBA_ASTC_4x4_KHR    Astc4x4Unorm 
        0x93b0 => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B4x4, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_5x4_KHR    0x93b1     GL_COMPRESSED_RGBA_ASTC_5x4_KHR    Astc5x4Unorm 
        0x93b1 => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B5x4, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_5x5_KHR    0x93b2     GL_COMPRESSED_RGBA_ASTC_5x5_KHR    Astc5x5Unorm
        0x93b2 => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B5x5, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_6x5_KHR    0x93b3     GL_COMPRESSED_RGBA_ASTC_6x5_KHR    Astc6x5Unorm 
        0x93b3 => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B6x5, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_6x6_KHR    0x93b4     GL_COMPRESSED_RGBA_ASTC_6x6_KHR    Astc6x6Unorm 
        0x93b4 => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B6x6, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_8x5_KHR    0x93b5     GL_COMPRESSED_RGBA_ASTC_8x5_KHR    Astc8x5Unorm 
        0x93b5 => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B8x5, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_8x6_KHR    0x93b6     GL_COMPRESSED_RGBA_ASTC_8x6_KHR    Astc8x6Unorm 
        0x93b6 => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B8x6, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_8x8_KHR    0x93b7     GL_COMPRESSED_RGBA_ASTC_8x8_KHR    Astc8x8Unorm 
        0x93b7 => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B8x8, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_10x5_KHR    0x93b8     GL_COMPRESSED_RGBA_ASTC_10x5_KHR    Astc10x5Unorm 
        0x93b8 => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B10x5, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_10x6_KHR    0x93b9     GL_COMPRESSED_RGBA_ASTC_10x6_KHR    Astc10x6Unorm 
        0x93b9 => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B10x6, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_10x8_KHR    0x93ba GL_COMPRESSED_RGBA_ASTC_10x8_KHR    Astc10x8Unorm  
        0x93ba => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B10x8, channel: AstcChannel::Unorm }),
        //  GL_COMPRESSED_RGBA_ASTC_10x10_KHR    0x93bb     GL_COMPRESSED_RGBA_ASTC_10x10_KHR    Astc10x10Unorm 
        0x93bb => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B10x10, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_12x10_KHR    0x93bc     GL_COMPRESSED_RGBA_ASTC_12x10_KHR    Astc12x10 
        0x93bc => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B12x10, channel: AstcChannel::Unorm }),
        // GL_COMPRESSED_RGBA_ASTC_12x12_KHR    0x93bd     GL_COMPRESSED_RGBA_ASTC_12x12_KHR    Astc12x12Unorm
        0x93bd => Ok(wgpu::TextureFormat::Astc { block: AstcBlock::B12x12, channel: AstcChannel::Unorm }),
        _ => Err(ErrorImageTexture::FormatError),
    }
}