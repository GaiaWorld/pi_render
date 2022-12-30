use std::num::NonZeroU32;

use render_core::rhi::{texture::Texture, device::RenderDevice, RenderQueue};

#[derive(Debug, Clone, Copy)]
pub enum EDataTextureFormat {
    RgbaF32,
    RgbaU8,
    RgbaU16,
}

pub struct DataTexture2D {
    tex: Texture,
    buff: Vec<u8>,
    size: wgpu::Extent3d,
    bytes_per_pixel: u32,
    kind: EDataTextureFormat,
}
impl DataTexture2D {
    fn new_tex(device: &RenderDevice, size: wgpu::Extent3d, format: wgpu::TextureFormat, usage: wgpu::TextureUsages) -> Texture {
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 0,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        });

        Texture::from(tex)
    }
    pub fn data(&self) -> &[u8] {
        &self.buff
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

        let size = wgpu::Extent3d { width: col, height, depth_or_array_layers: 0 };

        Self {
            tex: Self::new_tex(device, size, wgpu::TextureFormat::Rgba32Float, wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING),
            buff: vec![0; (col * width * bytes_per_pixel) as usize],
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

        Self {
            tex: Self::new_tex(device, size, wgpu::TextureFormat::Rgba8Unorm, wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING),
            buff: vec![0; (col * width * bytes_per_pixel) as usize],
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

        Self {
            tex: Self::new_tex(device, size, wgpu::TextureFormat::Rgba16Uint, wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING),
            buff: vec![0; (col * width * bytes_per_pixel) as usize],
            size,
            kind: EDataTextureFormat::RgbaU16,
            bytes_per_pixel,
        }
    }

    pub fn update_row(&mut self, row_index: u32, data: &[u8]) {
        let data_len = data.len();
        let row_len = (self.size.width * self.bytes_per_pixel) as usize;

        let update_len = data_len.min(row_len);

        let start = row_index as usize * row_len;
        let end = start + update_len;
        self.buff[start..end].clone_from_slice(&data[0..update_len]);
    }

    pub fn update_texture(&self, queue: &RenderQueue) {
        let texture = wgpu::ImageCopyTexture {
            texture: &self.tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        };

        let bytes_per_row = NonZeroU32::new(self.size.width * self.bytes_per_pixel);
        let rows_per_image = NonZeroU32::new(self.size.height);
        let data_layout = wgpu::ImageDataLayout { offset: 1, bytes_per_row, rows_per_image };
        queue.write_texture(texture, self.data(), data_layout, self.size);
    }
}