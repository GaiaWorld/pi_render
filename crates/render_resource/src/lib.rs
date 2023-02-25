// use std::{hash::Hash, fmt::Debug};
// use pi_atom::Atom;

// pub mod bind_group_layout;
// pub mod bind_group;
// pub mod uniform_buffer;
// pub mod sampler;
// pub mod texture2d;
// pub mod data_texture2d;
// pub mod buffer;
// pub mod memory;
// pub mod base;
// pub mod shader_bind;
// pub mod shader_set;
// pub mod texture;

// pub trait AssetKey: Debug + Clone + Hash + PartialEq + Eq + PartialOrd + Ord {
    
// }

// pub fn bind_group_entry_buffer(
//     binding: u32,
//     buffer: &wgpu::Buffer,
//     offset: wgpu::BufferAddress,
//     size: wgpu::BufferAddress,
// ) -> wgpu::BindGroupEntry {
//     wgpu::BindGroupEntry {
//         binding,
//         resource: wgpu::BindingResource::Buffer(
//             wgpu::BufferBinding {
//                 buffer,
//                 offset,
//                 size: wgpu::BufferSize::new(size),
//             }
//         ),
//     }
// }

// pub type ShaderAssetKey = Atom;

// pub type ShaderDefineMode = u128;


// pub type BindGroupSet = u8;

// pub type ImageAssetKey = Atom;
