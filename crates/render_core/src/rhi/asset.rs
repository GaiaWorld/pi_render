// use std::hash::Hash;

// use derive_deref_rs::Deref;
// use futures::future::BoxFuture;
// use pi_ecs::prelude::{
//     ResMut, SystemParam, SystemParamFetch, SystemParamState, SystemState, World,
// };
// use pi_ecs::sys::param::res::ResMutState;
// use pi_res_loader::{Asset, LoadMgr, Res, State as LoadState};
// use pi_share::{Share, ShareCell};
// use wgpu::util::{BufferInitDescriptor, DeviceExt};

// pub struct WgpuAssetDescriptor<'a, 'b, T> {
//     descriptor: &'a T,
//     device: &'b wgpu::Device,
// }

// trait AssetDescriptor<T> {
//     fn get_device(&self) -> &wgpu::Device;
//     fn get_descriptor(&self) -> &T;
// }

// impl<'a, 'b, T> AssetDescriptor<T> for WgpuAssetDescriptor<'a, 'b, T> {
//     fn get_device(&self) -> &wgpu::Device {
//         self.device
//     }

//     fn get_descriptor(&self) -> &T {
//         self.descriptor
//     }
// }

// macro_rules! impl_asset {
//     ($struct_name: ident, $create_fn: ident, $struct_descriptor: ident) => {
//         #[derive(Debug, Deref)]
//         pub struct $struct_name {
//             #[deref]
//             value: wgpu::$struct_name,
//             state: LoadState,
//         }
//         impl Res for $struct_name {
//             type Key = u64;
//         }

//         impl<'a, Desc> Asset<Desc> for $struct_name
//         where
//             Desc: AssetDescriptor<wgpu::$struct_descriptor<'a>>,
//         {
//             fn state(&self) -> pi_res_loader::State {
//                 self.state
//             }

//             fn is_async() -> bool {
//                 false
//             }

//             fn async_load(
//                 load_mgr: &mut LoadMgr,
//                 desc: Desc,
//             ) -> BoxFuture<'static, Self> {
//                 unimplemented!()
//             }

//             fn load(
//                 load_mgr: &mut LoadMgr,
//                 desc: Desc,
//             ) -> Self {
//                 // TODO
//                 let r = desc.get_device().$create_fn(desc.get_descriptor());
//                 // r.state = LoadState::Ok;
// 				$struct_name{
// 					value: r,
// 					state: LoadState::Ok,
// 				}
//                 // unsafe {&mut *(Share::as_ptr(&asset) as usize as *mut Self)}.value =
//             }
//         }
//     };
// }

// impl_asset!(
//     BindGroupLayout,
//     create_bind_group_layout,
//     BindGroupLayoutDescriptor
// );
// impl_asset!(BindGroup, create_bind_group, BindGroupDescriptor);
// impl_asset!(Buffer, create_buffer, BufferDescriptor);
// impl_asset!(
//     PipelineLayout,
//     create_pipeline_layout,
//     PipelineLayoutDescriptor
// );
// impl_asset!(
//     RenderPipeline,
//     create_render_pipeline,
//     RenderPipelineDescriptor
// );
// impl_asset!(
//     ComputePipeline,
//     create_compute_pipeline,
//     ComputePipelineDescriptor
// );
// // ...
