use std::{num::NonZeroU64, ops::Deref};

use render_core::rhi::{dyn_uniform_buffer::{BindOffset, AsBind}};

use crate::{shader::ShaderEffectMeta, unifrom_code::TUnifromShaderProperty};


pub trait ShaderBind {
    fn define_code(&self) -> String;
}

struct ShaderBindTemp(u32, wgpu::BufferAddress);
impl AsBind for ShaderBindTemp {
    fn min_size(&self) -> usize {
        self.1 as usize
    }

    fn index(&self) -> render_core::rhi::dyn_uniform_buffer::BindIndex {
        render_core::rhi::dyn_uniform_buffer::BindIndex::new(self.0 as usize)
    }
}

#[derive(Debug)]
pub struct ShaderBindSceneAboutCamera {
    bind_offset: BindOffset,
    bind: u32,
}
impl AsBind for ShaderBindSceneAboutCamera {
    fn min_size(&self) -> usize {
        Self::TOTAL_SIZE as usize
    }

    fn index(&self) -> render_core::rhi::dyn_uniform_buffer::BindIndex {
        render_core::rhi::dyn_uniform_buffer::BindIndex::new(self.bind as usize)
    }
}
impl ShaderBindSceneAboutCamera {

    pub const OFFSET_VIEW_MATRIX:           wgpu::BufferAddress = 0;
    pub const OFFSET_PROJECT_MATRIX:        wgpu::BufferAddress = 16 * 4;
    pub const OFFSET_VIEW_PROJECT_MATRIX:   wgpu::BufferAddress = 16 * 4 + 16 * 4;
    pub const OFFSET_CAMERA_POSITION:       wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4;
    pub const OFFSET_CAMERA_DIRECTION:      wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4 + 4 * 4;
    
    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4 + 4 * 4 + 4 * 4;

    pub fn layout_entry(bind: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: bind,
            visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
            count: None,
        }
    }

    pub fn new(bind: u32, dynbuffer: &mut render_resource::uniform_buffer::RenderDynUniformBuffer) -> Self {
        let temp = ShaderBindTemp(bind, Self::TOTAL_SIZE);
        let bind_offset = dynbuffer.alloc_binding_with_asbind(&temp);

        Self { bind_offset, bind }
    }

    pub fn bind_offset(&self) -> &BindOffset {
        &self.bind_offset
    }

    pub fn offset(&self) -> u32 {
        *self.bind_offset
    }
}


#[derive(Debug)]
pub struct ShaderBindSceneAboutTime {
    bind_offset: BindOffset,
    bind: u32,
}
impl AsBind for ShaderBindSceneAboutTime {
    fn min_size(&self) -> usize {
        Self::TOTAL_SIZE as usize
    }

    fn index(&self) -> render_core::rhi::dyn_uniform_buffer::BindIndex {
        render_core::rhi::dyn_uniform_buffer::BindIndex::new(self.bind as usize)
    }
}
impl ShaderBindSceneAboutTime {

    pub const OFFSET_TIME:                  wgpu::BufferAddress = 0;
    pub const OFFSET_DELTA_TIME:            wgpu::BufferAddress = 4 * 4;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 4 * 4 + 4 * 4;

    pub fn layout_entry(bind: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: bind,
            visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
            count: None,
        }
    }

    pub fn new(bind: u32, dynbuffer: &mut render_resource::uniform_buffer::RenderDynUniformBuffer) -> Self {
        let temp = ShaderBindTemp(bind, Self::TOTAL_SIZE);
        let bind_offset = dynbuffer.alloc_binding_with_asbind(&temp);

        Self { bind_offset, bind }
    }

    pub fn bind_offset(&self) -> &BindOffset {
        &self.bind_offset
    }

    pub fn offset(&self) -> u32 {
        *self.bind_offset
    }
}

#[derive(Debug)]
pub struct ShaderBindSceneAboutFog {
    bind_offset: BindOffset,
    bind: u32,
}
impl AsBind for ShaderBindSceneAboutFog {
    fn min_size(&self) -> usize {
        Self::TOTAL_SIZE as usize
    }

    fn index(&self) -> render_core::rhi::dyn_uniform_buffer::BindIndex {
        render_core::rhi::dyn_uniform_buffer::BindIndex::new(self.bind as usize)
    }
}
impl ShaderBindSceneAboutFog {

    pub const OFFSET_FOG_INFO:              wgpu::BufferAddress = 0;
    pub const OFFSET_FOG_PARAM:             wgpu::BufferAddress = 4 * 4;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 4 * 4 + 4 * 4;

    pub fn layout_entry(bind: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: bind,
            visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
            count: None,
        }
    }

    pub fn new(bind: u32, dynbuffer: &mut render_resource::uniform_buffer::RenderDynUniformBuffer) -> Self {
        let temp = ShaderBindTemp(bind, Self::TOTAL_SIZE);
        let bind_offset = dynbuffer.alloc_binding_with_asbind(&temp);

        Self { bind_offset, bind }
    }

    pub fn bind_offset(&self) -> &BindOffset {
        &self.bind_offset
    }

    pub fn offset(&self) -> u32 {
        *self.bind_offset
    }
}

#[derive(Debug)]
pub struct ShaderBindModelAboutMatrix {
    bind_offset: BindOffset,
    bind: u32,
}
impl AsBind for ShaderBindModelAboutMatrix {
    fn min_size(&self) -> usize {
        Self::TOTAL_SIZE as usize
    }

    fn index(&self) -> render_core::rhi::dyn_uniform_buffer::BindIndex {
        render_core::rhi::dyn_uniform_buffer::BindIndex::new(self.bind as usize)
    }
}
impl ShaderBindModelAboutMatrix {

    pub const OFFSET_WORLD_MATRIX:          wgpu::BufferAddress = 0;
    pub const OFFSET_WORLD_MATRIX_INV:      wgpu::BufferAddress = 16 * 4;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 16 * 4 + 16 * 4;

    pub fn layout_entry(bind: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: bind,
            visibility: wgpu::ShaderStages ::VERTEX,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
            count: None,
        }
    }

    pub fn new(bind: u32, dynbuffer: &mut render_resource::uniform_buffer::RenderDynUniformBuffer) -> Self {
        let temp = ShaderBindTemp(bind, Self::TOTAL_SIZE);
        let bind_offset = dynbuffer.alloc_binding_with_asbind(&temp);

        Self { bind_offset, bind }
    }


    pub fn bind_offset(&self) -> &BindOffset {
        &self.bind_offset
    }

    pub fn offset(&self) -> u32 {
        *self.bind_offset
    }
}

#[derive(Debug)]
pub enum ShaderBindModelAboutSkin {
    RowTexture(u32, BindOffset),
    FramesTexture(u32, BindOffset),
}
impl AsBind for ShaderBindModelAboutSkin {
    fn min_size(&self) -> usize {
        Self::TOTAL_SIZE as usize
    }

    fn index(&self) -> render_core::rhi::dyn_uniform_buffer::BindIndex {
        match self {
            ShaderBindModelAboutSkin::RowTexture(bind, _) => render_core::rhi::dyn_uniform_buffer::BindIndex::new(*bind as usize),
            ShaderBindModelAboutSkin::FramesTexture(bind, _) => render_core::rhi::dyn_uniform_buffer::BindIndex::new(*bind as usize),
        }
        
    }
}
impl ShaderBindModelAboutSkin {

    pub const OFFSET_BONE_TEX_SIZE:         wgpu::BufferAddress = 0;

    pub const TOTAL_SIZE:                   wgpu::BufferAddress = 0 + 4 * 4;

    pub fn layout_entry(entries: &mut Vec<wgpu::BindGroupLayoutEntry>, bind_info: u32, bind_tex: u32, bind_sampler: u32, ) {
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: bind_info,
            visibility: wgpu::ShaderStages ::VERTEX,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(Self::TOTAL_SIZE) },
            count: None,
        });
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: bind_tex,
            visibility: wgpu::ShaderStages ::VERTEX,
            ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: false }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
            count: None,
        });
        entries.push(wgpu::BindGroupLayoutEntry {
            binding: bind_sampler,
            visibility: wgpu::ShaderStages ::VERTEX,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            count: None,
        });
    }

    pub fn new_row(bind: u32, dynbuffer: &mut render_resource::uniform_buffer::RenderDynUniformBuffer) -> Self {
        let temp = ShaderBindTemp(bind, Self::TOTAL_SIZE);
        let bind_offset = dynbuffer.alloc_binding_with_asbind(&temp);
        Self::RowTexture(bind, bind_offset)
    }

    pub fn new_frames(bind: u32, dynbuffer: &mut render_resource::uniform_buffer::RenderDynUniformBuffer) -> Self {
        let temp = ShaderBindTemp(bind, Self::TOTAL_SIZE);
        let bind_offset = dynbuffer.alloc_binding_with_asbind(&temp);
        Self::FramesTexture(bind, bind_offset)
    }

    pub fn bind_offset(&self) -> &BindOffset {
        match self {
            ShaderBindModelAboutSkin::RowTexture(_, bind_offset) => bind_offset,
            ShaderBindModelAboutSkin::FramesTexture(_, bind_offset) => bind_offset,
        }
    }

    pub fn offset(&self) -> u32 {
        match self {
            ShaderBindModelAboutSkin::RowTexture(_, bind_offset) => *bind_offset.deref(),
            ShaderBindModelAboutSkin::FramesTexture(_, bind_offset) => *bind_offset.deref(),
        }
    }
}

#[derive(Debug)]
pub struct ShaderBindEffectValue {
    bind: u32,
    bind_offset: Option<BindOffset>,
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
}
impl AsBind for ShaderBindEffectValue {
    fn min_size(&self) -> usize {
        self.total_size as usize
    }

    fn index(&self) -> render_core::rhi::dyn_uniform_buffer::BindIndex {
        render_core::rhi::dyn_uniform_buffer::BindIndex::new(Self::BIND as usize)
    }
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

    pub fn new<
        TMat4: TUnifromShaderProperty,
        TMat2: TUnifromShaderProperty,
        TVec4: TUnifromShaderProperty,
        TVec2: TUnifromShaderProperty,
        TFloat: TUnifromShaderProperty,
        TInt: TUnifromShaderProperty,
        TUint: TUnifromShaderProperty
    >(
        effect: &ShaderEffectMeta<TMat4, TMat2, TVec4, TVec2, TFloat, TInt, TUint>,
        dynbuffer: &mut render_resource::uniform_buffer::RenderDynUniformBuffer
    ) -> Self {
        let uniforms = &effect.uniforms;
        let mat4_count      = uniforms.mat4_list.len() as u8;
        let mat2_count      = uniforms.mat2_list.len() as u8;
        let vec4_count      = uniforms.vec4_list.len() as u8;
        let vec2_count      = uniforms.vec2_list.len() as u8;
        let float_count     = uniforms.float_list.len() as u8;
        let int_count       = uniforms.int_list.len() as u8;
        let uint_count      = uniforms.uint_list.len() as u8;
        let align_bytes     = 16;
        
        let fill_vec2_count    = vec2_count % 2;
        let fill_int_count     = (float_count + int_count + uint_count) % 4;

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

        let bind_offset = if total_size > 0 {
            let temp = ShaderBindTemp(ShaderBindEffectValue::BIND, total_size as wgpu::BufferAddress);
            Some(dynbuffer.alloc_binding_with_asbind(&temp))
        } else {
            None
        };

        Self {
            bind: ShaderBindEffectValue::BIND,
            bind_offset,
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

    pub fn layout_entries(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>) {

        if self.total_size > 0 {
            entries.push(
                wgpu::BindGroupLayoutEntry {
                    binding: ShaderBindEffectValue::BIND,
                    visibility: wgpu::ShaderStages ::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: NonZeroU64::new(self.total_size as wgpu::BufferAddress) },
                    count: None,
                }
            );
        }
    }

    pub fn bind_offset(&self) -> Option<&BindOffset> {
        self.bind_offset.as_ref()
    }

    pub fn bind_offset_info(&self) -> Vec<u32> {
        if let Some(bind_offset) = &self.bind_offset {
            vec![*bind_offset.deref()]
        } else {
            vec![]
        }
    }
}