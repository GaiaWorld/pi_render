use std::{sync::Arc};

use pi_assets::{asset::Handle};

use crate::{
    render_3d::shader::*,
    rhi::device::RenderDevice,
    renderer::{
        shader::{KeyShaderMeta, TShaderBindCode},
        bind_buffer::{BindBufferAllocator, BindBufferRange},
        bind::{TKeyBind, KeyBindBuffer, KeyBindLayoutBuffer},
        shader_stage::EShaderStage
    }
};

#[derive(Debug, Clone)]
pub struct ShaderBindEffectValue {
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
    pub key_meta: KeyShaderMeta,
    pub meta: Handle<ShaderEffectMeta>,
    pub(crate) data: BindBufferRange,
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
        _: &RenderDevice,
        key_meta: KeyShaderMeta,
        meta: Handle<ShaderEffectMeta>,
        allocator: &mut BindBufferAllocator,
    ) -> Option<Self> {
        let uniforms = &meta.uniforms;
        let mat4_count      = uniforms.mat4_list.len() as u8;
        let mat2_count      = uniforms.mat2_list.len() as u8;
        let vec4_count      = uniforms.vec4_list.len() as u8;
        let vec2_count      = uniforms.vec2_list.len() as u8;
        let float_count     = uniforms.float_list.len() as u8;
        let int_count       = uniforms.int_list.len() as u8;
        let uint_count      = uniforms.uint_list.len() as u8;
        // let align_bytes     = 16;
        
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

        // if total_size == 0 {
        //     total_size += 4 * Self::UINT_BYTES; // 4 个 占位u32; 对应 MaterialValueBindDesc 中也有处理
        // }

        if total_size == 0 {
            None
        } else {
            match allocator.allocate(total_size) {
                Some(data) => {
                    Some(
                        Self {
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
                            key_meta,
                            meta,
                            data
                        }
                    )
                },
                None => None,
            }
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

    pub fn data(&self) -> &BindBufferRange {
        &self.data
    }
    pub fn key_layout(&self, binding: u16) -> KeyBindLayoutBuffer {
        KeyBindLayoutBuffer {
            binding,
            visibility: EShaderStage::VERTEXFRAGMENT,
            min_binding_size: self.data.size(),
        }
    }
    pub fn vs_define_code(meta: &ShaderEffectMeta, set: u32, binding: u32) -> String {
        meta.uniforms.vs_code(set, binding)
    }
    pub fn fs_define_code(meta: &ShaderEffectMeta, set: u32, binding: u32) -> String {
        meta.uniforms.fs_code(set, binding)
    }
}
impl std::hash::Hash for ShaderBindEffectValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // self.total_size.hash(state);
        // self.mat4_count.hash(state);
        // self.mat2_count.hash(state);
        // self.vec4_count.hash(state);
        // self.vec2_count.hash(state);
        // self.float_count.hash(state);
        // self.int_count.hash(state);
        // self.uint_count.hash(state);
        // self.fill_vec2_count.hash(state);
        // self.fill_int_count.hash(state);
        // self.mat4_begin.hash(state);
        // self.mat2_begin.hash(state);
        // self.vec4_begin.hash(state);
        // self.vec2_begin.hash(state);
        // self.float_begin.hash(state);
        // self.int_begin.hash(state);
        // self.uint_begin.hash(state);
        self.key_meta.hash(state);
        self.data.id_buffer().hash(state);
    }
}
impl PartialEq for ShaderBindEffectValue {
    fn eq(&self, other: &Self) -> bool {
        self.key_meta == other.key_meta && self.data.id_buffer() == other.data.id_buffer()
    }
}
impl Eq for ShaderBindEffectValue {
    fn assert_receiver_is_total_eq(&self) {}
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BindUseEffectValue {
    pub(crate) bind: u32,
    pub(crate) data: Arc<ShaderBindEffectValue>,
}
impl BindUseEffectValue {
    pub fn new(
        bind: u32,
        data: Arc<ShaderBindEffectValue>,
    ) -> Self {
        Self { bind, data }
    }
    pub fn data(&self) -> &ShaderBindEffectValue {
        &self.data
    }
}
impl TShaderBindCode for BindUseEffectValue {
    fn vs_define_code(&self, set: u32) -> String {
        self.data.meta.uniforms.vs_code(set, self.bind)
    }
    fn fs_define_code(&self, set: u32) -> String {
        self.data.meta.uniforms.fs_code(set, self.bind)
    }
}
impl TKeyBind for BindUseEffectValue {
    fn key_bind(&self) -> Option<crate::renderer::bind::EKeyBind> {
        Some(
            crate::renderer::bind::EKeyBind::Buffer(
                KeyBindBuffer {
                    data: self.data.data.clone(),
                    layout: Arc::new(
                        KeyBindLayoutBuffer {
                            binding: self.bind as u16,
                            visibility: EShaderStage::VERTEXFRAGMENT,
                            min_binding_size: self.data.data.size()
                        }
                    ) 
                }
            )
        )
    }
}
