
pub struct TextureScaleOffset {
    pub use_x: u32,
    pub use_y: u32,
    pub use_w: u32,
    pub use_h: u32,
    pub width: u32,
    pub height: u32,
    pub u_scale: f32,
    pub v_scale: f32,
    pub u_offset: f32,
    pub v_offset: f32,
}

impl TextureScaleOffset {
    pub fn from_rect(
        use_x: u32,
        use_y: u32,
        use_w: u32,
        use_h: u32,
        width: u32,
        height: u32,
    ) -> Self {
        let u_scale = width  as f32 / use_w as f32;
        let v_scale = height as f32 / use_h as f32;
        let u_offset = use_x as f32 / width  as f32;
        let v_offset = use_y as f32 / height as f32;
        
        Self { u_scale, v_scale, u_offset, v_offset, use_x, use_y, use_w, use_h, width, height }
    }
}

pub fn calc_uniform_size(
    device: &wgpu::Device,
    used_size: u64,
) -> u64 {
    let limit = device.limits().min_uniform_buffer_offset_alignment as u64;
    let t = used_size / limit;
    if used_size - t * limit > 0 {
        limit * (t + 1)
    } else {
        limit * t
    }
}