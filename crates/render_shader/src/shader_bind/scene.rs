use std::num::NonZeroU64;

use super::TShaderBind;


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindSceneAboutCamera(pub u32, pub u32);
impl ShaderBindSceneAboutCamera {

    pub const OFFSET_VIEW_MATRIX:           wgpu::BufferAddress = 0;
    pub const OFFSET_PROJECT_MATRIX:        wgpu::BufferAddress = 16 * 4;
    pub const OFFSET_VIEW_PROJECT_MATRIX:   wgpu::BufferAddress = 16 * 4 + 16 * 4;
    pub const OFFSET_CAMERA_POSITION:       wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4;
    pub const OFFSET_CAMERA_DIRECTION:      wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4 + 4 * 4;
    
    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4 + 4 * 4 + 4 * 4;

}

impl TShaderBind for ShaderBindSceneAboutCamera {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.0,
                visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
                count: None,
            }
        )
    }
    fn bind(&self) -> u32 {
        self.0
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindSceneAboutTime(pub u32, pub u32);
impl ShaderBindSceneAboutTime {

    pub const OFFSET_TIME:                  wgpu::BufferAddress = 0;
    pub const OFFSET_DELTA_TIME:            wgpu::BufferAddress = 4 * 4;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 4 * 4 + 4 * 4;

}
impl TShaderBind for ShaderBindSceneAboutTime {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.0,
                visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
                count: None,
            }
        );
    }
    fn bind(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindSceneAboutFog(pub u32, pub u32);
impl ShaderBindSceneAboutFog {

    pub const OFFSET_FOG_INFO:              wgpu::BufferAddress = 0;
    pub const OFFSET_FOG_PARAM:             wgpu::BufferAddress = 4 * 4;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 4 * 4 + 4 * 4;
}
impl TShaderBind for ShaderBindSceneAboutFog {

    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        entries.push(
            wgpu::BindGroupLayoutEntry {
                binding: self.0,
                visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
                count: None,
            }
        );
    }
    fn bind(&self) -> u32 {
        self.0
    }
}
