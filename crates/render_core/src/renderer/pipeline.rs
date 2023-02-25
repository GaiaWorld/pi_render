use std::{sync::Arc, marker::PhantomData};

use pi_assets::{asset::{Asset, Handle}, mgr::AssetMgr};
use pi_share::Share;

use crate::rhi::device::RenderDevice;

use super::{bind_group::{KeyBindGroupLayout, BindGroupLayout}, shader::{KeyShader, TKeyShaderSetBlock, Shader}, vertex_buffer::{KeyPipelineFromAttributes}, ASSET_SIZE_FOR_UNKOWN};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DepthBiasState {
    /// Constant depth biasing factor, in basic units of the depth format.
    pub constant: i32,
    /// Slope depth biasing factor.
    pub slope_scale: i32,
    /// Depth bias clamp value (absolute).
    pub clamp: i32,
}
impl DepthBiasState {
    pub const BASE_SLOPE_SCALE: f32 = 0.00001;
    pub const BASE_CLAMP: f32 = 0.00001;
    pub fn depth_bias_state(&self) -> wgpu::DepthBiasState {
        wgpu::DepthBiasState {
            constant: self.constant,
            slope_scale: self.slope_scale as f32 * Self::BASE_SLOPE_SCALE,
            clamp: self.clamp as f32 * Self::BASE_CLAMP,
        }
    } 
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct DepthStencilState {
    pub format: wgpu::TextureFormat,
    pub depth_write_enabled: bool,
    pub depth_compare: wgpu::CompareFunction,
    pub stencil: wgpu::StencilState,
    pub bias: DepthBiasState,
}
impl DepthStencilState {
    pub fn depth_stencil_state(&self) -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: self.format,
            depth_write_enabled: self.depth_write_enabled,
            depth_compare: self.depth_compare,
            stencil: self.stencil.clone(),
            bias: self.bias.depth_bias_state(),
        }
    } 
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyRenderPipelineState {
    pub primitive: wgpu::PrimitiveState,
    pub multisample: wgpu::MultisampleState,
    pub depth_stencil: Option<DepthStencilState>,
    pub target_state: Vec<Option<wgpu::ColorTargetState>>,
}

/// * Pipeline 
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyRenderPipeline<const MAX_BIND_GROUP_COUNT: usize, K: TKeyShaderSetBlock> {
    pub key_state: KeyRenderPipelineState,
    pub key_shader: KeyShader<MAX_BIND_GROUP_COUNT, K>,
    pub key_bindgroup_layouts: [Option<Arc<KeyBindGroupLayout>>; MAX_BIND_GROUP_COUNT],
    pub key_vertex_layouts: KeyPipelineFromAttributes,
}

pub trait TRenderPipeline: Clone {
    fn pipeline(&self) -> &crate::rhi::pipeline::RenderPipeline;
}

#[derive(Debug, Clone)]
pub struct RenderPipeline<const MAX_BIND_GROUP_COUNT: usize, K: TKeyShaderSetBlock>(pub crate::rhi::pipeline::RenderPipeline, PhantomData<K>);
impl<const MAX_BIND_GROUP_COUNT: usize, K: TKeyShaderSetBlock> RenderPipeline<MAX_BIND_GROUP_COUNT, K> {
    pub fn pipeline(
        &self
    ) -> &wgpu::RenderPipeline {
        &self.0
    }
    pub fn bind_group_layouts(
        key_bindgroup_layouts: &[Option<Arc<KeyBindGroupLayout>>; MAX_BIND_GROUP_COUNT],
        asset_mgr_bindgroup_layout: &Share<AssetMgr<BindGroupLayout>>,
    ) -> Option<Vec<Handle<BindGroupLayout>>> {
        let mut result = vec![];
        let len = key_bindgroup_layouts.len();
        for i in 0..len {
            if let Some(key) = key_bindgroup_layouts.get(i).unwrap() {
                if let Some(layout) = asset_mgr_bindgroup_layout.get(key) {
                    result.push(layout);
                } else {
                    return None;
                }
            }
        }
        Some(result)
    }
    pub fn create(
        key_state: KeyRenderPipelineState,
        key_shader: KeyShader<MAX_BIND_GROUP_COUNT, K>,
        shader: Shader<MAX_BIND_GROUP_COUNT, K>,
        key_bindgroup_layouts: [Option<Arc<KeyBindGroupLayout>>; MAX_BIND_GROUP_COUNT],
        bind_group_layouts: [Option<Handle<BindGroupLayout>>; MAX_BIND_GROUP_COUNT],
        key_vertex_layouts: KeyPipelineFromAttributes,
        asset_mgr_pipeline: &Share<AssetMgr<RenderPipeline<MAX_BIND_GROUP_COUNT, K>>>,
        device: &RenderDevice,
    ) -> Option<Handle<RenderPipeline<MAX_BIND_GROUP_COUNT, K>>> {
        let key = KeyRenderPipeline {
            key_state,
            key_shader,
            key_bindgroup_layouts,
            key_vertex_layouts,
        };

        if let Some(pipeline) = asset_mgr_pipeline.get(&key) {
            Some(pipeline)
        } else {
            let mut layouts: Vec<&wgpu::BindGroupLayout> = vec![];
            bind_group_layouts.iter().for_each(|v| {
                if let Some(v) = v {
                    layouts.push(&v.layout)
                }
            });
            let vs_state = wgpu::VertexState {
                module: &shader.vs,
                entry_point: shader.vs_point,
                buffers: &key.key_vertex_layouts.layouts(),
            };
            let fs_state = wgpu::FragmentState {
                module: &shader.fs,
                entry_point: shader.fs_point,
                targets: &key.key_state.target_state,
            };
    
            let pipeline_layout = device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &layouts,
                    push_constant_ranges: &[],
                }
            );
    
            let depth_stencil = if let Some(depth_stencil) = &key.key_state.depth_stencil {
                Some(depth_stencil.depth_stencil_state())
            } else {
                None
            };
    
            let pipeline = device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: None,
                    // label: Some(shader.key()),
                    layout: Some(&pipeline_layout),
                    vertex: vs_state,
                    fragment: Some(fs_state),
                    primitive: key.key_state.primitive.clone(),
                    depth_stencil,
                    multisample: key.key_state.multisample,
                    multiview: None,
                }
            );
            asset_mgr_pipeline.insert(key, RenderPipeline(pipeline, PhantomData))
        }
    }
}
impl<const MAX_BIND_GROUP_COUNT: usize, K: TKeyShaderSetBlock> Asset for RenderPipeline<MAX_BIND_GROUP_COUNT, K> {
    type Key = KeyRenderPipeline<MAX_BIND_GROUP_COUNT, K>;

    fn size(&self) -> usize {
        ASSET_SIZE_FOR_UNKOWN
    }
}
impl<const MAX_BIND_GROUP_COUNT: usize, K: TKeyShaderSetBlock> TRenderPipeline for RenderPipeline<MAX_BIND_GROUP_COUNT, K> {
    fn pipeline(&self) -> &crate::rhi::pipeline::RenderPipeline {
        &self.0
    }
}
