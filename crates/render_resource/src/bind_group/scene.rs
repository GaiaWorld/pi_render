use std::num::NonZeroU64;

use render_core::rhi::{dyn_uniform_buffer::BufferGroup, bind_group::BindGroup, device::RenderDevice, bind_group_layout::BindGroupLayout, buffer::Buffer};
use render_shader::{};

use crate::{shader_bind::{ShaderBindSceneAboutCamera, ShaderBindSceneAboutTime, ShaderBindSceneAboutFog, TShaderBind}, shader_set::RenderBindGroupScene};