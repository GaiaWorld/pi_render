use super::ShaderSetBind;


fn sampler_ty_code(ty: wgpu::SamplerBindingType) -> String {
    match ty {
        wgpu::SamplerBindingType::Filtering => String::from(" sampler "),
        wgpu::SamplerBindingType::NonFiltering => String::from(" sampler "),
        wgpu::SamplerBindingType::Comparison => String::from(" sampler_comparison "),
    }
}
pub fn sampler_code(slotname: &str, ty: wgpu::SamplerBindingType, set: u32, bind: u32) -> String {

    // layout(set = 2, binding = 0) uniform texture2D _MainTex;
    let mut result = ShaderSetBind::code_set_bind_head(set, bind);
    result += sampler_ty_code(ty).as_str();
    result += "sampler";
    result += slotname;
    result += ";\r\n";

    result
}