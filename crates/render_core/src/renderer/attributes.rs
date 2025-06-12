use pi_atom::Atom;

use super::{buildin_var::{ShaderVarVertices, ShaderVarUniform}, vertex_buffer_desc::VertexBufferDesc};


pub trait TVertexFormatShaderCode {
    fn shader_code(&self) -> String;
}

impl TVertexFormatShaderCode for wgpu::VertexFormat {
    fn shader_code(&self) -> String {
        match self {
            wgpu::VertexFormat::Uint8x2     => String::from("uvec2"),
            wgpu::VertexFormat::Uint8x4     => String::from("uvec4"),
            wgpu::VertexFormat::Sint8x2     => String::from("ivec2"),
            wgpu::VertexFormat::Sint8x4     => String::from("ivec4"),
            wgpu::VertexFormat::Unorm8x2    => String::from("vec2"),
            wgpu::VertexFormat::Unorm8x4    => String::from("vec4"),
            wgpu::VertexFormat::Snorm8x2    => String::from("vec2"),
            wgpu::VertexFormat::Snorm8x4    => String::from("vec4"),
            wgpu::VertexFormat::Uint16x2    => String::from("uvec2"),
            wgpu::VertexFormat::Uint16x4    => String::from("uvec4"),
            wgpu::VertexFormat::Sint16x2    => String::from("ivec2"),
            wgpu::VertexFormat::Sint16x4    => String::from("ivec4"),
            wgpu::VertexFormat::Unorm16x2   => String::from("vec2"),
            wgpu::VertexFormat::Unorm16x4   => String::from("vec4"),
            wgpu::VertexFormat::Snorm16x2   => String::from("vec2"),
            wgpu::VertexFormat::Snorm16x4   => String::from("vec4"),
            wgpu::VertexFormat::Float16x2   => String::from("vec2"),
            wgpu::VertexFormat::Float16x4   => String::from("vec4"),
            wgpu::VertexFormat::Float32     => String::from("float"),
            wgpu::VertexFormat::Float32x2   => String::from("vec2"),
            wgpu::VertexFormat::Float32x3   => String::from("vec3"),
            wgpu::VertexFormat::Float32x4   => String::from("vec4"),
            wgpu::VertexFormat::Uint32      => String::from("uint"),
            wgpu::VertexFormat::Uint32x2    => String::from("uvec2"),
            wgpu::VertexFormat::Uint32x3    => String::from("uvec3"),
            wgpu::VertexFormat::Uint32x4    => String::from("uvec4"),
            wgpu::VertexFormat::Sint32      => String::from("int"),
            wgpu::VertexFormat::Sint32x2    => String::from("ivec2"),
            wgpu::VertexFormat::Sint32x3    => String::from("ivec3"),
            wgpu::VertexFormat::Sint32x4    => String::from("ivec4"),
            wgpu::VertexFormat::Float64     => String::from(""),
            wgpu::VertexFormat::Float64x2   => String::from(""),
            wgpu::VertexFormat::Float64x3   => String::from(""),
            wgpu::VertexFormat::Float64x4   => String::from(""),
            wgpu::VertexFormat::Unorm10_10_10_2 => String::from(""),
            wgpu::VertexFormat::Unorm8x4Bgra    => String::from(""),
            wgpu::VertexFormat::Uint8           => String::from(""),
            wgpu::VertexFormat::Sint8           => String::from(""),
            wgpu::VertexFormat::Unorm8          => String::from(""),
            wgpu::VertexFormat::Snorm8          => String::from(""),
            wgpu::VertexFormat::Uint16          => String::from(""),
            wgpu::VertexFormat::Sint16          => String::from(""),
            wgpu::VertexFormat::Unorm16         => String::from(""),
            wgpu::VertexFormat::Snorm16         => String::from(""),
            wgpu::VertexFormat::Float16         => String::from(""),
        }
    }
}

///
/// 预留为支持 64 种顶点数据
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EVertexDataKind {
    Position               ,
    Position2D             ,
    Color4                 ,
    UV                     ,
    Normal                 ,
    Tangent                ,
    MatricesIndices        ,
    MatricesWeights        ,
    MatricesIndicesExtra   ,
    MatricesWeightsExtra   ,
    UV2                    ,
    UV3                    ,
    UV4                    ,
    UV5                    ,
    UV6                    ,
    // CustomVec4A            ,
    // CustomVec4B            ,
    // CustomVec4C            ,
    // CustomVec4D            ,
    // CustomVec3A            ,
    // CustomVec3B            ,
    // CustomVec2A            ,
    // CustomVec2B            ,
    // InsIndex               ,
    InsWorldRow1           ,
    InsWorldRow2           ,
    InsWorldRow3           ,
    InsWorldRow4           ,
    // InsColor               ,
    // InsTillOffset1         ,
    // InsTillOffset2         ,
    // InsCustomVec4A         ,
    // InsCustomVec4B         ,
    // InsCustomVec4C         ,
    // InsCustomVec4D         ,

    // InsVec3A         ,
    // InsVec3B         ,
    // InsVec3C         ,
    // InsVec3D         ,
    // InsVec3E         ,
    // InsVec3F         ,
    // InsVec3G         ,
    // InsVec3H         ,

    // InsVec4A         ,
    // InsVec4B         ,
    // InsVec4C         ,
    // InsVec4D         ,
    // InsVec4E         ,
    // InsVec4F         ,
    // InsVec4G         ,
    // InsVec4H         ,

    // InsCustomUVec4A        ,
    // InsCustomIVec4B        ,

    // MatricesIndices1       ,
    // MatricesWeights1       ,

    // MatricesIndices2       ,
    // MatricesWeights2       ,
    // MatricesIndicesExtra2  ,
    // MatricesWeightsExtra2  ,

    // MatricesIndices3       ,
    // MatricesWeights3       ,
    // MatricesIndicesExtra3  ,
    // MatricesWeightsExtra3  ,

    InsVelocity            ,
    // ParticleAgeLife        ,
    // ParticlePosition        ,
    // ParticleScaling         ,
    // ParticleRotation        ,
    // ParticleDirection       ,
    // ParticleColor           ,
    // ParticleTilloff         ,
    TrailAxisX              ,
    TrailAxisZ              ,
    TrailInfo               ,
}

impl EVertexDataKind {
    pub fn var_code(&self) -> &str {
        match self {
            EVertexDataKind::Position               => ShaderVarVertices::POSITION                  ,
            EVertexDataKind::Position2D             => ShaderVarVertices::POSITION2D                ,
            EVertexDataKind::Color4                 => ShaderVarVertices::COLOR4                    ,
            EVertexDataKind::UV                     => ShaderVarVertices::UV                        ,
            EVertexDataKind::Normal                 => ShaderVarVertices::NORMAL                    ,
            EVertexDataKind::Tangent                => ShaderVarVertices::TANGENT                   ,
            EVertexDataKind::MatricesIndices        => ShaderVarVertices::MATRICES_INDICES          ,
            EVertexDataKind::MatricesWeights        => ShaderVarVertices::MATRICES_WEIGHTS          ,
            EVertexDataKind::MatricesIndicesExtra   => ShaderVarVertices::MATRICES_INDICES_EXTRA    ,
            EVertexDataKind::MatricesWeightsExtra   => ShaderVarVertices::MATRICES_WEIGHTS_EXTRA    ,
            EVertexDataKind::UV2                    => ShaderVarVertices::UV2                       ,
            EVertexDataKind::UV3                    => ShaderVarVertices::UV3                       ,
            EVertexDataKind::UV4                    => ShaderVarVertices::UV4                       ,
            EVertexDataKind::UV5                    => ShaderVarVertices::UV5                       ,
            EVertexDataKind::UV6                    => ShaderVarVertices::UV6                       ,
            // EVertexDataKind::CustomVec4A            => ShaderVarVertices::CUSTOM_VEC4_A             ,
            // EVertexDataKind::CustomVec4B            => ShaderVarVertices::CUSTOM_VEC4_B             ,
            // EVertexDataKind::CustomVec4C            => ShaderVarVertices::CUSTOM_VEC4_C             ,
            // EVertexDataKind::CustomVec4D            => ShaderVarVertices::CUSTOM_VEC4_D             ,
            // EVertexDataKind::CustomVec3A            => ShaderVarVertices::CUSTOM_VEC3_A             ,
            // EVertexDataKind::CustomVec3B            => ShaderVarVertices::CUSTOM_VEC3_B             ,
            // EVertexDataKind::CustomVec2A            => ShaderVarVertices::CUSTOM_VEC2_A             ,
            // EVertexDataKind::CustomVec2B            => ShaderVarVertices::CUSTOM_VEC2_B             ,
            // EVertexDataKind::InsIndex               => ShaderVarVertices::INSTANCE_INDEX            ,
            EVertexDataKind::InsWorldRow1           => ShaderVarVertices::INS_WORLD_ROW1            ,
            EVertexDataKind::InsWorldRow2           => ShaderVarVertices::INS_WORLD_ROW2            ,
            EVertexDataKind::InsWorldRow3           => ShaderVarVertices::INS_WORLD_ROW3            ,
            EVertexDataKind::InsWorldRow4           => ShaderVarVertices::INS_WORLD_ROW4            ,
            // EVertexDataKind::InsColor               => ShaderVarVertices::INS_COLOR                 ,
            // EVertexDataKind::InsTillOffset1         => ShaderVarVertices::INS_TILL_OFFSET1          ,
            // EVertexDataKind::InsTillOffset2         => ShaderVarVertices::INS_TILL_OFFSET2          ,
            // EVertexDataKind::InsCustomVec4A         => ShaderVarVertices::INS_CUSTOM_VEC4_A         ,
            // EVertexDataKind::InsCustomVec4B         => ShaderVarVertices::INS_CUSTOM_VEC4_B         ,
            // EVertexDataKind::InsCustomVec4C         => ShaderVarVertices::INS_CUSTOM_VEC4_C         ,
            // EVertexDataKind::InsCustomVec4D         => ShaderVarVertices::INS_CUSTOM_VEC4_D         ,
            // EVertexDataKind::InsCustomUVec4A        => ShaderVarVertices::INS_CUSTOM_UVEC4_A        ,
            // EVertexDataKind::InsCustomIVec4B        => ShaderVarVertices::INS_CUSTOM_IVEC4_B        ,
            // EVertexDataKind::MatricesIndices1       => ShaderVarVertices::MATRICES_INDICES1         ,
            // EVertexDataKind::MatricesWeights1       => ShaderVarVertices::MATRICES_WEIGHTS1         ,
            // EVertexDataKind::MatricesIndices2       => ShaderVarVertices::MATRICES_INDICES2         ,
            // EVertexDataKind::MatricesWeights2       => ShaderVarVertices::MATRICES_WEIGHTS2         ,
            // EVertexDataKind::MatricesIndicesExtra2  => ShaderVarVertices::MATRICES_INDICES_EXTRA2   ,
            // EVertexDataKind::MatricesWeightsExtra2  => ShaderVarVertices::MATRICES_WEIGHTS_EXTRA2   ,
            // EVertexDataKind::MatricesIndices3       => ShaderVarVertices::MATRICES_INDICES3         ,
            // EVertexDataKind::MatricesWeights3       => ShaderVarVertices::MATRICES_WEIGHTS3         ,
            // EVertexDataKind::MatricesIndicesExtra3  => ShaderVarVertices::MATRICES_INDICES_EXTRA3   ,
            // EVertexDataKind::MatricesWeightsExtra3  => ShaderVarVertices::MATRICES_WEIGHTS_EXTRA3   ,
            EVertexDataKind::InsVelocity            => ShaderVarVertices::INS_VELOCITY              ,
            // EVertexDataKind::ParticleAgeLife        => ShaderVarVertices::PARTICLE_AGE_LIFE         ,
            // EVertexDataKind::ParticlePosition       => ShaderVarVertices::PARTICLE_POSITION         ,
            // EVertexDataKind::ParticleScaling        => ShaderVarVertices::PARTICLE_SCALING          ,
            // EVertexDataKind::ParticleRotation       => ShaderVarVertices::PARTICLE_ROTATION         ,
            // EVertexDataKind::ParticleDirection      => ShaderVarVertices::PARTICLE_DIRECTION        ,
            // EVertexDataKind::ParticleColor          => ShaderVarVertices::PARTICLE_COLOR            ,
            // EVertexDataKind::ParticleTilloff        => ShaderVarVertices::PARTICLE_TILLOFF          ,
            EVertexDataKind::TrailAxisX             => ShaderVarVertices::TRAIL_AXIS_X          ,
            EVertexDataKind::TrailAxisZ             => ShaderVarVertices::TRAIL_AXIS_Z          ,
            EVertexDataKind::TrailInfo              => ShaderVarVertices::TRAIL_INFO          ,
            // EVertexDataKind::InsVec3A               => ShaderVarVertices::INS_VEC4_A,
            // EVertexDataKind::InsVec3B               => ShaderVarVertices::INS_VEC4_B,
            // EVertexDataKind::InsVec3C               => ShaderVarVertices::INS_VEC4_C,
            // EVertexDataKind::InsVec3D               => ShaderVarVertices::INS_VEC4_D,
            // EVertexDataKind::InsVec3E               => ShaderVarVertices::INS_VEC4_E,
            // EVertexDataKind::InsVec3F               => ShaderVarVertices::INS_VEC4_F,
            // EVertexDataKind::InsVec3G               => ShaderVarVertices::INS_VEC4_G,
            // EVertexDataKind::InsVec3H               => ShaderVarVertices::INS_VEC4_H,
            // EVertexDataKind::InsVec4A               => ShaderVarVertices::INS_VEC3_A,
            // EVertexDataKind::InsVec4B               => ShaderVarVertices::INS_VEC3_B,
            // EVertexDataKind::InsVec4C               => ShaderVarVertices::INS_VEC3_C,
            // EVertexDataKind::InsVec4D               => ShaderVarVertices::INS_VEC3_D,
            // EVertexDataKind::InsVec4E               => ShaderVarVertices::INS_VEC3_E,
            // EVertexDataKind::InsVec4F               => ShaderVarVertices::INS_VEC3_F,
            // EVertexDataKind::InsVec4G               => ShaderVarVertices::INS_VEC3_G,
            // EVertexDataKind::InsVec4H               => ShaderVarVertices::INS_VEC3_H,
        }
    }
    pub fn kind(&self) -> &str {
        match self {
            EVertexDataKind::Position               => "vec3"       ,
            EVertexDataKind::Position2D             => "vec2"       ,
            EVertexDataKind::Color4                 => "vec4"       ,
            EVertexDataKind::UV                     => "vec2"       ,
            EVertexDataKind::Normal                 => "vec3"       ,
            EVertexDataKind::Tangent                => "vec4"       ,
            EVertexDataKind::MatricesIndices        => "uvec4"      ,
            EVertexDataKind::MatricesWeights        => "vec4"       ,
            EVertexDataKind::MatricesIndicesExtra   => "uvec4"      ,
            EVertexDataKind::MatricesWeightsExtra   => "vec4"       ,
            EVertexDataKind::UV2                    => "vec2"       ,
            EVertexDataKind::UV3                    => "vec2"       ,
            EVertexDataKind::UV4                    => "vec2"       ,
            EVertexDataKind::UV5                    => "vec2"       ,
            EVertexDataKind::UV6                    => "vec2"       ,
            // EVertexDataKind::CustomVec4A            => "vec4"       ,
            // EVertexDataKind::CustomVec4B            => "vec4"       ,
            // EVertexDataKind::CustomVec4C            => "vec4"       ,
            // EVertexDataKind::CustomVec4D            => "vec4"       ,
            // EVertexDataKind::CustomVec3A            => "vec3"       ,
            // EVertexDataKind::CustomVec3B            => "vec3"       ,
            // EVertexDataKind::CustomVec2A            => "vec2"       ,
            // EVertexDataKind::CustomVec2B            => "vec2"       ,
            // EVertexDataKind::InsIndex               => "uint"       ,
            EVertexDataKind::InsWorldRow1           => "vec4"       ,
            EVertexDataKind::InsWorldRow2           => "vec4"       ,
            EVertexDataKind::InsWorldRow3           => "vec4"       ,
            EVertexDataKind::InsWorldRow4           => "vec4"       ,
            // EVertexDataKind::InsColor               => "vec4"       ,
            // EVertexDataKind::InsTillOffset1         => "vec4"       ,
            // EVertexDataKind::InsTillOffset2         => "vec4"       ,
            // EVertexDataKind::InsCustomVec4A         => "vec4"       ,
            // EVertexDataKind::InsCustomVec4B         => "vec4"       ,
            // EVertexDataKind::InsCustomVec4C         => "vec4"       ,
            // EVertexDataKind::InsCustomVec4D         => "vec4"       ,
            // EVertexDataKind::InsCustomUVec4A        => "uvec4"      ,
            // EVertexDataKind::InsCustomIVec4B        => "ivec4"      ,
            // EVertexDataKind::MatricesIndices1       => "uvec2"      ,
            // EVertexDataKind::MatricesWeights1       => "float"      ,
            // EVertexDataKind::MatricesIndices2       => "uvec2"      ,
            // EVertexDataKind::MatricesWeights2       => "vec2"       ,
            // EVertexDataKind::MatricesIndicesExtra2  => "uvec2"      ,
            // EVertexDataKind::MatricesWeightsExtra2  => "vec2"       ,
            // EVertexDataKind::MatricesIndices3       => "uvec3"      ,
            // EVertexDataKind::MatricesWeights3       => "vec3"       ,
            // EVertexDataKind::MatricesIndicesExtra3  => "uvec3"      ,
            // EVertexDataKind::MatricesWeightsExtra3  => "vec3"       ,
            EVertexDataKind::InsVelocity            => "vec4"       ,
            // EVertexDataKind::ParticleAgeLife        => "vec2"       ,
            // EVertexDataKind::ParticlePosition       => "vec3"       ,
            // EVertexDataKind::ParticleScaling        => "vec3"       ,
            // EVertexDataKind::ParticleRotation       => "vec4"       ,
            // EVertexDataKind::ParticleDirection      => "vec3"       ,
            // EVertexDataKind::ParticleColor          => "vec4"       ,
            // EVertexDataKind::ParticleTilloff        => "vec4"       ,
            EVertexDataKind::TrailAxisX             => "vec3"       ,
            EVertexDataKind::TrailAxisZ             => "vec3"       ,
            EVertexDataKind::TrailInfo              => "vec2"       ,
            // EVertexDataKind::InsVec3A               => "vec3"       ,
            // EVertexDataKind::InsVec3B               => "vec3"       ,
            // EVertexDataKind::InsVec3C               => "vec3"       ,
            // EVertexDataKind::InsVec3D               => "vec3"       ,
            // EVertexDataKind::InsVec3E               => "vec3"       ,
            // EVertexDataKind::InsVec3F               => "vec3"       ,
            // EVertexDataKind::InsVec3G               => "vec3"       ,
            // EVertexDataKind::InsVec3H               => "vec3"       ,
            // EVertexDataKind::InsVec4A               => "vec4"       ,
            // EVertexDataKind::InsVec4B               => "vec4"       ,
            // EVertexDataKind::InsVec4C               => "vec4"       ,
            // EVertexDataKind::InsVec4D               => "vec4"       ,
            // EVertexDataKind::InsVec4E               => "vec4"       ,
            // EVertexDataKind::InsVec4F               => "vec4"       ,
            // EVertexDataKind::InsVec4G               => "vec4"       ,
            // EVertexDataKind::InsVec4H               => "vec4"       ,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EBuildinVertexAtribute {
    Position                ,
    Position2D              ,
    Color4                  ,
    UV                      ,
    Normal                  ,
    Tangent                 ,
    MatricesIndices         ,
    MatricesWeights         ,
    MatricesIndicesExtra    ,
    MatricesWeightsExtra    ,
    UV2                     ,
    UV3                     ,
    UV4                     ,
    UV5                     ,
    UV6                     ,
    Trail                   ,
    TrailBillboard          ,
    TrailAxisX              ,
    TrailAxisZ              ,
    InsWorldRow1            ,
    InsWorldRow2            ,
    InsWorldRow3            ,
    InsWorldRow4            ,
    MatIdxs              ,
    ModelMaterialSkin       ,
}
impl EBuildinVertexAtribute {
    pub fn format(&self) -> wgpu::VertexFormat {
        match self {
            EBuildinVertexAtribute::Position => wgpu::VertexFormat::Float32x3,
            EBuildinVertexAtribute::Position2D => wgpu::VertexFormat::Float32x2,
            EBuildinVertexAtribute::Color4 => wgpu::VertexFormat::Float32x4,
            EBuildinVertexAtribute::UV => wgpu::VertexFormat::Float32x2,
            EBuildinVertexAtribute::Normal => wgpu::VertexFormat::Float32x3,
            EBuildinVertexAtribute::Tangent => wgpu::VertexFormat::Float32x4,
            EBuildinVertexAtribute::MatricesIndices => wgpu::VertexFormat::Uint16x4,
            EBuildinVertexAtribute::MatricesWeights => wgpu::VertexFormat::Float32x4,
            EBuildinVertexAtribute::MatricesIndicesExtra => wgpu::VertexFormat::Uint16x4,
            EBuildinVertexAtribute::MatricesWeightsExtra => wgpu::VertexFormat::Float32x4,
            EBuildinVertexAtribute::UV2 => wgpu::VertexFormat::Float32x2,
            EBuildinVertexAtribute::UV3 => wgpu::VertexFormat::Float32x2,
            EBuildinVertexAtribute::UV4 => wgpu::VertexFormat::Float32x2,
            EBuildinVertexAtribute::UV5 => wgpu::VertexFormat::Float32x2,
            EBuildinVertexAtribute::UV6 => wgpu::VertexFormat::Float32x2,
            EBuildinVertexAtribute::Trail => wgpu::VertexFormat::Float32x2,
            EBuildinVertexAtribute::TrailBillboard => wgpu::VertexFormat::Float32x2,
            EBuildinVertexAtribute::TrailAxisX => wgpu::VertexFormat::Float32x3,
            EBuildinVertexAtribute::TrailAxisZ => wgpu::VertexFormat::Float32x3,
            EBuildinVertexAtribute::InsWorldRow1 => wgpu::VertexFormat::Float32x4,
            EBuildinVertexAtribute::InsWorldRow2 => wgpu::VertexFormat::Float32x4,
            EBuildinVertexAtribute::InsWorldRow3 => wgpu::VertexFormat::Float32x4,
            EBuildinVertexAtribute::InsWorldRow4 => wgpu::VertexFormat::Float32x4,
            EBuildinVertexAtribute::MatIdxs => wgpu::VertexFormat::Uint32x4,
            EBuildinVertexAtribute::ModelMaterialSkin => wgpu::VertexFormat::Uint32x4,
        }
    }
    pub fn kind(&self) -> String {
        self.format().shader_code()
    }
    pub fn var_code(&self) -> &str {
        match self {
            EBuildinVertexAtribute::Position                => ShaderVarVertices::POSITION,
            EBuildinVertexAtribute::Position2D              => ShaderVarVertices::POSITION2D,
            EBuildinVertexAtribute::Color4                  => ShaderVarVertices::COLOR4,
            EBuildinVertexAtribute::UV                      => ShaderVarVertices::UV,
            EBuildinVertexAtribute::Normal                  => ShaderVarVertices::NORMAL,
            EBuildinVertexAtribute::Tangent                 => ShaderVarVertices::TANGENT,
            EBuildinVertexAtribute::MatricesIndices         => ShaderVarVertices::MATRICES_INDICES,
            EBuildinVertexAtribute::MatricesWeights         => ShaderVarVertices::MATRICES_WEIGHTS,
            EBuildinVertexAtribute::MatricesIndicesExtra    => ShaderVarVertices::MATRICES_INDICES_EXTRA,
            EBuildinVertexAtribute::MatricesWeightsExtra    => ShaderVarVertices::MATRICES_WEIGHTS_EXTRA,
            EBuildinVertexAtribute::UV2                     => ShaderVarVertices::UV2,
            EBuildinVertexAtribute::UV3                     => ShaderVarVertices::UV3,
            EBuildinVertexAtribute::UV4                     => ShaderVarVertices::UV4,
            EBuildinVertexAtribute::UV5                     => ShaderVarVertices::UV5,
            EBuildinVertexAtribute::UV6                     => ShaderVarVertices::UV6,
            EBuildinVertexAtribute::Trail                   => ShaderVarVertices::TRAIL_INFO,
            EBuildinVertexAtribute::TrailBillboard          => ShaderVarVertices::TRAIL_INFO,
            EBuildinVertexAtribute::TrailAxisX              => ShaderVarVertices::TRAIL_AXIS_X,
            EBuildinVertexAtribute::TrailAxisZ              => ShaderVarVertices::TRAIL_AXIS_Z,
            EBuildinVertexAtribute::InsWorldRow1            => ShaderVarVertices::INS_WORLD_ROW1,
            EBuildinVertexAtribute::InsWorldRow2            => ShaderVarVertices::INS_WORLD_ROW2,
            EBuildinVertexAtribute::InsWorldRow3            => ShaderVarVertices::INS_WORLD_ROW3,
            EBuildinVertexAtribute::InsWorldRow4            => ShaderVarVertices::INS_WORLD_ROW4,
            EBuildinVertexAtribute::MatIdxs                 => ShaderVarVertices::INS_MAT_IDX,
            EBuildinVertexAtribute::ModelMaterialSkin       => ShaderVarVertices::INS_MODEL_MAT_SKIN,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ECustomVertexType {
    IVec4,
    Vec4,
    Vec3,
    Vec2,
    Float,
    Uint,
    U16x2,
    U16x4,
    U8x4,
    Unorm16x2,
    Unorm16x4,
    Unorm8x4,
    Int,
}
impl ECustomVertexType {
    pub fn format(&self) -> wgpu::VertexFormat {
        match self {
            ECustomVertexType::IVec4 => wgpu::VertexFormat::Sint32x4,
            ECustomVertexType::Vec4 => wgpu::VertexFormat::Float32x4,
            ECustomVertexType::Vec3 => wgpu::VertexFormat::Float32x3,
            ECustomVertexType::Vec2 => wgpu::VertexFormat::Float32x2,
            ECustomVertexType::Float => wgpu::VertexFormat::Float32,
            ECustomVertexType::Uint => wgpu::VertexFormat::Uint32,
            ECustomVertexType::U16x2 => wgpu::VertexFormat::Uint16x2,
            ECustomVertexType::U16x4 => wgpu::VertexFormat::Uint16x4,
            ECustomVertexType::U8x4 => wgpu::VertexFormat::Uint8x4,
            ECustomVertexType::Unorm16x2 => wgpu::VertexFormat::Unorm16x2,
            ECustomVertexType::Unorm16x4 => wgpu::VertexFormat::Unorm16x4,
            ECustomVertexType::Unorm8x4 => wgpu::VertexFormat::Unorm8x4,
            ECustomVertexType::Int => wgpu::VertexFormat::Sint32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CustomVertexAttribute {
    key: Atom,
    code: Atom,
    foruniform: Option<Atom>,
    vtype: ECustomVertexType,
}
impl CustomVertexAttribute {
    pub fn new(
        key: Atom,
        code: Atom,
        vtype: ECustomVertexType,
        foruniform: Option<Atom>,
    ) -> Self {
        Self { key, code, vtype, foruniform }
    }
    pub fn vtype(&self) -> ECustomVertexType {
        self.vtype
    }
    pub fn var_code(&self) -> &str {
        self.key.as_str()
    }
    pub fn kind(&self) -> String {
        self.format().shader_code()
    }
    pub fn foruniform(&self) -> &Option<Atom> {
        &self.foruniform
    }
    pub fn format(&self) -> wgpu::VertexFormat {
        self.vtype.format()
    }
    pub fn vs_running_code(&self) -> &str {
        self.code.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EVertexAttribute {
    Buildin(EBuildinVertexAtribute, wgpu::VertexFormat),
    Custom(CustomVertexAttribute),
}
impl TAsWgpuVertexAtribute for EVertexAttribute {
    fn as_attribute(&self, offset: wgpu::BufferAddress, shader_location: u32) -> wgpu::VertexAttribute {
        match self {
            EVertexAttribute::Buildin(val, format) => wgpu::VertexAttribute { format: *format, offset, shader_location, },
            EVertexAttribute::Custom(val) => wgpu::VertexAttribute { format: val.format(), offset, shader_location, },
        }
    }
}
impl EVertexAttribute {
    pub fn var_code(&self) -> &str {
        match self {
            EVertexAttribute::Buildin(val, _) => val.var_code(),
            EVertexAttribute::Custom(val) => val.var_code(),
        }
    }
    pub fn kind(&self) -> String {
        match self {
            EVertexAttribute::Buildin(val, _) => val.kind(),
            EVertexAttribute::Custom(val) => val.kind(),
        }
    }
    pub fn format(&self) -> wgpu::VertexFormat {
        match self {
            EVertexAttribute::Buildin(val, format) => *format,
            EVertexAttribute::Custom(val) => val.format(),
        }
    }
    
    pub fn matrix() -> String {
        let mut result = String::from("");
        result += ShaderVarUniform::WORLD_MATRIX;
        result += " = mat4(";
        result += ShaderVarVertices::INS_WORLD_ROW1; result += ", ";
        result += ShaderVarVertices::INS_WORLD_ROW2; result += ", ";
        result += ShaderVarVertices::INS_WORLD_ROW3; result += ", ";
        result += ShaderVarVertices::INS_WORLD_ROW4; result += ")";
        result += ";\r\n";

        result
    }

    pub fn matidx() -> String {
        let mut result = String::from("");

        result += ShaderVarUniform::MATIDX;
        result += " = ";
        result += ShaderVarVertices::INS_MODEL_MAT_SKIN;
        result += ";\r\n";

        result
    }

    pub fn trail() -> String {
        String::from("
    A_UV = vec2(TRAIL_INFO.y, step(0., TRAIL_INFO.x));

    vec3 zaxis = normalize(TRAIL_AXIS_Z);
    vec3 xaxis = normalize(TRAIL_AXIS_X);
    A_POSITION += xaxis * TRAIL_INFO.x;

    A_NORMAL = normalize(cross(zaxis, xaxis));
        ")
    }
    pub fn trail_billboard() -> String {
        String::from("
    A_UV = vec2(TRAIL_INFO.y, step(0., TRAIL_INFO.x));

    vec3 zaxis = normalize(TRAIL_AXIS_Z);
    vec3 yaxis = normalize(PI_CAMERA_POSITION.xyz - A_POSITION.xyz);
    vec3 xaxis = normalize(cross(yaxis, zaxis)) * TRAIL_INFO.x;
    A_POSITION += xaxis;

    A_NORMAL = yaxis;
        ")
    }
}

pub(crate) trait TAsWgpuVertexAtribute {
    fn as_attribute(&self, offset: wgpu::BufferAddress, shader_location: u32) ->wgpu::VertexAttribute;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexAttribute {
    pub kind: EVertexDataKind,
    pub format: wgpu::VertexFormat,
}
impl PartialOrd for VertexAttribute {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.kind.partial_cmp(&other.kind)
    }
}
impl Ord for VertexAttribute {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl TAsWgpuVertexAtribute for VertexAttribute {
    fn as_attribute(&self, offset: wgpu::BufferAddress, shader_location: u32) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: self.format,
            offset,
            shader_location,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyAttributesLayouts(pub Vec<(Vec<wgpu::VertexAttribute>, wgpu::VertexStepMode, u32)>);
impl KeyAttributesLayouts {
    pub fn layouts(&self) -> Vec<wgpu::VertexBufferLayout> {
        let mut result = vec![];
        self.0.iter().for_each(|(attrs, stepmode, strip)| {
            result.push(wgpu::VertexBufferLayout { array_stride: *strip as u64, step_mode: *stepmode, attributes: &attrs })
        });

        return result;
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyShaderFromAttributes(pub Vec<EVertexAttribute>);
impl KeyShaderFromAttributes {
    pub fn new(value: &Vec<VertexBufferDesc>) -> Self {
        let mut attrs = vec![];
        value.iter().for_each(|desc| {
            desc.attributes().iter().for_each(|attr| {
                attrs.push(attr.clone());
            });
        });
        Self(attrs)
    }
}

