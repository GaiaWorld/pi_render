// use pi_assets::asset::Handle;
// use pi_map::vecmap::VecMap;

// use super::{
//     asset::RenderRes, bind_group::BindGroup, buffer::Buffer, dyn_uniform_buffer::DynBufferIndex,
//     pipeline::RenderPipeline,
// };

// /// 渲染对象
// #[derive(Debug)]
// pub struct DrawState {
//     // 一个 Pipeleine
//     pub pipeline: Option<Handle<RenderRes<RenderPipeline>>>, //Option<Share<RenderPipeline>>,

//     // 一堆 UBO
//     pub bind_groups: Groups,

//     // 一堆 VB
//     pub vbs: VecMap<(Handle<RenderRes<Buffer>>, u64)>,

//     // IB 可有 可无
//     pub ib: Option<(Handle<RenderRes<Buffer>>, u64, wgpu::IndexFormat)>,
// }

// #[derive(Debug, Default)]
// pub struct Groups(VecMap<DrawGroup>);

// impl Groups {
//     #[inline]
//     pub fn get_group(&self, group_id: GroupId) -> Option<&DrawGroup> {
//         self.0.get(group_id as usize)
//     }

//     pub fn insert_group(&mut self, group_id: GroupId, value: DrawGroup) {
//         self.0.insert(group_id as usize, value);
//     }

//     #[inline]
//     pub fn groups(&self) -> &VecMap<DrawGroup> {
//         &self.0
//     }
// }

// #[derive(Debug)]
// pub enum DrawGroup {
//     Offset(DynBufferIndex), // 某个buffer的部分 (在全局中的索引， buffer偏移量) 具有动态偏移
//     Independ(Handle<RenderRes<BindGroup>>), // 无动态偏移
// }

// #[derive(Debug)]
// pub struct DynDrawGroup {
//     index: usize,
//     offsets: SmallVec<[u32; 1]>,
//     offset_index: SmallVec<[BindOffset; 1]>,
// }

// impl DynDrawGroup {
//     pub fn new(index: usize, offsets: SmallVec<[BindOffset; 1]>) -> Self {
//         let offsets1: SmallVec<[u32; 1]> = offsets.iter().map(|r| **r).collect();
//         Self {
//             index,
//             offsets: offsets1,
//             offset_index: offsets,
//         }
//     }
// }

// impl DrawGroup {
//     pub fn get_offset(&self, bind_index: BindIndex) -> Option<&BindOffset> {
//         if let DrawGroup::Offset(DynDrawGroup { offset_index, .. }) = self {
//             return offset_index.get(*bind_index);
//         }
//         None
//     }

//     pub fn draw<'w, 'a>(
//         &'a self,
//         rpass: &'w mut RenderPass<'a>,
//         dyn_groups: &'a DynBindGroups,
//         i: u32,
//     ) {
//         match self {
//             DrawGroup::Offset(DynDrawGroup { index, offsets, .. }) => {
//                 if let Some((Some(group), _, _)) = dyn_groups.get(*index) {
//                     rpass.set_bind_group(i as u32, group, offsets.as_slice())
//                 }
//             }
//             DrawGroup::Independ(group) => rpass.set_bind_group(i as u32, group, &[]),
//         };
//     }
// }

// // impl Default for DrawState {
// //     fn default() -> Self {
// //         Self { pipeline: Default::default(), bind_groups: VecMap::default(), vbs: Default::default(), ib: Default::default() }
// //     }
// // }

// // pub struct DrawContext {
// // 	// pub group_context: Share<ShareMutex<BindGroupContext>>,
// // 	pub groups: Vec<BindGroup>,
// // }

// impl DrawState {
//     pub fn draw<'w, 'a>(&'a self, rpass: &'w mut RenderPass<'a>, dyn_groups: &'a DynBindGroups) {
//         if let (Some(p), Some(ib)) = (&self.pipeline, &self.ib) {
//             rpass.set_pipeline(p);
//             let mut i = 0;
//             for r in self.bind_groups.groups().iter() {
//                 if let Some(group) = r {
//                     group.draw(rpass, dyn_groups, i as u32);
//                 }
//                 i += 1;
//             }
//             i = 0;
//             for r in self.vbs.iter() {
//                 if let Some(vertex_buf) = r {
//                     rpass.set_vertex_buffer(i as u32, (****vertex_buf.0).slice(..));
//                 }
//                 i += 1;
//             }

//             rpass.set_index_buffer((****ib.0).slice(..), ib.2);
//             rpass.draw_indexed(0..ib.1 as u32, 0, 0..1);
//         }
//     }
// }
