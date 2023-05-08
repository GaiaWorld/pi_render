


// #[derive(Debug, Clone, Hash, PartialEq, Eq)]
// pub struct KeyTextureViewN<const N: usize>(pub [EKeyTexture;N], pub TextureViewDesc);

// pub struct TextureViewN<const N: usize> {
//     textures: [ETexture;N],
//     views: Vec<wgpu::TextureView>,
// }
// impl<const N: usize> TextureViewN<N> {
//     pub fn new(
//         textures: [ETexture;N],
//         key: &KeyTextureViewN<N>,
//     ) -> Self {
//         let mut views = vec![];
//         textures.iter().for_each(|v| {
//             views.push(v.create_view(&key.1.desc()));
//         });

//         Self { textures, views }
//     }
//     pub fn views(&self) -> &[&wgpu::TextureView] {
//         self.views.as_slice()
//     }
// }