use crate::renderer::{attributes::{VertexAttribute, EVertexDataKind}, buildin_var::{ShaderVarVertices, ShaderVarUniform}};


#[derive(Debug, Clone, Copy)]
pub struct InstanceState;
impl InstanceState {
    pub const INSTANCE_INDEX: u32           = 0b0000_0000_0000_0000_0000_0000_0000_0001;
    pub const INSTANCE_BASE: u32            = 0b0000_0000_0000_0000_0000_0000_0000_0010;
    pub const INSTANCE_COLOR: u32           = 0b0000_0000_0000_0000_0000_0000_0000_0100;
    pub const INSTANCE_TILL_OFF_1: u32      = 0b0000_0000_0000_0000_0000_0000_0000_1000;
    pub const INSTANCE_SKIN: u32            = 0b0000_0000_0000_0000_0000_0000_0001_0000;
    pub const INSTANCE_CUSTOM_VEC4_A: u32   = 0b0000_0000_0000_0000_0000_0000_0010_0000;
    pub const INSTANCE_CUSTOM_VEC4_B: u32   = 0b0000_0000_0000_0000_0000_0000_0100_0000;
    pub const INSTANCE_CUSTOM_UVEC4_A: u32  = 0b0000_0000_0000_0000_0000_0000_1000_0000;
    pub const INSTANCE_CUSTOM_IVEC4_B: u32  = 0b0000_0000_0000_0000_0000_0001_0000_0000;
    pub const INSTANCE_TILL_OFF_2: u32      = 0b0000_0000_0000_0000_0000_0010_0000_0000;
    pub const INSTANCE_VELOCITY: u32        = 0b0000_0000_0000_0000_0000_0100_0000_0000;
    pub fn attributes(state: u32) -> Vec<VertexAttribute> {
        let mut result = vec![];
        if (state & Self::INSTANCE_INDEX) == Self::INSTANCE_INDEX {
            result.push(VertexAttribute { kind: EVertexDataKind::InsIndex, format: wgpu::VertexFormat::Uint32 });
        }
        if (state & Self::INSTANCE_BASE) == Self::INSTANCE_BASE {
            result.push(VertexAttribute { kind: EVertexDataKind::InsWorldRow1, format: wgpu::VertexFormat::Float32x4 });
            result.push(VertexAttribute { kind: EVertexDataKind::InsWorldRow2, format: wgpu::VertexFormat::Float32x4 });
            result.push(VertexAttribute { kind: EVertexDataKind::InsWorldRow3, format: wgpu::VertexFormat::Float32x4 });
            result.push(VertexAttribute { kind: EVertexDataKind::InsWorldRow4, format: wgpu::VertexFormat::Float32x4 });
        }
        if (state & Self::INSTANCE_COLOR) == Self::INSTANCE_COLOR {
            result.push(VertexAttribute { kind: EVertexDataKind::InsColor, format: wgpu::VertexFormat::Float32x4 });
        }
        if (state & Self::INSTANCE_TILL_OFF_1) == Self::INSTANCE_TILL_OFF_1 {
            result.push(VertexAttribute { kind: EVertexDataKind::InsTillOffset1, format: wgpu::VertexFormat::Float32x4 });
        }
        if (state & Self::INSTANCE_CUSTOM_VEC4_A) == Self::INSTANCE_CUSTOM_VEC4_A {
            result.push(VertexAttribute { kind: EVertexDataKind::InsCustomVec4A, format: wgpu::VertexFormat::Float32x4 });
        }
        if (state & Self::INSTANCE_CUSTOM_VEC4_B) == Self::INSTANCE_CUSTOM_VEC4_B {
            result.push(VertexAttribute { kind: EVertexDataKind::InsCustomVec4B, format: wgpu::VertexFormat::Float32x4 });
        }
        if (state & Self::INSTANCE_CUSTOM_UVEC4_A) == Self::INSTANCE_CUSTOM_UVEC4_A {
            result.push(VertexAttribute { kind: EVertexDataKind::InsCustomUVec4A, format: wgpu::VertexFormat::Uint32x4 });
        }
        if (state & Self::INSTANCE_CUSTOM_IVEC4_B) == Self::INSTANCE_CUSTOM_IVEC4_B {
            result.push(VertexAttribute { kind: EVertexDataKind::InsCustomIVec4B, format: wgpu::VertexFormat::Sint32x4 });
        }
        if (state & Self::INSTANCE_TILL_OFF_2) == Self::INSTANCE_TILL_OFF_2 {
            result.push(VertexAttribute { kind: EVertexDataKind::InsTillOffset2, format: wgpu::VertexFormat::Float32x4 });
        }
        if (state & Self::INSTANCE_VELOCITY) == Self::INSTANCE_VELOCITY {
            result.push(VertexAttribute { kind: EVertexDataKind::InsVelocity, format: wgpu::VertexFormat::Float32x4 });
        }

        result
    }
    pub fn bytes_per_instance(state: u32) -> u32 {
        let mut result = 0;
        if (state & Self::INSTANCE_INDEX) == Self::INSTANCE_INDEX {
            result += 4;
        }
        if (state & Self::INSTANCE_BASE) == Self::INSTANCE_BASE {
            result += 4 * 4;
            result += 4 * 4;
            result += 4 * 4;
            result += 4 * 4;
        }
        if (state & Self::INSTANCE_COLOR) == Self::INSTANCE_COLOR {
            result += 4 * 4;
        }
        if (state & Self::INSTANCE_TILL_OFF_1) == Self::INSTANCE_TILL_OFF_1 {
            result += 4 * 4;
        }
        if (state & Self::INSTANCE_CUSTOM_VEC4_A) == Self::INSTANCE_CUSTOM_VEC4_A {
            result += 4 * 4;
        }
        if (state & Self::INSTANCE_CUSTOM_VEC4_B) == Self::INSTANCE_CUSTOM_VEC4_B {
            result += 4 * 4;
        }
        if (state & Self::INSTANCE_CUSTOM_UVEC4_A) == Self::INSTANCE_CUSTOM_UVEC4_A {
            result += 4 * 4;
        }
        if (state & Self::INSTANCE_CUSTOM_IVEC4_B) == Self::INSTANCE_CUSTOM_IVEC4_B {
            result += 4 * 4;
        }
        if (state & Self::INSTANCE_TILL_OFF_2) == Self::INSTANCE_TILL_OFF_2 {
            result += 4 * 4;
        }
        if (state & Self::INSTANCE_VELOCITY) == Self::INSTANCE_VELOCITY {
            result += 4 * 4;
        }

        result
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EVerticeExtendCode(pub u32);
impl EVerticeExtendCode {
    pub const NONE: u32 = 0;
    pub const INSTANCE_INDEX: u32           = InstanceState::INSTANCE_INDEX;
    pub const INSTANCE_BASE: u32            = InstanceState::INSTANCE_BASE;
    pub const INSTANCE_COLOR: u32           = InstanceState::INSTANCE_COLOR;
    pub const INSTANCE_TILL_OFF_1: u32      = InstanceState::INSTANCE_TILL_OFF_1;
    pub const INSTANCE_VELOCITY: u32        = InstanceState::INSTANCE_VELOCITY;
    pub const INSTANCE_SKIN: u32            = InstanceState::INSTANCE_SKIN;
    pub const INSTANCE_CUSTOM_VEC4_A: u32   = InstanceState::INSTANCE_CUSTOM_VEC4_A;
    pub const INSTANCE_CUSTOM_VEC4_B: u32   = InstanceState::INSTANCE_CUSTOM_VEC4_B;
    pub const INSTANCE_CUSTOM_UVEC4_A: u32  = InstanceState::INSTANCE_CUSTOM_UVEC4_A;
    pub const INSTANCE_CUSTOM_IVEC4_B: u32  = InstanceState::INSTANCE_CUSTOM_IVEC4_B;
    pub const INSTANCE_TILL_OFF_2: u32      = InstanceState::INSTANCE_TILL_OFF_2;

    pub const TRIAL: u32                    = 0b0001_0000_0000_0000_0000_0000_0000_0000;
    pub const TRIAL_BILLBOARD: u32          = 0b0010_0000_0000_0000_0000_0000_0000_0000;
    pub const TRAIL_LINE_X: u32             = 0b0100_0000_0000_0000_0000_0000_0000_0000;
    pub const TRAIL_LINE_Z: u32             = 0b1000_0000_0000_0000_0000_0000_0000_0000;
    pub fn vs_running_code(&self) -> String {
        let mut result = String::from("
    mat4 PI_ObjectToWorld = U_PI_ObjectToWorld;
    vec4 PI_ObjectVelocity = U_PI_ObjectVelocity;
    uint PI_SkinBoneOffset0 = U_PI_SkinBoneOffset0;
    uint PI_SkinBoneOffset1 = U_PI_SkinBoneOffset1;
        ");

        if (self.0 & Self::INSTANCE_INDEX) == Self::INSTANCE_INDEX {
            result += Self::index().as_str();
        } else {
            result += Self::index_null().as_str();
        }
        if (self.0 & Self::INSTANCE_BASE) == Self::INSTANCE_BASE {
            result += Self::base().as_str();
        }
        if (self.0 & Self::INSTANCE_COLOR) == Self::INSTANCE_COLOR {
            result += Self::color().as_str();
        }
        if (self.0 & Self::INSTANCE_TILL_OFF_1) == Self::INSTANCE_TILL_OFF_1 {
            result += Self::uv().as_str();
        }
        if (self.0 & Self::INSTANCE_VELOCITY) == Self::INSTANCE_VELOCITY {
            result += Self::velocity().as_str();
        }
        if (self.0 & Self::INSTANCE_SKIN) == Self::INSTANCE_SKIN {
            result += Self::skin().as_str();
        }

        if (self.0 & Self::TRIAL) == Self::TRIAL {
            result += Self::trail().as_str();
        }

        if (self.0 & Self::TRIAL_BILLBOARD) == Self::TRIAL_BILLBOARD {
            result += Self::trail_billboard().as_str();
        }
        if (self.0 & Self::TRAIL_LINE_X) == Self::TRAIL_LINE_X {
            result += Self::trail_line_x().as_str();
        }
        if (self.0 & Self::TRAIL_LINE_Z) == Self::TRAIL_LINE_Z {
            result += Self::trail_line_z().as_str();
        }
    
        result
    }
    fn index_null() -> String {
        let mut result = String::from("uint ");
        result += ShaderVarVertices::OBJECT_INDEX;
        result += " = 0";
        result += ";\r\n";

        result
    }
    fn index() -> String {
        let mut result = String::from("uint ");
        result += ShaderVarVertices::OBJECT_INDEX;
        result += " = ";
        result += ShaderVarVertices::INSTANCE_INDEX;
        result += ";\r\n";

        result
    }
    fn base() -> String {
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
    fn color() -> String {
        let mut result = String::from("");
        result += ShaderVarVertices::COLOR4;
        result += " = ";
        result += ShaderVarVertices::COLOR4; result += " * ";
        result += ShaderVarVertices::INS_COLOR;
        result += ";\r\n";
        result
    }
    fn uv() -> String {
        let mut result = String::from("");
        result += ShaderVarVertices::UV;
        result += " = ";
        result += ShaderVarVertices::UV; result += " * ";
        result += ShaderVarVertices::INS_TILL_OFFSET1; result += ".xy + ";
        result += ShaderVarVertices::INS_TILL_OFFSET1; result += ".zw";
        result += ";\r\n";
        result
    }
    fn velocity() -> String {
        let mut result = String::from("");
        result += ShaderVarUniform::VELOCITY;
        result += " = ";
        result += ShaderVarVertices::CUSTOM_VEC2_A;
        result += ";\r\n";
        result
    }
    fn skin() -> String {
        let mut result = String::from("");
        result += ShaderVarUniform::_SKIN_BONE_OFFSET0;
        result += " = ";
        result += ShaderVarVertices::INS_SKIN_BONE_OFFSET0;
        result += ";\r\n";
        result
    }
    fn trail() -> String {
        String::from("
    A_UV = vec2(TRAIL_INFO.y, step(0., TRAIL_INFO.x));

    vec3 zaxis = normalize(TRAIL_AXIS_Z);
    vec3 xaxis = normalize(TRAIL_AXIS_X);
    A_POSITION += xaxis * TRAIL_INFO.x;

    A_NORMAL = normalize(cross(zaxis, xaxis));
        ")
    }
    fn trail_billboard() -> String {
        String::from("
    A_UV = vec2(TRAIL_INFO.y, step(0., TRAIL_INFO.x));

    vec3 zaxis = normalize(TRAIL_AXIS_Z);
    vec3 yaxis = normalize(PI_CAMERA_POSITION.xyz - A_POSITION.xyz);
    vec3 xaxis = normalize(cross(yaxis, zaxis)) * TRAIL_INFO.x;
    A_POSITION += xaxis;

    A_NORMAL = yaxis;
        ")
    }
    fn trail_line_x() -> String {
        String::from("
    A_UV = vec2(TRAIL_INFO.y, step(0., TRAIL_INFO.x));

    vec3 zaxis = normalize(TRAIL_AXIS_Z);
    vec3 xaxis = normalize(TRAIL_AXIS_X);
    A_POSITION += xaxis * abs(TRAIL_INFO.x) * step(0., TRAIL_INFO.x);

    A_NORMAL = normalize(cross(zaxis, xaxis));
        ")
    }
    fn trail_line_z() -> String {
        String::from("
    A_UV = vec2(TRAIL_INFO.y, step(0., TRAIL_INFO.x));

    vec3 zaxis = normalize(TRAIL_AXIS_Z);
    vec3 xaxis = normalize(TRAIL_AXIS_X);
    A_POSITION += zaxis * step(0., TRAIL_INFO.x) * max(0.1, length(TRAIL_AXIS_Z));

    A_NORMAL = normalize(cross(zaxis, xaxis));
        ")
    }
}