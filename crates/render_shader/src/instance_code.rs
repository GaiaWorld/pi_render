
#[derive(Debug, Clone, Copy)]
pub struct EInstanceCode(pub u32);
impl EInstanceCode {
    pub const NONE: u32 = 0;
    pub const BASE: u32 = 1;
    pub const COLOR: u32 = 2;
    pub const TILL_SCALE_1: u32 = 4;
    pub fn vs_running_code(&self) -> String {
        let mut result = String::from("");

        if self.0 == 0 {
            result += Self::none().as_str();
        }
        else {
            result += Self::base().as_str();
        }
        if self.0 & Self::COLOR == Self::COLOR {
            result += Self::color().as_str();
        }
        if self.0 & Self::TILL_SCALE_1 == Self::TILL_SCALE_1 {
            result += Self::uv().as_str();
        }

        result
    }
    fn none() -> String {
        String::from("
        mat4 PI_ObjectToWorld = U_PI_ObjectToWorld;
        ")
    }
    fn base() -> String {
        String::from("
        mat4 PI_ObjectToWorld = mat4(A_INS_World1, A_INS_World2, A_INS_World3, A_INS_World4); 
        ")
    }
    fn color() -> String {
        String::from("
        A_COLOR4 = A_COLOR4 * A_INS_Color;
        ")
    }
    fn uv() -> String {
        String::from("
        A_UV = A_UV * A_INS_TileOff1.xy + A_INS_TileOff1.zw;
        ")
    }
}