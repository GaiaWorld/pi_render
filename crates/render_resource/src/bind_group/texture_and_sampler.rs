use std::sync::Arc;

use pi_assets::asset::Handle;
use pi_atom::Atom;
use render_core::rhi::{dyn_uniform_buffer::BufferGroup, device::RenderDevice, texture::Sampler, asset::TextureRes};
use render_shader::{unifrom_code::EffectUniformTextureWithSamplerUseinfo};

use crate::shader_set::RenderBindGroupTextureSamplers;
