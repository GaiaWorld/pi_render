use super::pipeline_key::PipelineKeyCalcolator;

pub fn create_target(
    format: wgpu::TextureFormat,
    blend: Option<wgpu::BlendState>,
    write_mask: wgpu::ColorWrites,
) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format,
        blend,
        write_mask,
    }
}

pub fn create_default_target() -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    }
}

pub fn gen_fragment_state_key(
    calcolator: &mut PipelineKeyCalcolator,
    target: &wgpu::ColorTargetState,
) {
    gen_texture_foramt(target.format, USE_BYTE_TEXTURE_FORMAT, calcolator);

    match target.blend {
        Some(blend) => {
            gen_blend_factor(blend.color.src_factor, USE_BYTE_BLEND_FACTOR, calcolator);
            gen_blend_factor(blend.color.dst_factor, USE_BYTE_BLEND_FACTOR, calcolator);
            gen_blend_operation(blend.color.operation, USE_BYTE_BLEND_OPERATION, calcolator);
            gen_blend_factor(blend.alpha.src_factor, USE_BYTE_BLEND_FACTOR, calcolator);
            gen_blend_factor(blend.alpha.dst_factor, USE_BYTE_BLEND_FACTOR, calcolator);
            gen_blend_operation(blend.alpha.operation, USE_BYTE_BLEND_OPERATION, calcolator);
        }
        None => {
            calcolator.key += 0;
            calcolator.use_bytes += USE_BYTE_BLEND_FACTOR;
            calcolator.use_bytes += USE_BYTE_BLEND_FACTOR;
            calcolator.use_bytes += USE_BYTE_BLEND_OPERATION;
            calcolator.use_bytes += USE_BYTE_BLEND_FACTOR;
            calcolator.use_bytes += USE_BYTE_BLEND_FACTOR;
            calcolator.use_bytes += USE_BYTE_BLEND_OPERATION;
        },
    };
    
    gen_color_writes(target.write_mask, USE_BYTE_COLOR_WRITES, calcolator);
}

pub const USE_BYTE_BLEND_FACTOR: u8 = 4;
pub const USE_BYTE_BLEND_OPERATION: u8 = 4;
pub const USE_BYTE_TEXTURE_FORMAT: u8 = 7;
pub const USE_BYTE_COLOR_WRITES: u8 = 3;

pub fn gen_texture_foramt(
    value: wgpu::TextureFormat,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = u128::pow(2, calcolator.use_bytes as u32);

    calcolator.key += value as u128 * diff;

    // calcolator.key += match value {
    //     wgpu::TextureFormat::R8Unorm => todo!(),
    //     wgpu::TextureFormat::R8Snorm => todo!(),
    //     wgpu::TextureFormat::R8Uint => todo!(),
    //     wgpu::TextureFormat::R8Sint => todo!(),
    //     wgpu::TextureFormat::R16Uint => todo!(),
    //     wgpu::TextureFormat::R16Sint => todo!(),
    //     wgpu::TextureFormat::R16Unorm => todo!(),
    //     wgpu::TextureFormat::R16Snorm => todo!(),
    //     wgpu::TextureFormat::R16Float => todo!(),
    //     wgpu::TextureFormat::Rg8Unorm => todo!(),
    //     wgpu::TextureFormat::Rg8Snorm => todo!(),
    //     wgpu::TextureFormat::Rg8Uint => todo!(),
    //     wgpu::TextureFormat::Rg8Sint => todo!(),
    //     wgpu::TextureFormat::R32Uint => todo!(),
    //     wgpu::TextureFormat::R32Sint => todo!(),
    //     wgpu::TextureFormat::R32Float => todo!(),
    //     wgpu::TextureFormat::Rg16Uint => todo!(),
    //     wgpu::TextureFormat::Rg16Sint => todo!(),
    //     wgpu::TextureFormat::Rg16Unorm => todo!(),
    //     wgpu::TextureFormat::Rg16Snorm => todo!(),
    //     wgpu::TextureFormat::Rg16Float => todo!(),
    //     wgpu::TextureFormat::Rgba8Unorm => todo!(),
    //     wgpu::TextureFormat::Rgba8UnormSrgb => todo!(),
    //     wgpu::TextureFormat::Rgba8Snorm => todo!(),
    //     wgpu::TextureFormat::Rgba8Uint => todo!(),
    //     wgpu::TextureFormat::Rgba8Sint => todo!(),
    //     wgpu::TextureFormat::Bgra8Unorm => todo!(),
    //     wgpu::TextureFormat::Bgra8UnormSrgb => todo!(),
    //     wgpu::TextureFormat::Rgb10a2Unorm => todo!(),
    //     wgpu::TextureFormat::Rg11b10Float => todo!(),
    //     wgpu::TextureFormat::Rg32Uint => todo!(),
    //     wgpu::TextureFormat::Rg32Sint => todo!(),
    //     wgpu::TextureFormat::Rg32Float => todo!(),
    //     wgpu::TextureFormat::Rgba16Uint => todo!(),
    //     wgpu::TextureFormat::Rgba16Sint => todo!(),
    //     wgpu::TextureFormat::Rgba16Unorm => todo!(),
    //     wgpu::TextureFormat::Rgba16Snorm => todo!(),
    //     wgpu::TextureFormat::Rgba16Float => todo!(),
    //     wgpu::TextureFormat::Rgba32Uint => todo!(),
    //     wgpu::TextureFormat::Rgba32Sint => todo!(),
    //     wgpu::TextureFormat::Rgba32Float => todo!(),
    //     wgpu::TextureFormat::Depth32Float => todo!(),
    //     wgpu::TextureFormat::Depth24Plus => todo!(),
    //     wgpu::TextureFormat::Depth24PlusStencil8 => todo!(),
    //     wgpu::TextureFormat::Rgb9e5Ufloat => todo!(),
    //     wgpu::TextureFormat::Bc1RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Bc1RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Bc2RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Bc2RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Bc3RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Bc3RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Bc4RUnorm => todo!(),
    //     wgpu::TextureFormat::Bc4RSnorm => todo!(),
    //     wgpu::TextureFormat::Bc5RgUnorm => todo!(),
    //     wgpu::TextureFormat::Bc5RgSnorm => todo!(),
    //     wgpu::TextureFormat::Bc6hRgbUfloat => todo!(),
    //     wgpu::TextureFormat::Bc6hRgbSfloat => todo!(),
    //     wgpu::TextureFormat::Bc7RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Bc7RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Etc2Rgb8Unorm => todo!(),
    //     wgpu::TextureFormat::Etc2Rgb8UnormSrgb => todo!(),
    //     wgpu::TextureFormat::Etc2Rgb8A1Unorm => todo!(),
    //     wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb => todo!(),
    //     wgpu::TextureFormat::Etc2Rgba8Unorm => todo!(),
    //     wgpu::TextureFormat::Etc2Rgba8UnormSrgb => todo!(),
    //     wgpu::TextureFormat::EacR11Unorm => todo!(),
    //     wgpu::TextureFormat::EacR11Snorm => todo!(),
    //     wgpu::TextureFormat::EacRg11Unorm => todo!(),
    //     wgpu::TextureFormat::EacRg11Snorm => todo!(),
    //     wgpu::TextureFormat::Astc4x4RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc4x4RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc5x4RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc5x4RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc5x5RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc5x5RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc6x5RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc6x5RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc6x6RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc6x6RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc8x5RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc8x5RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc8x6RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc8x6RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc10x5RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc10x5RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc10x6RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc10x6RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc8x8RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc8x8RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc10x8RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc10x8RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc10x10RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc10x10RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc12x10RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc12x10RgbaUnormSrgb => todo!(),
    //     wgpu::TextureFormat::Astc12x12RgbaUnorm => todo!(),
    //     wgpu::TextureFormat::Astc12x12RgbaUnormSrgb => todo!(),
    // }

    calcolator.use_bytes += use_byte;
}

pub fn gen_blend_factor(
    factor: wgpu::BlendFactor,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = u128::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match factor {
        wgpu::BlendFactor::Zero => 0 * diff,
        wgpu::BlendFactor::One => 1 * diff,
        wgpu::BlendFactor::Src => 2 * diff,
        wgpu::BlendFactor::OneMinusSrc => 3 * diff,
        wgpu::BlendFactor::SrcAlpha => 4 * diff,
        wgpu::BlendFactor::OneMinusSrcAlpha => 5 * diff,
        wgpu::BlendFactor::Dst => 6 * diff,
        wgpu::BlendFactor::OneMinusDst => 7 * diff,
        wgpu::BlendFactor::DstAlpha => 8 * diff,
        wgpu::BlendFactor::OneMinusDstAlpha => 9 * diff,
        wgpu::BlendFactor::SrcAlphaSaturated => 10 * diff,
        wgpu::BlendFactor::Constant => 11 * diff,
        wgpu::BlendFactor::OneMinusConstant => 12 * diff,
    };

    calcolator.use_bytes += use_byte;
}

pub fn gen_blend_operation(
    value: wgpu::BlendOperation,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = u128::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match value {
        wgpu::BlendOperation::Add => 0 * diff,
        wgpu::BlendOperation::Subtract => 1 * diff,
        wgpu::BlendOperation::ReverseSubtract => 2 * diff,
        wgpu::BlendOperation::Min => 3 * diff,
        wgpu::BlendOperation::Max => 4 * diff,
    };

    calcolator.use_bytes += use_byte;
}

pub fn gen_color_writes(
    value: wgpu::ColorWrites,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = u128::pow(2, calcolator.use_bytes as u32);
    calcolator.key += if value == wgpu::ColorWrites::RED {
        0 * diff
    } else if value == wgpu::ColorWrites::GREEN {
        1 * diff
    } else if value == wgpu::ColorWrites::BLUE {
        2 * diff
    } else if value == wgpu::ColorWrites::COLOR {
        3 * diff
    } else if value == wgpu::ColorWrites::ALPHA {
        4 * diff
    } else {
        5 * diff
    };

    calcolator.use_bytes += use_byte;
}