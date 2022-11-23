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

    // calcolator.key += value as u128 * diff;

    calcolator.key += match value {
        wgpu::TextureFormat::R8Unorm                => 01 * diff,
        wgpu::TextureFormat::R8Snorm                => 02 * diff,
        wgpu::TextureFormat::R8Uint                 => 03 * diff,
        wgpu::TextureFormat::R8Sint                 => 04 * diff,
        wgpu::TextureFormat::R16Uint                => 05 * diff,
        wgpu::TextureFormat::R16Sint                => 06 * diff,
        wgpu::TextureFormat::R16Unorm               => 07 * diff,
        wgpu::TextureFormat::R16Snorm               => 08 * diff,
        wgpu::TextureFormat::R16Float               => 09 * diff,
        wgpu::TextureFormat::Rg8Unorm               => 10 * diff,
        wgpu::TextureFormat::Rg8Snorm               => 11 * diff,
        wgpu::TextureFormat::Rg8Uint                => 12 * diff,
        wgpu::TextureFormat::Rg8Sint                => 13 * diff,
        wgpu::TextureFormat::R32Uint                => 14 * diff,
        wgpu::TextureFormat::R32Sint                => 15 * diff,
        wgpu::TextureFormat::R32Float               => 16 * diff,
        wgpu::TextureFormat::Rg16Uint               => 17 * diff,
        wgpu::TextureFormat::Rg16Sint               => 18 * diff,
        wgpu::TextureFormat::Rg16Unorm              => 19 * diff,
        wgpu::TextureFormat::Rg16Snorm              => 20 * diff,
        wgpu::TextureFormat::Rg16Float              => 21 * diff,
        wgpu::TextureFormat::Rgba8Unorm             => 22 * diff,
        wgpu::TextureFormat::Rgba8UnormSrgb         => 23 * diff,
        wgpu::TextureFormat::Rgba8Snorm             => 24 * diff,
        wgpu::TextureFormat::Rgba8Uint              => 25 * diff,
        wgpu::TextureFormat::Rgba8Sint              => 26 * diff,
        wgpu::TextureFormat::Bgra8Unorm             => 27 * diff,
        wgpu::TextureFormat::Bgra8UnormSrgb         => 28 * diff,
        wgpu::TextureFormat::Rgb10a2Unorm           => 29 * diff,
        wgpu::TextureFormat::Rg11b10Float           => 30 * diff,
        wgpu::TextureFormat::Rg32Uint               => 31 * diff,
        wgpu::TextureFormat::Rg32Sint               => 32 * diff,
        wgpu::TextureFormat::Rg32Float              => 33 * diff,
        wgpu::TextureFormat::Rgba16Uint             => 34 * diff,
        wgpu::TextureFormat::Rgba16Sint             => 35 * diff,
        wgpu::TextureFormat::Rgba16Unorm            => 36 * diff,
        wgpu::TextureFormat::Rgba16Snorm            => 37 * diff,
        wgpu::TextureFormat::Rgba16Float            => 38 * diff,
        wgpu::TextureFormat::Rgba32Uint             => 39 * diff,
        wgpu::TextureFormat::Rgba32Sint             => 40 * diff,
        wgpu::TextureFormat::Rgba32Float            => 41 * diff,
        wgpu::TextureFormat::Depth32Float           => 42 * diff,
        wgpu::TextureFormat::Depth24Plus            => 43 * diff,
        wgpu::TextureFormat::Depth24PlusStencil8    => 44 * diff,
        _ => 00,
    };

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