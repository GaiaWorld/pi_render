use wgpu::util::DeviceExt;

use crate::rhi::{device::RenderDevice, RenderQueue};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EDefaultTexture {
    Black,
    White,
}

pub struct DefaultTexture;
impl DefaultTexture {
    pub const BLACK: [u8;4] = [0, 0, 0, 0];
    pub const WHITE: [u8;4] = [255, 255, 255, 255];
    pub const BLACK_1D: &'static str = "BLACK_1D";
    pub const BLACK_2D: &'static str = "BLACK_2D";
    pub const BLACK_3D: &'static str = "BLACK_3D";
    pub const WHITE_1D: &'static str = "WHITE_1D";
    pub const WHITE_2D: &'static str = "WHITE_2D";
    pub const WHITE_3D: &'static str = "WHITE_3D";
    pub fn path(mode: EDefaultTexture, dim: wgpu::TextureDimension) -> &'static str {
        match mode {
            EDefaultTexture::Black => {
                match dim {
                    wgpu::TextureDimension::D1 => Self::BLACK_1D,
                    wgpu::TextureDimension::D2 => Self::BLACK_2D,
                    wgpu::TextureDimension::D3 => Self::BLACK_3D,
                }
            },
            EDefaultTexture::White => {
                match dim {
                    wgpu::TextureDimension::D1 => Self::WHITE_1D,
                    wgpu::TextureDimension::D2 => Self::WHITE_2D,
                    wgpu::TextureDimension::D3 => Self::WHITE_3D,
                }
            },
        }
    }
    pub fn create(device: &RenderDevice, queue: &RenderQueue, mode: EDefaultTexture, dim: wgpu::TextureDimension) -> wgpu::Texture {
        match mode {
            EDefaultTexture::Black => {
                Self::texture(device, queue, dim, Self::BLACK.as_slice())
            },
            EDefaultTexture::White => {
                Self::texture(device, queue, dim, Self::WHITE.as_slice())
            },
        }
    }
    pub fn texture(device: &RenderDevice, queue: &RenderQueue, dim: wgpu::TextureDimension, data: &[u8]) -> wgpu::Texture {
        device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: dim,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
				view_formats: &[],
            },
            data
        )
    }
}