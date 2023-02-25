// use fragment_state::gen_fragment_state_key;
// use pi_assets::asset::Asset;
// use pipeline_key::{PipelineStateKey, PipelineStateKeyCalcolator, gen_pipeline_key};
// use render_core::rhi::{bind_group::BindGroupId, device::RenderDevice, pipeline::RenderPipeline, bind_group_layout::BindGroupLayout};
// use render_geometry::vertex_data::{VertexBufferLayouts, ResVertexBufferLayout};
// use render_shader::shader::{KeyShader, ResShader};

// pub mod fragment_state;
// pub mod uniform_info;
// pub mod pipeline_key;

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub struct DepthBiasState {
//     /// Constant depth biasing factor, in basic units of the depth format.
//     pub constant: i32,
//     /// Slope depth biasing factor.
//     pub slope_scale: i32,
//     /// Depth bias clamp value (absolute).
//     pub clamp: i32,
// }
// impl DepthBiasState {
//     pub const BASE_SLOPE_SCALE: f32 = 0.00001;
//     pub const BASE_CLAMP: f32 = 0.00001;
//     pub fn depth_bias_state(&self) -> wgpu::DepthBiasState {
//         wgpu::DepthBiasState {
//             constant: self.constant,
//             slope_scale: self.slope_scale as f32 * Self::BASE_SLOPE_SCALE,
//             clamp: self.clamp as f32 * Self::BASE_CLAMP,
//         }
//     } 
// }

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub struct DepthStencilState {
//     pub format: wgpu::TextureFormat,
//     pub depth_write_enabled: bool,
//     pub depth_compare: wgpu::CompareFunction,
//     pub stencil: wgpu::StencilState,
//     pub bias: DepthBiasState,
// }
// impl DepthStencilState {
//     pub fn depth_stencil_state(&self) -> wgpu::DepthStencilState {
//         wgpu::DepthStencilState {
//             format: self.format,
//             depth_write_enabled: self.depth_write_enabled,
//             depth_compare: self.depth_compare,
//             stencil: self.stencil.clone(),
//             bias: self.bias.depth_bias_state(),
//         }
//     } 
// }

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub struct KeyPipelineState {
//     pub primitive: wgpu::PrimitiveState,
//     pub target_state: Vec<Option<wgpu::ColorTargetState>>,
//     pub depth_stencil: Option<DepthStencilState>,
// }

// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub struct KeyRenderPipeline {
//     pub key_shader: KeyShader,
//     pub key_bind_groups: Vec<BindGroupId>,
//     pub key_vertex_layout: Vec<ResVertexBufferLayout>,
//     pub key_state: KeyPipelineState,
// }
// impl PartialOrd for KeyRenderPipeline {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         match self.key_shader.partial_cmp(&other.key_shader) {
//             Some(core::cmp::Ordering::Equal) => {}
//             ord => return ord,
//         }
//         // match self.key_bind_groups.partial_cmp(&other.key_bind_groups) {
//         //     Some(core::cmp::Ordering::Equal) => {}
//         //     ord => return ord,
//         // }
//         self.key_bind_groups.partial_cmp(&other.key_bind_groups)
//     }
// }
// impl Ord for KeyRenderPipeline {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.partial_cmp(&other).unwrap()
//     }
// }

// #[derive(Debug)]
// pub struct ResRenderPipeline(pub RenderPipeline);
// impl Asset for ResRenderPipeline {
//     type Key = KeyRenderPipeline;

//     fn size(&self) -> usize {
//         256
//     }
// }

// impl ResRenderPipeline {
//     pub fn new(
//         device: &RenderDevice,
//         shader: &ResShader,
//         targets: &[Option<wgpu::ColorTargetState>],
//         depth_stencil: &Option<DepthStencilState>,
//         primitive: wgpu::PrimitiveState,
//         vertex_layouts: &[wgpu::VertexBufferLayout],
//         bind_group_layouts: &[&wgpu::BindGroupLayout],
//     ) -> Self {

//         let vs_state = wgpu::VertexState {
//             module: &shader.vs,
//             entry_point: shader.vs_point,
//             buffers: &vertex_layouts,
//         };
//         let fs_state = wgpu::FragmentState {
//             module: &shader.fs,
//             entry_point: shader.fs_point,
//             targets,
//         };

//         let pipeline_layout = device.create_pipeline_layout(
//             &wgpu::PipelineLayoutDescriptor {
//                 label: None,
//                 bind_group_layouts,
//                 push_constant_ranges: &[],
//             }
//         );

//         let depth_stencil = if let Some(depth_stencil) = depth_stencil {
//             Some(depth_stencil.depth_stencil_state())
//         } else {
//             None
//         };

//         let pipeline = device.create_render_pipeline(
//             &wgpu::RenderPipelineDescriptor {
//                 label: None,
//                 // label: Some(shader.key()),
//                 layout: Some(&pipeline_layout),
//                 vertex: vs_state,
//                 fragment: Some(fs_state),
//                 primitive,
//                 depth_stencil,
//                 multisample: wgpu::MultisampleState {
//                     count: 1,
//                     mask: !0,
//                     alpha_to_coverage_enabled: false
//                 },
//                 multiview: None,
//             }
//         );

//         Self(pipeline)
//     }
// }

// pub fn pipeline_state_key(
//     targets: &[Option<wgpu::ColorTargetState>],
//     primitive: &wgpu::PrimitiveState,
//     depth_stencil: &Option<wgpu::DepthStencilState>,
//     depth_stencil_bias_mode: u8,
//     depth_stencil_bias_modes_use_bite: u8,
// ) -> PipelineStateKey {
//     let mut calcolator = PipelineStateKeyCalcolator::new();
//     gen_pipeline_key(&mut calcolator, &primitive, &depth_stencil, depth_stencil_bias_mode, depth_stencil_bias_modes_use_bite);
//     match targets.get(0) {
//         Some(target) => {
//             match target {
//                 Some(target) => {
//                     gen_fragment_state_key(&mut calcolator, target);
//                 },
//                 None => {},
//             }
//         },
//         None => {},
//     }
//     calcolator.key
// }