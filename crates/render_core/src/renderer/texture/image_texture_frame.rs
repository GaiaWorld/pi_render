use std::{sync::Arc, ops::Deref};

use crossbeam::queue::SegQueue;
use guillotiere::{AllocId, AllocatorOptions, AtlasAllocator};
use ktx::KtxInfo;
use pi_assets::{asset::{Handle, Asset, Size, Garbageer}, mgr::LoadResult};
use pi_atom::Atom;
use pi_futures::BoxFuture;
use pi_hal::{image::DynamicImage, texture::ImageTexture};
use pi_share::Share;
use wgpu::TextureView;

use crate::{asset::TAssetKeyU64, renderer::buildin_data::DefaultTexture, rhi::{device::RenderDevice, sampler::SamplerDesc, RenderQueue}};

use super::TextureViewDesc;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyImageTextureFrame {
    /// 路径
    pub url: Atom,
    /// 是否从文件加载
    pub file: bool,
    /// 是否压缩纹理
    pub compressed: bool,
    pub cancombine: bool,
}
impl Default for KeyImageTextureFrame {
    fn default() -> Self {
        Self {
            url: Atom::from(DefaultTexture::BLACK_2D), file: false, compressed: false, cancombine: false,
        }
    }
}
impl TAssetKeyU64 for KeyImageTextureFrame {}
impl std::ops::Deref for KeyImageTextureFrame {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.url.as_str()
    }
}

pub struct TextureFrame {
    id: AllocId,
    rect: (u16, u16, u16, u16, u16, u16),
    idx: usize,
    seq: Share<SegQueue<(usize, AllocId)>>,
}
impl TextureFrame {
    
    pub fn tilloff(&self, result: &mut [f32], offset: usize) {
        result[offset + 0] = self.rect.2 as f32 / self.rect.4 as f32;
        result[offset + 1] = self.rect.3 as f32 / self.rect.5 as f32;
        result[offset + 2] = self.rect.0 as f32 / self.rect.4 as f32;
        result[offset + 3] = self.rect.1 as f32 / self.rect.5 as f32;
    }
    pub fn texture_coord(&self) -> usize {
        self.idx
    }
    pub fn copy_dst_orign(&self, format: wgpu::TextureFormat) -> wgpu::Origin3d {
        let (blockw, blockh) = format.block_dimensions();
        wgpu::Origin3d {
            x: (self.rect.0 as u32 + blockw - 1) / blockw * blockw,
            y: (self.rect.1 as u32 + blockh - 1) / blockh * blockh,
            z: self.idx as u32,
        }
    }
    pub fn copy_rect(&self, format: wgpu::TextureFormat) -> wgpu::Extent3d {
        let (blockw, blockh) = format.block_dimensions();
        wgpu::Extent3d {
            width:  (self.rect.4 as u32 + blockw - 1) / blockw * blockw,
            height: (self.rect.5 as u32 + blockh - 1) / blockh * blockh,
            depth_or_array_layers: 1,
        }
    }
    pub fn copy_extent(&self, format: wgpu::TextureFormat) -> wgpu::Extent3d {
        let (blockw, blockh) = format.block_dimensions();
        wgpu::Extent3d {
            width:  (self.rect.2 as u32 + blockw - 1) / blockw,
            height: (self.rect.3 as u32 + blockh - 1) / blockh,
            depth_or_array_layers: 1,
        }
    }
}
impl Drop for TextureFrame {
    fn drop(&mut self) {
        self.seq.push((self.idx, self.id));
    }
}

pub struct ImageTextureFrame {
    frame: Option<TextureFrame>,
    size: usize,
    pub(crate) tex: Arc<ImageTexture>,
    pub extend: Vec<u8>,
}
impl ImageTextureFrame {
    pub fn new(tex: ImageTexture) -> Self {
        Self { frame: None, size: tex.size, tex: Arc::new(tex), extend: vec![] }
    }
    pub fn tilloff(&self) -> [f32;4] {
        if let Some(frame) = &self.frame {
            [
                frame.rect.2 as f32 / frame.rect.4 as f32,
                frame.rect.3 as f32 / frame.rect.5 as f32,
                frame.rect.0 as f32 / frame.rect.4 as f32,
                frame.rect.1 as f32 / frame.rect.5 as f32,
            ]
        } else {
            [1., 1., 0., 0.]
        }
    }
    pub fn texture(&self) -> &ImageTexture {
        &self.tex
    }
    pub fn update_texture(&self, queue: &RenderQueue, data: &[u8]) {
        if let Some(frame) = &self.frame {
            let format = self.tex.texture.format();
            let (blockw, blockh) = self.tex.texture.format().block_dimensions();
            // log::error!("{:?}", (frame.copy_dst_orign(format), (frame.rect.4 as u32 + blockw - 1) / blockw, (frame.rect.5 as u32 + blockh - 1) / blockh));
            ImageTextureFrame::update_sub(&self.tex.texture, queue, frame.copy_dst_orign(format),
                (frame.rect.2 as u32 + blockw - 1) / blockw, (frame.rect.3 as u32 + blockh - 1) / blockh,
                1, None, data, 0
            );
        }
    }
    pub fn create_image(
        device: &RenderDevice, queue: &RenderQueue, key: &Atom,
        dimension: wgpu::TextureViewDimension,
        data: DynamicImage,
    ) -> Option<ImageTexture> {
        let width = data.width();
        let height = data.height();
        let depth_or_array_layers = 1;
        let mut temprgb = None;
        let mut temprgb16 = None;
        let mut temprgb32 = None;
        let (image_buffer, format) = match &data {
            DynamicImage::ImageLuma8(image_buffer) => {
                (image_buffer.as_raw().as_slice(), wgpu::TextureFormat::R8Unorm)
            },
            DynamicImage::ImageLumaA8(image_buffer) => {
                (image_buffer.as_raw().as_slice(), wgpu::TextureFormat::R8Unorm)
            },
            DynamicImage::ImageRgb8(image_buffer) => {
                temprgb = Some(data.to_rgba8());
                (temprgb.as_ref().unwrap().as_raw().as_slice(), wgpu::TextureFormat::Rgba8Unorm)
            },
            DynamicImage::ImageRgba8(image_buffer) => {
                (image_buffer.as_raw().as_slice(), wgpu::TextureFormat::Rgba8Unorm)
            },
            DynamicImage::ImageLuma16(image_buffer) => {
                (bytemuck::cast_slice(image_buffer.as_raw()), wgpu::TextureFormat::R16Unorm)
            },
            DynamicImage::ImageLumaA16(image_buffer) => {
                (bytemuck::cast_slice(image_buffer.as_raw()), wgpu::TextureFormat::R16Unorm)
            },
            DynamicImage::ImageRgb16(image_buffer) => {
                temprgb16 = Some(data.to_rgba16());
                (bytemuck::cast_slice(temprgb16.as_ref().unwrap().as_raw()), wgpu::TextureFormat::Rgba16Unorm)
            },
            DynamicImage::ImageRgba16(image_buffer) => {
                (bytemuck::cast_slice(image_buffer.as_raw()), wgpu::TextureFormat::Rgba16Unorm)
            },
            DynamicImage::ImageRgb32F(image_buffer) => {
                temprgb32 = Some(data.to_rgba32f());
                (bytemuck::cast_slice(temprgb32.as_ref().unwrap().as_raw()), wgpu::TextureFormat::Rgba32Float)
            },
            DynamicImage::ImageRgba32F(image_buffer) => {
                (bytemuck::cast_slice(image_buffer.as_raw()), wgpu::TextureFormat::Rgba32Float)
            },
            _ => return None,
        };
        let texture = ImageTextureFrame::create_texture(device, key, width, height, format, dimension.compatible_texture_dimension(), depth_or_array_layers);

        let (block_width, block_height) = format.block_dimensions();
        let extent_width    = (width  + block_width  - 1) / block_width;
        let extent_height   = (height + block_height - 1) / block_height;
        let bytes_per_row = if let Some(pre_pixel_size) = format.block_copy_size(None) {
            Some(extent_width * pre_pixel_size)
        } else { None };

        let offset = 0;
        let texture_extent = wgpu::Extent3d {
            width: extent_width,
            height: extent_height,
            depth_or_array_layers,
        };
        queue.write_texture(
            texture.as_image_copy(),
            image_buffer,
            wgpu::ImageDataLayout {
                offset,
                bytes_per_row,
                rows_per_image: None,
            },
            texture_extent,
        );

        // log::error!("{:?}", (key, width, height, format, dimension, depth_or_array_layers, block_width, block_height, extent_width, extent_height, bytes_per_row));
        let size = if let Some(bytes_per_row) = bytes_per_row { extent_height * bytes_per_row } else { extent_width * extent_height * 4 };
        Some(ImageTexture {
            width, height, size: size as usize, texture, format, view_dimension: dimension, is_opacity: true
        })
    }
    pub fn create_ktx(
        device: &RenderDevice, queue: &RenderQueue, key: &Atom,
        dimension: wgpu::TextureViewDimension,
        format: wgpu::TextureFormat,
        ktx: &ktx::Ktx<&[u8]>,
    ) -> Option<ImageTexture> {
        let width = ktx.pixel_width();
        let height = ktx.pixel_height();
        let mipmaps = ktx.mipmap_levels();
        
        if ktx.textures().count() == 0 || ktx.textures().count() > 1 || ktx.faces() > 1 || ktx.pixel_depth() > 1 {
            return None;
        }

        for data in ktx.textures() {
            return Some(ImageTextureFrame::create_data_texture(device, queue, &key, width, height, format, dimension, true, 1, None, Some(data), 0));
        }
        None
    }
    pub fn create_data_texture(
        device: &RenderDevice, queue: &RenderQueue, key: &Atom, width: u32, height: u32,
        format: wgpu::TextureFormat, dimension: wgpu::TextureViewDimension, is_opacity: bool, depth_or_array_layers: u32, aspect: Option<wgpu::TextureAspect>,
        data: Option<&[u8]>, dataoffset: u64
    ) -> ImageTexture {
        let texture = ImageTextureFrame::create_texture(device, key, width, height, format, dimension.compatible_texture_dimension(), depth_or_array_layers);

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
        ImageTexture {
            width, height, size: size as usize, texture, format, view_dimension: dimension, is_opacity
        }
    }
    pub fn create_texture(
        device: &RenderDevice, key: &Atom, width: u32, height: u32,
        format: wgpu::TextureFormat, dimension: wgpu::TextureDimension, depth_or_array_layers: u32
    ) -> wgpu::Texture {
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers,
        };

        let texture = (**device).create_texture(&wgpu::TextureDescriptor {
            label: Some(key.as_str()),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        texture
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
        self.tex.width
    }
    pub fn height(&self) -> u32 {
        self.tex.height
    }
    pub fn coord(&self) -> u32 {
        if let Some(frame) = &self.frame {
            frame.idx as u32
        } else {
            0
        }
    }
}

impl Size for ImageTextureFrame {
    fn size(&self) -> usize {
        self.size
    }
}
impl Asset for ImageTextureFrame {
    type Key = KeyImageTextureFrame;
}

pub struct Atlas {
    maxwidth: u32,
    maxheight: u32,
    allocator: Vec<AtlasAllocator>,
    format: wgpu::TextureFormat,
    texture: Arc<ImageTexture>,
    key: Atom,
    recycle: Share<SegQueue<(usize, AllocId)>>,
}
impl Atlas {
    ///
    /// maxcount 可容纳图块的数目最大值, 根据 bindbuffer
    pub fn new(key: Atom, maxwidth: u32, maxheight: u32, depth_or_array_layers: u32, format: wgpu::TextureFormat,
        device: &RenderDevice, queue: &RenderQueue,
    ) -> Self {
        let dimension = wgpu::TextureViewDimension::D2Array;
        let aspect = None;
        let data = None;
        let dataoffset = 0;
        let texture = ImageTextureFrame::create_data_texture(device, queue, &key, maxwidth, maxheight, format, dimension, true, depth_or_array_layers, aspect, data, dataoffset);
        let (blockw, blockh) = format.block_dimensions();

        let mut allocator = Vec::with_capacity(depth_or_array_layers as usize);
        for _ in 0..depth_or_array_layers {
            allocator.push(AtlasAllocator::with_options(
                guillotiere::Size { width: maxwidth as i32, height: maxheight as i32, ..Default::default() },
                &AllocatorOptions {
                    alignment: guillotiere::Size { width: blockw as i32, height: blockh as i32, ..Default::default() },
                    small_size_threshold: maxwidth as i32 / 16,
                    large_size_threshold: maxwidth as i32 /  4,
                }
            ));
        }
        let recycle = Share::new(SegQueue::default());
        Self {
            maxwidth,
            maxheight,
            allocator,
            format,
            key,
            texture: Arc::new(texture),
            recycle,
        }
    }
    pub fn allocate(&mut self, width: u32, height: u32) -> Option<ImageTextureFrame> {
        let mut result = None;
        while let Some((idx, id))  = self.recycle.pop() {
            if let Some(allocator) = self.allocator.get_mut(idx) {
                allocator.deallocate(id);
            }
        }
        let mut idx = 0;
        for allocator in self.allocator.iter_mut() {
            if let Some(rect) = allocator.allocate(guillotiere::Size { width: width as i32, height: height as i32, ..Default::default() }) {
                // log::error!("Alloc: {:?}", (width, height, rect.rectangle.min.x, rect.rectangle.min.y));
                let ox = rect.rectangle.min.x as u16;
                let oy = rect.rectangle.min.y as u16;
                let sx = width  as u16;
                let sy = height as u16;
                let w = self.maxwidth  as u16;
                let h = self.maxheight as u16;
                let format = self.texture.texture.format();
                let (blockw, blockh) = format.block_dimensions();
                let blocksize = if let Some(size) = format.block_copy_size(None) {
                    size
                } else { 1 };
                result = Some(ImageTextureFrame {
                    frame: Some(TextureFrame {idx,
                        id: rect.id,
                        seq: self.recycle.clone(),
                        rect: (ox, oy, sx, sy, w, h),
                    }),
                    tex: self.texture.clone(),
                    size: (blocksize * (width + blockw - 1) / blockw * (height + blockh - 1) / blockh) as usize,
                    extend: vec![],
                });
                break;
            } else {
                // log::error!("Alloc Error");
            }
            idx += 1;
        }
        result
    }
}

pub struct CombineAtlas2DMgr {
    atlasarr: Vec<Atlas>,
    format: wgpu::TextureFormat,
    maxcount: usize,
    maxlayer: u32,
    maxsize: u32,
}
impl CombineAtlas2DMgr {
    pub fn new(
        device: &RenderDevice, 
        format: wgpu::TextureFormat,
        maxlayer: u32,
        maxsize: u32,
        maxcount: usize,
    ) -> Self {
        let limit = device.limits();
        let maxsize  = maxsize.min(limit.max_texture_dimension_2d);
        let maxlayer = maxlayer.min(limit.max_texture_array_layers);
        Self { atlasarr: vec![], format, maxcount, maxlayer, maxsize }
    }
    pub fn combine(&mut self,
        format: wgpu::TextureFormat,
        width: u32, height: u32,
        device: &RenderDevice, queue: &RenderQueue
    ) -> Option<ImageTextureFrame> {
        if self.format == format {
            let mut idx = 0;
            let mut frame = None;
            for atlas in self.atlasarr.iter_mut() {
                if let Some(val) = atlas.allocate(width, height) {
                    frame = Some(val);
                    break;
                }
                idx += 1;
            }
            if frame.is_none() && idx < self.maxcount {
                let key = self.key() + &idx.to_string();
                let key = Atom::from(key);
                let mut atlas = Atlas::new(key, self.maxsize, self.maxsize, self.maxlayer, self.format, device, queue);
                frame = atlas.allocate(width, height);
                self.atlasarr.push(atlas);
            }
            frame
        } else {
            // log::error!("Combin Faile A");
            None
        }
    }
    fn key(&self) -> String {
        let mut result = String::from("Combine");
        result += "#";
        // result += &serde_json::to_string(&self.format).unwrap();
        // result += &serde_json::to_string(&self.dimesion).unwrap();
        // result += &serde_json::to_string(&self.sample_type).unwrap();
        // result += &serde_json::to_string(&self.sampler).unwrap();
        result
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyImageTextureViewFrame {
    pub(crate) tex: KeyImageTextureFrame,
    pub(crate) desc: TextureViewDesc,
}
impl TAssetKeyU64 for KeyImageTextureViewFrame {}
impl KeyImageTextureViewFrame {
    pub fn new(tex: KeyImageTextureFrame, desc: TextureViewDesc) -> Self {
        Self { tex, desc }
    }
    pub fn url(&self) -> &KeyImageTextureFrame {
        &self.tex
    }
    pub fn view_desc(&self) -> &TextureViewDesc {
        &self.desc
    }
}

pub struct ImageTextureViewFrame {
    pub(crate) texture: Handle<ImageTextureFrame>,
    pub(crate) view: Share<TextureView>,
}
impl Asset for ImageTextureViewFrame {
    type Key = u64;
    // const TYPE: &'static str = "ImageTextureViewFrame";
}

impl Size for ImageTextureViewFrame {
    fn size(&self) -> usize {
        self.texture.size() + 64
    }
}
impl ImageTextureViewFrame {
    pub fn new(
        key: &KeyImageTextureViewFrame,
        texture: Handle<ImageTextureFrame>,
    ) -> Self {
        let view = texture.tex.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(key.url().deref()),
            format: Some(texture.tex.format.clone()),
            dimension: Some(texture.tex.view_dimension.clone()),
            aspect: wgpu::TextureAspect::All, // key.desc.aspect,
            base_mip_level: key.desc.base_mip_level as u32,
            mip_level_count: key.desc.mip_level_count(),
            base_array_layer: key.desc.base_array_layer as u32,
            array_layer_count: key.desc.array_layer_count(),
        });

        Self {
            texture,
            view: Share::new(view) ,
        }
    }
    pub fn texture(&self) -> &Handle<ImageTextureFrame> {
        &self.texture
    }
    pub fn async_load<'a, G: Garbageer<Self>>(image: Handle<ImageTextureFrame>, key: KeyImageTextureViewFrame, result: LoadResult<'a, Self, G>) -> BoxFuture<'a, Result<Handle<Self>, ()>> {
        Box::pin(async move {
            match result {
                LoadResult::Ok(r) => { Ok(r) },
                LoadResult::Wait(f) => {
                    // log::error!("ImageTexture Wait");
                    match f.await {
                        Ok(result) => Ok(result),
                        Err(_) => Err(()),
                    }
                },
                LoadResult::Receiver(recv) => {
                    let texture_view = Self::new(&key, image);
                    let key = key.asset_u64();
                    match recv.receive(key, Ok(texture_view)).await {
                        Ok(result) => { Ok(result) },
                        Err(_) => Err(()),
                    }
                }
            }
        })
    }
}

pub type EImageTextureViewUsage = Handle<ImageTextureViewFrame>;
// #[derive(Clone)]
// pub enum EImageTextureViewUsage {
//     Handle(Handle<ImageTextureViewFrame>),
//     Arc(Arc<ImageTextureViewFrame>),
// }
