
mod scene;
mod model;
mod effect_value;
mod texture_and_sampler;

pub use scene::*;
pub use model::*;
pub use effect_value::*;
pub use texture_and_sampler::*;

pub trait ShaderBind {
    fn define_code(&self) -> String;
}

// struct ShaderBindTemp(u32, wgpu::BufferAddress);
// impl WriteBuffer for ShaderBindTemp {
//     fn write_into(&self, index: u32, buffer: &mut [u8]) {
//         todo!()
//     }

//     fn byte_len(&self) -> u32 {
//         todo!()
//     }

//     fn offset(&self) -> u32 {
//         todo!()
//     }
// }

pub trait TShaderBind {
    fn layout_entry(&self, entries: &mut Vec<wgpu::BindGroupLayoutEntry>);
    fn bind(&self) -> u32;
}
