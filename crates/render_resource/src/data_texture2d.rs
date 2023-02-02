use std::num::NonZeroU32;

use render_core::rhi::{texture::{Texture, TextureView}, device::RenderDevice, RenderQueue};

#[derive(Debug, Clone, Copy)]
pub enum EDataTextureFormat {
    RgbaF32,
    RgbaU8,
    RgbaU16,
}

pub struct DataTexture2D {
    tex: Texture,
    view: TextureView,
    size: wgpu::Extent3d,
    bytes_per_pixel: u32,
    kind: EDataTextureFormat,
}
impl DataTexture2D {
    fn new_tex(device: &RenderDevice, size: wgpu::Extent3d, format: wgpu::TextureFormat, usage: wgpu::TextureUsages) -> Texture {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        });

        Texture::from(tex)
    }
    pub fn size(&self) -> wgpu::Extent3d {
        self.size.clone()
    }
    pub fn create_data(&self) -> Vec<u8> {
        let channel = 4;
        let byte_per_channel = 4;
        let bytes_per_pixel = channel * byte_per_channel;

        vec![0; (self.size.width * self.size.height * bytes_per_pixel) as usize]
    }
    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.view
    }
    pub fn kind(&self) -> EDataTextureFormat {
        self.kind
    }
    pub fn new_rgba_f32(device: &RenderDevice, width: u32, height: u32) -> Self {
        // 256 为 buffer 对齐要求 -
        // Must be a multiple of 256 for [`CommandEncoder::copy_buffer_to_texture`][CEcbtt]
        // and [`CommandEncoder::copy_texture_to_buffer`][CEcttb]. You must manually pad the
        // image such that this is a multiple of 256. It will not affect the image data.
        let channel = 4;
        let byte_per_channel = 4;
        let bytes_per_pixel = channel * byte_per_channel;
        let pixel_block_col = 256 / bytes_per_pixel;
        let mut col = width / pixel_block_col;
        col = col * pixel_block_col;
        col = if col < width { col + pixel_block_col } else { col };

        let size = wgpu::Extent3d { width: col, height, depth_or_array_layers: 1 };

        let tex = Self::new_tex(device, size, wgpu::TextureFormat::Rgba32Float, wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING);
        let view = tex.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: NonZeroU32::new(1),
        });

        Self {
            tex,
            view,
            size,
            kind: EDataTextureFormat::RgbaF32,
            bytes_per_pixel,
        }
    }

    pub fn new_rgba_u8(device: &RenderDevice, width: u32, height: u32) -> Self {
        let channel = 4;
        let byte_per_channel = 1;
        let bytes_per_pixel = channel * byte_per_channel;
        let pixel_block_col = 256 / bytes_per_pixel;
        let mut col = width / pixel_block_col;
        col = col * pixel_block_col;
        col = if col < width { col + pixel_block_col } else { col };

        let size = wgpu::Extent3d { width: col, height, depth_or_array_layers: 0 };

        let tex = Self::new_tex(device, size, wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING);
        let view = tex.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: NonZeroU32::new(0),
        });

        Self {
            tex,
            view,
            size,
            kind: EDataTextureFormat::RgbaU8,
            bytes_per_pixel,
        }
    }

    pub fn new_rgba_u16(device: &RenderDevice, width: u32, height: u32) -> Self {
        let channel = 4;
        let byte_per_channel = 2;
        let bytes_per_pixel = channel * byte_per_channel;
        let pixel_block_col = 256 / bytes_per_pixel;
        let mut col = width / pixel_block_col;
        col = col * pixel_block_col;
        col = if col < width { col + pixel_block_col } else { col };

        let size = wgpu::Extent3d { width: col, height, depth_or_array_layers: 0 };
        let tex = Self::new_tex(device, size, wgpu::TextureFormat::Rgba16Uint, wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING);
        let view = tex.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba16Uint),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: NonZeroU32::new(0),
        });

        Self {
            tex,
            view,
            size,
            kind: EDataTextureFormat::RgbaU16,
            bytes_per_pixel,
        }
    }

    pub fn update_row(&self, row_index: u32, row_data: &[u8], buff: &mut Vec<u8>) {
        let data_len = row_data.len();
        let row_len = (self.size.width * self.bytes_per_pixel) as usize;

        let update_len = data_len.min(row_len);

        let start = row_index as usize * row_len;
        let end = start + update_len;
        println!(">>>>>> {}, {}, {}", start, end, update_len);
        buff[start..end].clone_from_slice(&row_data[0..update_len]);
    }

    pub fn update_texture(&self, queue: &RenderQueue, data: &[u8]) {
        let texture = wgpu::ImageCopyTexture {
            texture: &self.tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };

        let bytes_per_row = NonZeroU32::new(self.size.width * self.bytes_per_pixel);
        let rows_per_image = NonZeroU32::new(self.size.height);
        let data_layout = wgpu::ImageDataLayout { offset: 0, bytes_per_row, rows_per_image };
        queue.write_texture(texture, data, data_layout, self.size);
    }
}