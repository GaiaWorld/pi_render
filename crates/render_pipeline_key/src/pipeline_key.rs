

/// UES_BYTE 总数不超过 127
/// depth_stencil 使用的纹理格式 信息 对应占位字节数 [4 种类型]
const UES_BYTE_DEPTH_STENCIL_FORMAT: u8 = 2; // 2

/// depth_stencil 深度测试是否开启的 信息 对应占位字节数 [2 种类型]
const UES_BYTE_DEPTH_ENABLE: u8 = 1; // 3
/// depth_stencil 深度测试比较运算符 信息 对应占位字节数 [8 种类型]
const UES_BYTE_DEPTH_COMPARE: u8 = 3; // 6

/// depth_stencil 写 信息 对应占位字节数 [low 8 bits 种类型]
const UES_BYTE_DEPTH_STENCIL_READ_MASK: u8 = 8; // 14
/// depth_stencil 读 信息 对应占位字节数 [low 8 bits 种类型]
const UES_BYTE_DEPTH_STENCIL_WRITE_MASK: u8 = 8; // 22

/// depth_stencil 模板front比较运算符 信息 对应占位字节数 [8 种类型]
const UES_BYTE_STENCIL_FRONT_COMPARE: u8 = 3; // 25 [8 种类型]
/// depth_stencil 模板front比较失败时操作 对应占位字节数
const UES_BYTE_STENCIL_FRONT_FAIL_OPERATION: u8 = 3; // 28
/// depth_stencil 模板front比较深度失败时操作 对应占位字节数 [8 种类型]
const UES_BYTE_STENCIL_FRONT_DEPTH_FAIL_OPERATION: u8 = 3; // 31
/// depth_stencil 模板front比较pass失败时操作 对应占位字节数 [8 种类型]
const UES_BYTE_STENCIL_FRONT_PASS_FAIL_OPERATION: u8 = 3; // 34

/// depth_stencil 模板 back 比较运算符 信息 对应占位字节数 [8 种类型]
const UES_BYTE_STENCIL_BACK_COMPARE: u8 = 3; // 37
/// depth_stencil 模板 back 比较失败时操作 对应占位字节数 [8 种类型]
const UES_BYTE_STENCIL_BACK_FAIL_OPERATION: u8 = 3; // 40
/// depth_stencil 模板 back 比较深度失败时操作 对应占位字节数 [8 种类型]
const UES_BYTE_STENCIL_BACK_DEPTH_FAIL_OPERATION: u8 = 3; // 43
/// depth_stencil 模板 back 比较pass失败时操作 对应占位字节数 [8 种类型]
const UES_BYTE_STENCIL_BACK_PASS_FAIL_OPERATION: u8 = 3; // 46

/// depth_stencil 偏移设置类型 对应占位字节数
/// * 外部提供确定的几种类型 最多 2^5 32
const USE_BYTE_DEPTH_STENCIL_BIAS_TEMPLATE: u8 = 5; // 51

/// depth_stencil 顶点组装拓扑类型 对应占位字节数
const USE_BYTE_TOPOLOGY: u8 = 3; // 54
/// depth_stencil 顶点组装多边形填充类型 对应占位字节数
const USE_BYTE_POLYGON_MODE: u8 = 2; // 56
/// depth_stencil 顶点序号信息格式 对应占位字节数
const USE_BYTE_INDEX_FORMAT: u8 = 2; // 58
/// depth_stencil 多边形剔除 对应占位字节数
const USE_BYTE_CULL_MODE: u8 = 2; // 60
/// depth_stencil 前向面描述 对应占位字节数
const USE_BYTE_FRONT_FACE: u8 = 1; // 61
/// depth_stencil unclipped_depth 深度剔除 对应占位字节数
const USE_BYTE_UNCLIPPED_DEPTH: u8 = 1; // 62
/// depth_stencil conservative 对应占位字节数
const USE_BYTE_CONSERVATIVE: u8 = 1; // 63

pub type PipelineKey = u128;

pub struct PipelineKeyCalcolator {
    pub key: PipelineKey,
    pub use_bytes: u8,
}

impl PipelineKeyCalcolator {
    pub fn new() -> Self {
        Self { key: 0, use_bytes: 0 }
    }
}
/// gen_pipeline_key
/// * `depth_stencil_bias_mode` DepthStencilState.bias 的设置 由外部定义一定数量的设置模板, 该值即为模板标识
/// * `depth_stencil_bias_modes_use_bite` DepthStencilState.bias 的设置 由外部定义一定数量的设置模板, 该值即描述模板数量的数值可以用多少个字节表达
pub fn gen_pipeline_key(
    calcolator: &mut PipelineKeyCalcolator,
    primitive: &wgpu::PrimitiveState,
    depth_stencil: &Option<wgpu::DepthStencilState>,
    depth_stencil_bias_mode: u8,
    depth_stencil_bias_modes_use_bite: u8,
) {
    gen_pipeline_primitive_key(calcolator, &primitive);
    gen_pipeline_depth_stencil_key(calcolator, &depth_stencil, depth_stencil_bias_mode, depth_stencil_bias_modes_use_bite);
}

pub fn gen_pipeline_primitive_key(
    calcolator: &mut PipelineKeyCalcolator,
    primitive: &wgpu::PrimitiveState,
) {
    gen_pipeline_primitive_topology_key(&primitive.topology, USE_BYTE_TOPOLOGY, calcolator);
    gen_pipeline_primitive_cull_mode_key(&primitive.cull_mode, USE_BYTE_CULL_MODE, calcolator);
    gen_pipeline_primitive_polygon_key(&primitive.polygon_mode, USE_BYTE_POLYGON_MODE, calcolator);
    gen_pipeline_primitive_front_face_key(&primitive.front_face, USE_BYTE_FRONT_FACE, calcolator);
    gen_pipeline_primitive_index_format_key(&primitive.strip_index_format, USE_BYTE_INDEX_FORMAT, calcolator);
    gen_pipeline_bool_value_key(&primitive.unclipped_depth, USE_BYTE_UNCLIPPED_DEPTH, calcolator);
    gen_pipeline_bool_value_key(&primitive.conservative, USE_BYTE_CONSERVATIVE, calcolator);
}

pub fn gen_pipeline_depth_stencil_key(
    calcolator: &mut PipelineKeyCalcolator,
    depth_stencil: &Option<wgpu::DepthStencilState>,
    depth_stencil_bias_mode: u8,
    depth_stencil_bias_modes_use_bite: u8,
) {
    match depth_stencil {
        Some(depth_stencil) => {
            gen_pipeline_depth_stencil_format_key(&depth_stencil.format, UES_BYTE_DEPTH_STENCIL_FORMAT, calcolator);
            gen_pipeline_bool_value_key(&depth_stencil.depth_write_enabled, UES_BYTE_DEPTH_ENABLE, calcolator);
            gen_pipeline_compare_function_value_key(&depth_stencil.depth_compare, UES_BYTE_DEPTH_COMPARE, calcolator);

            gen_pipeline_compare_function_value_key(&depth_stencil.stencil.front.compare, UES_BYTE_STENCIL_FRONT_COMPARE, calcolator);
            gen_pipeline_stencil_operation_value_key(&depth_stencil.stencil.front.fail_op, UES_BYTE_STENCIL_FRONT_FAIL_OPERATION, calcolator);
            gen_pipeline_stencil_operation_value_key(&depth_stencil.stencil.front.depth_fail_op, UES_BYTE_STENCIL_FRONT_DEPTH_FAIL_OPERATION, calcolator);
            gen_pipeline_stencil_operation_value_key(&depth_stencil.stencil.front.pass_op, UES_BYTE_STENCIL_FRONT_PASS_FAIL_OPERATION, calcolator);
            
            gen_pipeline_compare_function_value_key(&depth_stencil.stencil.back.compare, UES_BYTE_STENCIL_BACK_COMPARE, calcolator);
            gen_pipeline_stencil_operation_value_key(&depth_stencil.stencil.back.fail_op, UES_BYTE_STENCIL_BACK_FAIL_OPERATION, calcolator);
            gen_pipeline_stencil_operation_value_key(&depth_stencil.stencil.back.depth_fail_op, UES_BYTE_STENCIL_BACK_DEPTH_FAIL_OPERATION, calcolator);
            gen_pipeline_stencil_operation_value_key(&depth_stencil.stencil.back.pass_op, UES_BYTE_STENCIL_BACK_PASS_FAIL_OPERATION, calcolator);

            gen_pipeline_depth_stencil_bias_key(depth_stencil_bias_mode, depth_stencil_bias_modes_use_bite, calcolator);
        },
        None => {
            calcolator.key += 0;

            calcolator.use_bytes += UES_BYTE_DEPTH_STENCIL_FORMAT;
            calcolator.use_bytes += UES_BYTE_DEPTH_ENABLE;
            calcolator.use_bytes += UES_BYTE_DEPTH_COMPARE;

            calcolator.use_bytes += UES_BYTE_STENCIL_FRONT_COMPARE;
            calcolator.use_bytes += UES_BYTE_STENCIL_FRONT_FAIL_OPERATION;
            calcolator.use_bytes += UES_BYTE_STENCIL_FRONT_DEPTH_FAIL_OPERATION;
            calcolator.use_bytes += UES_BYTE_STENCIL_FRONT_PASS_FAIL_OPERATION;
            
            calcolator.use_bytes += UES_BYTE_STENCIL_BACK_COMPARE;
            calcolator.use_bytes += UES_BYTE_STENCIL_BACK_FAIL_OPERATION;
            calcolator.use_bytes += UES_BYTE_STENCIL_BACK_DEPTH_FAIL_OPERATION;
            calcolator.use_bytes += UES_BYTE_STENCIL_BACK_PASS_FAIL_OPERATION;
            
            calcolator.use_bytes += depth_stencil_bias_modes_use_bite;
        },
    }
}

pub fn gen_pipeline_primitive_topology_key(
    topology: &wgpu::PrimitiveTopology,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match topology {
        wgpu::PrimitiveTopology::PointList => 0 * diff,
        wgpu::PrimitiveTopology::LineList => 1 * diff,
        wgpu::PrimitiveTopology::LineStrip => 2 * diff,
        wgpu::PrimitiveTopology::TriangleList => 3 * diff,
        wgpu::PrimitiveTopology::TriangleStrip => 4 * diff,
    };

    calcolator.use_bytes += use_byte;
}

pub fn gen_pipeline_primitive_cull_mode_key(
    cull_mode: &Option<wgpu::Face>,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match cull_mode {
        Some(mode) => match mode {
            wgpu::Face::Front => 1 * diff,
            wgpu::Face::Back => 2 * diff,
        },
        None => {
            0 * diff
        },
    };

    calcolator.use_bytes += use_byte;
}

pub fn gen_pipeline_primitive_polygon_key(
    polygon: &wgpu::PolygonMode,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match polygon {
        wgpu::PolygonMode::Fill => 0 * diff,
        wgpu::PolygonMode::Line => 1 * diff,
        wgpu::PolygonMode::Point => 2 * diff,
    };

    calcolator.use_bytes += use_byte;
}

pub fn gen_pipeline_primitive_index_format_key(
    strip_index_format: &Option<wgpu::IndexFormat>,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match strip_index_format {
        Some(format) => match format {
            wgpu::IndexFormat::Uint16 => {
                1 * diff
            },
            wgpu::IndexFormat::Uint32 => {
                2 * diff
            },
        },
        None => {
            0 * diff
        },
    };

    calcolator.use_bytes += use_byte;
}

pub fn gen_pipeline_primitive_front_face_key(
    front_face: &wgpu::FrontFace,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match front_face {
        wgpu::FrontFace::Ccw => 0 * diff,
        wgpu::FrontFace::Cw => 1 * diff,
    };

    calcolator.use_bytes += use_byte;
}

pub fn gen_pipeline_depth_stencil_format_key(
    value: &wgpu::TextureFormat,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match value {
        wgpu::TextureFormat::Depth32Float => 1 * diff,
        wgpu::TextureFormat::Depth24Plus => 2 * diff,
        wgpu::TextureFormat::Depth24PlusStencil8 => 3 * diff,
        _ => 0,
    };

    calcolator.use_bytes += use_byte;
}

pub fn gen_pipeline_read_write_mask_key(
    value: usize,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    let r = (value & 0b0000_0000_0000_0000_0000_0000_1111_1111) as PipelineKey;
    calcolator.key += r * diff;

    calcolator.use_bytes += use_byte;
}


pub fn gen_pipeline_depth_stencil_bias_key(
    value: u8,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    let r = value as PipelineKey;
    calcolator.key += r * diff;

    calcolator.use_bytes += use_byte;
}

pub fn gen_pipeline_compare_function_value_key(
    value: &wgpu::CompareFunction,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match value {
        wgpu::CompareFunction::Never => 0 * diff,
        wgpu::CompareFunction::Less => 1 * diff,
        wgpu::CompareFunction::Equal => 2 * diff,
        wgpu::CompareFunction::LessEqual => 3 * diff,
        wgpu::CompareFunction::Greater => 4 * diff,
        wgpu::CompareFunction::NotEqual => 5 * diff,
        wgpu::CompareFunction::GreaterEqual => 6 * diff,
        wgpu::CompareFunction::Always => 7 * diff,
    };

    calcolator.use_bytes += use_byte;
}

pub fn gen_pipeline_stencil_operation_value_key(
    value: &wgpu::StencilOperation,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match value {
        wgpu::StencilOperation::Keep => 0 * diff,
        wgpu::StencilOperation::Zero => 1 * diff,
        wgpu::StencilOperation::Replace => 2 * diff,
        wgpu::StencilOperation::Invert => 3 * diff,
        wgpu::StencilOperation::IncrementClamp => 4 * diff,
        wgpu::StencilOperation::DecrementClamp => 5 * diff,
        wgpu::StencilOperation::IncrementWrap => 6 * diff,
        wgpu::StencilOperation::DecrementWrap => 7 * diff,
    };

    calcolator.use_bytes += use_byte;
}

pub fn gen_pipeline_bool_value_key(
    value: &bool,
    use_byte: u8,
    calcolator: &mut PipelineKeyCalcolator,
) {
    let diff = PipelineKey::pow(2, calcolator.use_bytes as u32);
    calcolator.key += match value {
        true => 0 * diff,
        false => 1 * diff,
    };

    calcolator.use_bytes += use_byte;
}