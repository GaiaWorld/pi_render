
#[derive(Debug, Clone, Copy)]
pub struct EVerticeExtendCode(pub u32);
impl EVerticeExtendCode {
    pub const NONE: u32 = 0;
    pub const INSTANCE_BASE: u32            = 0b0000_0000_0000_0000_0000_0000_0000_0001;
    pub const INSTANCE_COLOR: u32           = 0b0000_0000_0000_0000_0000_0000_0000_0010;
    pub const INSTANCE_TILL_OFF_1: u32      = 0b0000_0000_0000_0000_0000_0000_0000_0100;
    pub const INSTANCE_VELOCITY: u32        = 0b0000_0000_0000_0000_0000_0000_0000_1000;
    pub const INSTANCE_SKIN: u32            = 0b0000_0000_0000_0000_0000_0000_0001_0000;
    pub const TRIAL: u32                    = 0b0000_0000_0000_0000_0000_0000_0010_0000;
    pub const TRIAL_BILLBOARD: u32          = 0b0000_0000_0000_0000_0000_0000_0100_0000;
    pub fn vs_running_code(&self) -> String {
        let mut result = String::from("
    mat4 PI_ObjectToWorld = U_PI_ObjectToWorld;
    vec4 PI_ObjectVelocity = U_PI_ObjectVelocity;
    uint PI_SkinBoneOffset0 = U_PI_SkinBoneOffset0;
    uint PI_SkinBoneOffset1 = U_PI_SkinBoneOffset1;
        ");

        if self.0 == 0 {
            result += Self::none().as_str();
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
    
        result
    }
    fn none() -> String {
        String::from("
        ")
    }
    fn base() -> String {
        String::from("
    PI_ObjectToWorld = mat4(A_INS_World1, A_INS_World2, A_INS_World3, A_INS_World4);
        ")
    }
    fn color() -> String {
        String::from("
    A_COLOR4 = A_COLOR4 * A_INS_Color;
        ")
    }
    fn uv() -> String {
        String::from("
    A_UV = A_UV * A_INS_TillOff1.xy + A_INS_TillOff1.zw;
        ")
    }
    fn velocity() -> String {
        String::from("
    PI_ObjectVelocity = A_INS_Velocity;
        ")
    }
    fn skin() -> String {
        String::from("
    PI_SkinBoneOffset0 = A_INS_SkinBoneOffset0;
        ")
    }
    fn trail() -> String {
        String::from("
    vec2 A_UV = vec2(TRAIL_INFO.y, step(0., TRAIL_INFO.x));

    vec3 zaxis = normalize(TRAIL_AXIS_Z);
    vec3 xaxis = normalize(TRAIL_AXIS_X);
    A_POSITION += xaxis * TRAIL_INFO.x;

    A_NORMAL = normalize(cross(zaxis, xaxis));
        ")
    }
    fn trail_billboard() -> String {
        String::from("
    vec2 A_UV = vec2(TRAIL_INFO.y, step(0., TRAIL_INFO.x));

    vec3 zaxis = normalize(TRAIL_AXIS_Z);
    vec3 yaxis = normalize(PI_CAMERA_POSITION.xyz - A_POSITION.xyz);
    vec3 xaxis = normalize(cross(yaxis, zaxis)) * TRAIL_INFO.x;
    A_POSITION += xaxis;

    A_NORMAL = yaxis;
        ")
    }
}