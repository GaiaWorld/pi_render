pub mod draw_obj;
pub mod draw_sort;
pub mod draw_obj_list;
pub mod vertices;
pub mod indices;
pub mod buffer;
pub mod bind_buffer;
pub mod bind;
pub mod bind_group;
pub mod attributes;
pub mod buildin_data;
pub mod buildin_var;
pub mod shader;
pub mod instance;
pub mod vertex_buffer;
pub mod vertex_buffer_desc;
pub mod vertex_format;
pub mod pipeline;
pub mod texture;
pub mod sampler;
pub mod shader_stage;
pub mod error;

pub const ASSET_SIZE_FOR_UNKOWN: usize = 256;

pub fn bytes_write_to_memory(
    bytes: &[u8],
    offset: usize,
    memory: &mut [u8],
) {
    let mut index = 0;
    for v in bytes.iter() {
        memory[offset + index] = *v;
        index += 1;
    }
}