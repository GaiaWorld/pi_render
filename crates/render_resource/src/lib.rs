use pi_atom::Atom;
use render_pipeline_key::pipeline_key::PipelineKey;
use sampler::SamplerAssetKey;

pub mod bind_group_layout;
pub mod bind_group;
pub mod uniform_buffer;
pub mod sampler;

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
    pub shader: ShaderKey,
    pub define: ShaderDefineMode,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct RenderPipelineAssetKey {
    pub shader_effect: ShaderEffectKey,
    pub pipeline: PipelineKey,
}

pub type BindGroupSet = u8;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindGroupLayoutAssetKey {
    pub shader_effect: ShaderEffectKey,
    pub set: BindGroupSet,
}

pub type BindGroupAssetKey = BindGroupLayoutAssetKey;

pub type ImageAssetKey = Atom;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextureAssetKey {
    pub image: ImageAssetKey,
    pub sampler: SamplerAssetKey,
}