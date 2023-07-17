
#[derive(Debug, Clone, Copy)]
pub struct EInstanceCode(pub u32);
impl EInstanceCode {
    pub const NONE: u32 = 0;
    pub const BASE: u32         = 0b0000_0000_0000_0000_0000_0000_0000_0001;
    pub const COLOR: u32        = 0b0000_0000_0000_0000_0000_0000_0000_0010;
    pub const TILL_OFF_1: u32   = 0b0000_0000_0000_0000_0000_0000_0000_0100;
    pub const VELOCITY: u32     = 0b0000_0000_0000_0000_0000_0000_0000_1000;
    pub const SKIN: u32         = 0b0000_0000_0000_0000_0000_0000_0001_0000;
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
        else {
            result += Self::base().as_str();
        }
        if (self.0 & Self::COLOR) == Self::COLOR {
            result += Self::color().as_str();
        }
        if (self.0 & Self::TILL_OFF_1) == Self::TILL_OFF_1 {
            result += Self::uv().as_str();
        }
        if (self.0 & Self::VELOCITY) == Self::VELOCITY {
            result += Self::velocity().as_str();
        }
        if (self.0 & Self::SKIN) == Self::SKIN {
            result += Self::skin().as_str();
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
}