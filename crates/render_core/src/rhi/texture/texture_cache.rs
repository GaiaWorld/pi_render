// use super::Texture;
// use crate::rhi::device::RenderDevice;
// use pi_hash::XHashMap;
// use pi_share::Share;
// use wgpu::{TextureDescriptor, TextureViewDescriptor};

// /// The internal representation of a [`CachedTexture`] used to track whether it was recently used
// /// and is currently taken.
// struct CachedTextureMeta {
//     texture: Texture,
//     default_view: Share<wgpu::TextureView>,
//     taken: bool,
//     frames_since_last_use: usize,
// }

// /// A cached GPU [`Texture`] with corresponding [`TextureView`].
// /// This is useful for textures that are created repeatedly (each frame) in the rendering process
// /// to reduce the amount of GPU memory allocations.
// pub struct CachedTexture {
//     pub texture: Texture,
//     pub default_view: Share<wgpu::TextureView>,
// }

// /// This resource caches textures that are created repeatedly in the rendering process and
// /// are only required for one frame.
// #[derive(Default)]
// pub struct TextureCache {
//     textures: XHashMap<wgpu::TextureDescriptor<'static>, Vec<CachedTextureMeta>>,
// }

// impl TextureCache {
//     /// Retrieves a texture that matches the `descriptor`. If no matching one is found a new
//     /// [`CachedTexture`] is created.
//     pub fn get(
//         &mut self,
//         render_device: &RenderDevice,
//         descriptor: TextureDescriptor<'static>,
//     ) -> CachedTexture {
//         match self.textures.entry(descriptor) {
//             std::collections::hash_map::Entry::Occupied(mut entry) => {
//                 for texture in entry.get_mut().iter_mut() {
//                     if !texture.taken {
//                         texture.frames_since_last_use = 0;
//                         texture.taken = true;
//                         return CachedTexture {
//                             texture: texture.texture.clone(),
//                             default_view: texture.default_view.clone(),
//                         };
//                     }
//                 }

//                 let texture = render_device.create_texture(&entry.key().clone());
//                 let default_view = texture.create_view(&TextureViewDescriptor::default());
//                 entry.get_mut().push(CachedTextureMeta {
//                     texture: texture.clone(),
//                     default_view: Share::new(default_view.clone()),
//                     frames_since_last_use: 0,
//                     taken: true,
//                 });
//                 CachedTexture {
//                     texture,
//                     default_view,
//                 }
//             }
//             std::collections::hash_map::Entry::Vacant(entry) => {
//                 let texture = render_device.create_texture(entry.key());
//                 let default_view = texture.create_view(&TextureViewDescriptor::default());
//                 entry.insert(vec![CachedTextureMeta {
//                     texture: texture.clone(),
//                     default_view: default_view.clone(),
//                     taken: true,
//                     frames_since_last_use: 0,
//                 }]);
//                 CachedTexture {
//                     texture,
//                     default_view,
//                 }
//             }
//         }
//     }

//     /// Updates the cache and only retains recently used textures.
//     pub fn update(&mut self) {
//         for textures in self.textures.values_mut() {
//             for texture in textures.iter_mut() {
//                 texture.frames_since_last_use += 1;
//                 texture.taken = false;
//             }

//             textures.retain(|texture| texture.frames_since_last_use < 3);
//         }
//     }
// }
