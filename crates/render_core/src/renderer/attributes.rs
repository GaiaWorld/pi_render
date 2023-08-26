use super::buildin_var::ShaderVarVertices;


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
    CustomVec4A            ,
    CustomVec4B            ,
    CustomVec3A            ,
    CustomVec3B            ,
    CustomVec2A            ,
    CustomVec2B            ,
    InsIndex               ,
    InsWorldRow1           ,
    InsWorldRow2           ,
    InsWorldRow3           ,
    InsWorldRow4           ,
    InsColor               ,
    InsTillOffset1         ,
    InsTillOffset2         ,
    InsCustomVec4A         ,
    InsCustomVec4B         ,
    InsCustomUVec4A        ,
    InsCustomIVec4B        ,

    MatricesIndices1       ,
    MatricesWeights1       ,

    MatricesIndices2       ,
    MatricesWeights2       ,
    MatricesIndicesExtra2  ,
    MatricesWeightsExtra2  ,

    MatricesIndices3       ,
    MatricesWeights3       ,
    MatricesIndicesExtra3  ,
    MatricesWeightsExtra3  ,

    InsVelocity            ,
    ParticleAgeLife        ,
    ParticlePosition        ,
    ParticleScaling         ,
    ParticleRotation        ,
    ParticleDirection       ,
    ParticleColor           ,
    ParticleTilloff         ,
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
            EVertexDataKind::CustomVec4A            => ShaderVarVertices::CUSTOM_VEC4_A             ,
            EVertexDataKind::CustomVec4B            => ShaderVarVertices::CUSTOM_VEC4_B             ,
            EVertexDataKind::CustomVec3A            => ShaderVarVertices::CUSTOM_VEC3_A             ,
            EVertexDataKind::CustomVec3B            => ShaderVarVertices::CUSTOM_VEC3_B             ,
            EVertexDataKind::CustomVec2A            => ShaderVarVertices::CUSTOM_VEC2_A             ,
            EVertexDataKind::CustomVec2B            => ShaderVarVertices::CUSTOM_VEC2_B             ,
            EVertexDataKind::InsIndex               => ShaderVarVertices::INSTANCE_INDEX            ,
            EVertexDataKind::InsWorldRow1           => ShaderVarVertices::INS_WORLD_ROW1            ,
            EVertexDataKind::InsWorldRow2           => ShaderVarVertices::INS_WORLD_ROW2            ,
            EVertexDataKind::InsWorldRow3           => ShaderVarVertices::INS_WORLD_ROW3            ,
            EVertexDataKind::InsWorldRow4           => ShaderVarVertices::INS_WORLD_ROW4            ,
            EVertexDataKind::InsColor               => ShaderVarVertices::INS_COLOR                 ,
            EVertexDataKind::InsTillOffset1         => ShaderVarVertices::INS_TILL_OFFSET1          ,
            EVertexDataKind::InsTillOffset2         => ShaderVarVertices::INS_TILL_OFFSET2          ,
            EVertexDataKind::InsCustomVec4A         => ShaderVarVertices::INS_CUSTOM_VEC4_A         ,
            EVertexDataKind::InsCustomVec4B         => ShaderVarVertices::INS_CUSTOM_VEC4_B         ,
            EVertexDataKind::InsCustomUVec4A        => ShaderVarVertices::INS_CUSTOM_UVEC4_A        ,
            EVertexDataKind::InsCustomIVec4B        => ShaderVarVertices::INS_CUSTOM_IVEC4_B        ,
            EVertexDataKind::MatricesIndices1       => ShaderVarVertices::MATRICES_INDICES1         ,
            EVertexDataKind::MatricesWeights1       => ShaderVarVertices::MATRICES_WEIGHTS1         ,
            EVertexDataKind::MatricesIndices2       => ShaderVarVertices::MATRICES_INDICES2         ,
            EVertexDataKind::MatricesWeights2       => ShaderVarVertices::MATRICES_WEIGHTS2         ,
            EVertexDataKind::MatricesIndicesExtra2  => ShaderVarVertices::MATRICES_INDICES_EXTRA2   ,
            EVertexDataKind::MatricesWeightsExtra2  => ShaderVarVertices::MATRICES_WEIGHTS_EXTRA2   ,
            EVertexDataKind::MatricesIndices3       => ShaderVarVertices::MATRICES_INDICES3         ,
            EVertexDataKind::MatricesWeights3       => ShaderVarVertices::MATRICES_WEIGHTS3         ,
            EVertexDataKind::MatricesIndicesExtra3  => ShaderVarVertices::MATRICES_INDICES_EXTRA3   ,
            EVertexDataKind::MatricesWeightsExtra3  => ShaderVarVertices::MATRICES_WEIGHTS_EXTRA3   ,
            EVertexDataKind::InsVelocity            => ShaderVarVertices::INS_VELOCITY              ,
            EVertexDataKind::ParticleAgeLife        => ShaderVarVertices::PARTICLE_AGE_LIFE         ,
            EVertexDataKind::ParticlePosition       => ShaderVarVertices::PARTICLE_POSITION         ,
            EVertexDataKind::ParticleScaling        => ShaderVarVertices::PARTICLE_SCALING          ,
            EVertexDataKind::ParticleRotation       => ShaderVarVertices::PARTICLE_ROTATION         ,
            EVertexDataKind::ParticleDirection      => ShaderVarVertices::PARTICLE_DIRECTION        ,
            EVertexDataKind::ParticleColor          => ShaderVarVertices::PARTICLE_COLOR            ,
            EVertexDataKind::ParticleTilloff        => ShaderVarVertices::PARTICLE_TILLOFF          ,
            EVertexDataKind::TrailAxisX             => ShaderVarVertices::TRAIL_AXIS_X          ,
            EVertexDataKind::TrailAxisZ             => ShaderVarVertices::TRAIL_AXIS_Z          ,
            EVertexDataKind::TrailInfo              => ShaderVarVertices::TRAIL_INFO          ,
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
            EVertexDataKind::CustomVec4A            => "vec4"       ,
            EVertexDataKind::CustomVec4B            => "vec4"       ,
            EVertexDataKind::CustomVec3A            => "vec3"       ,
            EVertexDataKind::CustomVec3B            => "vec3"       ,
            EVertexDataKind::CustomVec2A            => "vec2"       ,
            EVertexDataKind::CustomVec2B            => "vec2"       ,
            EVertexDataKind::InsIndex               => "uint"       ,
            EVertexDataKind::InsWorldRow1           => "vec4"       ,
            EVertexDataKind::InsWorldRow2           => "vec4"       ,
            EVertexDataKind::InsWorldRow3           => "vec4"       ,
            EVertexDataKind::InsWorldRow4           => "vec4"       ,
            EVertexDataKind::InsColor               => "vec4"       ,
            EVertexDataKind::InsTillOffset1         => "vec4"       ,
            EVertexDataKind::InsTillOffset2         => "vec4"       ,
            EVertexDataKind::InsCustomVec4A         => "vec4"       ,
            EVertexDataKind::InsCustomVec4B         => "vec4"       ,
            EVertexDataKind::InsCustomUVec4A        => "uvec4"      ,
            EVertexDataKind::InsCustomIVec4B        => "ivec4"      ,
            EVertexDataKind::MatricesIndices1       => "uvec2"      ,
            EVertexDataKind::MatricesWeights1       => "float"      ,
            EVertexDataKind::MatricesIndices2       => "uvec2"      ,
            EVertexDataKind::MatricesWeights2       => "vec2"       ,
            EVertexDataKind::MatricesIndicesExtra2  => "uvec2"      ,
            EVertexDataKind::MatricesWeightsExtra2  => "vec2"       ,
            EVertexDataKind::MatricesIndices3       => "uvec3"      ,
            EVertexDataKind::MatricesWeights3       => "vec3"       ,
            EVertexDataKind::MatricesIndicesExtra3  => "uvec3"      ,
            EVertexDataKind::MatricesWeightsExtra3  => "vec3"       ,
            EVertexDataKind::InsVelocity            => "vec4"       ,
            EVertexDataKind::ParticleAgeLife        => "vec2"       ,
            EVertexDataKind::ParticlePosition       => "vec3"       ,
            EVertexDataKind::ParticleScaling        => "vec3"       ,
            EVertexDataKind::ParticleRotation       => "vec4"       ,
            EVertexDataKind::ParticleDirection      => "vec3"       ,
            EVertexDataKind::ParticleColor          => "vec4"       ,
            EVertexDataKind::ParticleTilloff        => "vec4"       ,
            EVertexDataKind::TrailAxisX             => "vec3"       ,
            EVertexDataKind::TrailAxisZ             => "vec3"       ,
            EVertexDataKind::TrailInfo              => "vec2"       ,
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ShaderAttribute {
    pub kind: EVertexDataKind,
    pub location: u32,
}
impl ShaderAttribute {
    pub fn define_code(&self) -> String {
        let mut result = String::from("layout(location = ");
        result += self.location.to_string().as_str();
        result += ") in ";
        result += self.kind.kind();
        result += " ";

        result += "V";
        result += self.kind.var_code();
        result += ";\r\n";

        result
    }
    pub fn running_code(&self) -> String {
        
        let mut result = String::from("");
        match self.kind {
            EVertexDataKind::Normal => {
            },
            EVertexDataKind::Color4 => {
            },
            EVertexDataKind::UV => {
            },
            _ => {
                result += self.kind.kind();
                result += " ";
            }
        }
        result += self.kind.var_code();
        result += " = V";
        result += self.kind.var_code();
        result += ";\r\n";

        result
    }
}


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyShaderFromAttributes(pub Vec<ShaderAttribute>);
