use crate::block_code::BlockCode;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ERenderTag {
    MainCamera,
    ShadowCast,
}
impl ERenderTag {
    pub fn vs_code(&self) -> BlockCode {
        match self {
            ERenderTag::MainCamera => {
                Self::vs_main_camera()
            },
            ERenderTag::ShadowCast => {
                Self::vs_shadow_cast()
            },
        }
    }
    pub fn fs_code(&self) -> BlockCode {
        match self {
            ERenderTag::MainCamera => {
                Self::fs_main_camera()
            },
            ERenderTag::ShadowCast => {
                Self::fs_shadow_cast()
            },
        }
    }
    fn vs_main_camera() -> BlockCode  {
        BlockCode {
            define: String::from("
layout(set = 0, binding = 0) uniform Camera {
    mat4 PI_MATRIX_V;
    mat4 PI_MATRIX_P;
    mat4 PI_MATRIX_VP;
    vec4 PI_CAMERA_POSITION;
    vec4 PI_VIEW_DIRECTION;
};

layout(set = 0, binding = 1) uniform Time {
    vec4 PI_Time;
    vec4 PI_DeltaTime;
};

layout(set = 1, binding = 0) uniform Model {
    mat4 U_PI_ObjectToWorld;
    mat4 PI_WorldToObject;
};
"),
            running: String::from(""),
        }
    }
    fn fs_main_camera() -> BlockCode  {
        BlockCode {
            define: String::from("
layout(set = 0, binding = 1) uniform Time {
    vec4 PI_Time;
    vec4 PI_DeltaTime;
};
"),
            running: String::from(""),
        }
    }
    fn vs_shadow_cast() -> BlockCode  {
        BlockCode {
            define: String::from(""),
            running: String::from(""),
        }
    }
    fn fs_shadow_cast() -> BlockCode  {
        BlockCode {
            define: String::from(""),
            running: String::from(""),
        }
    }
}