pub struct UniformCommon {
    pub offset: u32,
}

impl UniformCommon {
    const CODE: &str = "layout(set = 0, binding = 0) uniform CommonParam { mat4 PI_View; mat4 PI_Project; mat4 PI_ViewProject; vec4 PI_Time;  }";
}