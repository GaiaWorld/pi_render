use std::{num::NonZeroU64, sync::Arc};

use crate::unifrom_code::{MaterialValueBindDesc, TBindDescToShaderCode};

use super::TShaderBind;


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShaderBindEffectValue {
    bind: u32,
    pub total_size: usize,
    
    pub mat4_count: u8,
    pub mat2_count: u8,
    pub vec4_count: u8,
    pub vec2_count: u8,
    pub float_count: u8,
    pub int_count: u8,
    pub uint_count: u8,

    pub fill_vec2_count: u8,
    pub fill_int_count: u8,
    
    pub mat4_begin: u32,
    pub mat2_begin: u32,
    pub vec4_begin: u32,
    pub vec2_begin: u32,
    pub float_begin: u32,
    pub int_begin: u32,
    pub uint_begin: u32,
    effect: Arc<MaterialValueBindDesc>,
}
impl ShaderBindEffectValue {
    pub const BIND: u32 = 0;

    pub const LABEL_MASK: &'static str = "#";
    pub const MAT4_BYTES: u32 = 16 * 4;
    pub const MAT2_BYTES: u32 = 4 * 4;
    pub const VEC4_BYTES: u32 = 4 * 4;
    pub const VEC2_BYTES: u32 = 2 * 4;
    pub const FLOAT_BYTES: u32 = 1 * 4;
    pub const INT_BYTES: u32 = 1 * 4;
    pub const UINT_BYTES: u32 = 1 * 4;

    pub fn new(
        bind: u32,
        effect: Arc<MaterialValueBindDesc>,
    ) -> Self {
        let uniforms = &effect;
        let mat4_count      = uniforms.mat4_list.len() as u8;
        let mat2_count      = uniforms.mat2_list.len() as u8;
        let vec4_count      = uniforms.vec4_list.len() as u8;
        let vec2_count      = uniforms.vec2_list.len() as u8;
        let float_count     = uniforms.float_list.len() as u8;
        let int_count       = uniforms.int_list.len() as u8;
        let uint_count      = uniforms.uint_list.len() as u8;
        let align_bytes     = 16;
        
        let mut fill_vec2_count    = vec2_count % 2;
        fill_vec2_count = if fill_vec2_count == 0 { 0 } else { 2 - fill_vec2_count };
        let mut fill_int_count     = (float_count + int_count + uint_count) % 4;
        fill_int_count = if fill_int_count == 0 { 0 } else { 4 - fill_int_count };

        let mut total_size = 0;

        let mat4_begin: u32  = total_size;
        total_size += mat4_count as u32 * Self::MAT4_BYTES;

        let mat2_begin: u32  = total_size;
        total_size += mat2_count as u32 * Self::MAT2_BYTES;

        let vec4_begin: u32  = total_size;
        total_size += vec4_count as u32 * Self::VEC4_BYTES;

        let vec2_begin: u32  = total_size;
        total_size += (vec2_count as u32 + fill_vec2_count as u32) * Self::VEC2_BYTES;

        let float_begin: u32 = total_size;
        total_size += float_count as u32 * Self::FLOAT_BYTES;

        let int_begin: u32   = total_size;
        total_size += int_count as u32 * Self::INT_BYTES;

        let uint_begin: u32  = total_size;
        total_size += uint_count as u32 * Self::UINT_BYTES;

        total_size += fill_int_count as u32 * Self::INT_BYTES;

        if total_size == 0 {
            total_size += 4 * Self::UINT_BYTES; // 4 个 占位u32; 对应MaterialValueBindDesc中也有处理
        }

        Self {
            bind,
            total_size: total_size as usize,
            mat4_count,
            mat2_count,
            vec4_count,
            vec2_count,
            float_count,
            int_count,
            uint_count,
            fill_vec2_count,
            fill_int_count,
            mat4_begin,
            mat2_begin,
            vec4_begin,
            vec2_begin,
            float_begin,
            int_begin,
            uint_begin,
            effect
        }
    }
    
    pub fn label(
        &self
    ) -> String {
        self.mat4_count.to_string() 
        + Self::LABEL_MASK + &self.mat2_count.to_string() 
        + Self::LABEL_MASK + &self.vec4_count.to_string() 
        + Self::LABEL_MASK + &self.vec2_count.to_string() 
        + Self::LABEL_MASK + &self.float_count.to_string() 
        + Self::LABEL_MASK + &self.int_count.to_string()
        + Self::LABEL_MASK + &self.uint_count.to_string()
    }
    
    pub fn vs_define_code(&self, set: u32) -> String {
        self.effect.vs_code(set, self.bind)
    }
    
    pub fn fs_define_code(&self, set: u32) -> String {
        self.effect.fs_code(set, self.bind)
    }
}
impl TShaderBind for ShaderBindEffectValue {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {
        if self.total_size > 0 {
            entries.push(
                wgpu::BindGroupLayoutEntry {
                    binding: self.bind,
                    visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: NonZeroU64::new(self.total_size as wgpu::BufferAddress)
                    },
                    count: None,
                }
            );
        }
    }

    fn bind(&self) -> u32 {
        self.bind
    }
}
