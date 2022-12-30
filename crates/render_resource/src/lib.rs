use std::{hash::Hash, fmt::Debug};
use pi_atom::Atom;
use render_pipeline_key::pipeline_key::PipelineStateKey;
use sampler::SamplerAssetKey;

pub mod bind_group_layout;
pub mod bind_group;
pub mod uniform_buffer;
pub mod sampler;
pub mod texture2d;
pub mod data_texture2d;

pub trait AssetKey: Debug + Clone + Hash + PartialEq + Eq + PartialOrd + Ord {
    
}

pub fn bind_group_entry_buffer(
    binding: u32,
    buffer: &wgpu::Buffer,
    offset: wgpu::BufferAddress,
    size: wgpu::BufferAddress,
) -> wgpu::BindGroupEntry {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Buffer(
            wgpu::BufferBinding {
                buffer,
                offset,
                size: wgpu::BufferSize::new(size),
            }
        ),
    }
}

pub type ShaderAssetKey = Atom;

pub type ShaderDefineMode = u128;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShaderEffectAssetKey {
    pub shader: ShaderAssetKey,
    pub define: ShaderDefineMode,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderPipelineAssetKey {
    pub shader_effect: ShaderEffectAssetKey,
    pub pipeline: PipelineStateKey,
}

pub type BindGroupSet = u8;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindGroupLayoutAssetKey {
    pub shader_effect: ShaderEffectAssetKey,
    pub set: BindGroupSet,
}

pub type BindGroupAssetKey = BindGroupLayoutAssetKey;

pub type ImageAssetKey = Atom;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextureAssetKey {
    pub image: ImageAssetKey,
    pub sampler: SamplerAssetKey,
}