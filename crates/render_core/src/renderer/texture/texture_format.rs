
pub trait TTextureFormatPixelByte {
    fn pixel_bytes(&self) -> usize;
}
impl TTextureFormatPixelByte for wgpu::TextureFormat {
    fn pixel_bytes(&self) -> usize {
        match self {
            wgpu::TextureFormat::R8Unorm        => 01,
            wgpu::TextureFormat::R8Snorm        => 01,
            wgpu::TextureFormat::R8Uint         => 01,
            wgpu::TextureFormat::R8Sint         => 01,
            wgpu::TextureFormat::R16Uint        => 02,
            wgpu::TextureFormat::R16Sint        => 02,
            wgpu::TextureFormat::R16Unorm       => 02,
            wgpu::TextureFormat::R16Snorm       => 02,
            wgpu::TextureFormat::R16Float       => 02,
            wgpu::TextureFormat::Rg8Unorm       => 02,
            wgpu::TextureFormat::Rg8Snorm       => 02,
            wgpu::TextureFormat::Rg8Uint        => 02,
            wgpu::TextureFormat::Rg8Sint        => 02,
            wgpu::TextureFormat::R32Uint        => 04,
            wgpu::TextureFormat::R32Sint        => 04,
            wgpu::TextureFormat::R32Float       => 04,
            wgpu::TextureFormat::Rg16Uint       => 04,
            wgpu::TextureFormat::Rg16Sint       => 04,
            wgpu::TextureFormat::Rg16Unorm      => 04,
            wgpu::TextureFormat::Rg16Snorm      => 04,
            wgpu::TextureFormat::Rg16Float      => 04,
            wgpu::TextureFormat::Rgba8Unorm     => 04,
            wgpu::TextureFormat::Rgba8UnormSrgb => 04,
            wgpu::TextureFormat::Rgba8Snorm     => 04,
            wgpu::TextureFormat::Rgba8Uint      => 04,
            wgpu::TextureFormat::Rgba8Sint      => 04,
            wgpu::TextureFormat::Bgra8Unorm     => 04,
            wgpu::TextureFormat::Bgra8UnormSrgb => 04,
            wgpu::TextureFormat::Rgb10a2Unorm   => 04,
            wgpu::TextureFormat::Rg11b10Float   => 04,
            wgpu::TextureFormat::Rg32Uint       => 08,
            wgpu::TextureFormat::Rg32Sint       => 08,
            wgpu::TextureFormat::Rg32Float      => 08,
            wgpu::TextureFormat::Rgba16Uint     => 08,
            wgpu::TextureFormat::Rgba16Sint     => 08,
            wgpu::TextureFormat::Rgba16Unorm    => 08,
            wgpu::TextureFormat::Rgba16Snorm    => 08,
            wgpu::TextureFormat::Rgba16Float    => 08,
            wgpu::TextureFormat::Rgba32Uint     => 16,
            wgpu::TextureFormat::Rgba32Sint     => 16,
            wgpu::TextureFormat::Rgba32Float    => 16,
            wgpu::TextureFormat::Depth32Float           => 04,
            wgpu::TextureFormat::Depth32FloatStencil8   => 05,
            wgpu::TextureFormat::Depth24Plus            => 03,
            wgpu::TextureFormat::Depth24PlusStencil8    => 04,
            wgpu::TextureFormat::Depth24UnormStencil8   => 04,
            wgpu::TextureFormat::Rgb9e5Ufloat           => 04,

            wgpu::TextureFormat::Bc1RgbaUnorm           => 01,
            wgpu::TextureFormat::Bc1RgbaUnormSrgb       => 01,
            wgpu::TextureFormat::Bc2RgbaUnorm           => 01,
            wgpu::TextureFormat::Bc2RgbaUnormSrgb       => 01,
            wgpu::TextureFormat::Bc3RgbaUnorm           => 01,
            wgpu::TextureFormat::Bc3RgbaUnormSrgb       => 01,
            wgpu::TextureFormat::Bc4RUnorm              => 01,
            wgpu::TextureFormat::Bc4RSnorm              => 01,
            wgpu::TextureFormat::Bc5RgUnorm             => 01,
            wgpu::TextureFormat::Bc5RgSnorm             => 01,
            wgpu::TextureFormat::Bc6hRgbUfloat          => 01,
            wgpu::TextureFormat::Bc6hRgbSfloat          => 01,
            wgpu::TextureFormat::Bc7RgbaUnorm           => 01,
            wgpu::TextureFormat::Bc7RgbaUnormSrgb       => 01,
            wgpu::TextureFormat::Etc2Rgb8Unorm          => 01,
            wgpu::TextureFormat::Etc2Rgb8UnormSrgb      => 01,
            wgpu::TextureFormat::Etc2Rgb8A1Unorm        => 01,
            wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb    => 01,
            wgpu::TextureFormat::Etc2Rgba8Unorm         => 01,
            wgpu::TextureFormat::Etc2Rgba8UnormSrgb     => 01,
            wgpu::TextureFormat::EacR11Unorm            => 01,
            wgpu::TextureFormat::EacR11Snorm            => 01,
            wgpu::TextureFormat::EacRg11Unorm           => 01,
            wgpu::TextureFormat::EacRg11Snorm           => 01,
            wgpu::TextureFormat::Astc { block, channel } => 01,
        }
    }
}
