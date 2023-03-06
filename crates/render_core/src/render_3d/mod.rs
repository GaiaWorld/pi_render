pub mod binds;
pub mod bind_groups;
pub mod shader;


pub fn vec_u8_to_f32_16(val: &Vec<u8>) -> [f32;16] {
    if val.len() >= 64 {
        let mut temp: [u8;64] = [0;64];
        for i in 0..64 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;64], [f32;16]>(temp)
        }
    } else {
        [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.]
    }
}

pub fn vec_u8_to_f32_4(val: &Vec<u8>) -> [f32;4] {
    if val.len() >= 16 {
        let mut temp: [u8;16] = [0;16];
        for i in 0..16 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;16], [f32;4]>(temp)
        }
    } else {
        [1., 0., 0., 0.]
    }
}

pub fn vec_u8_to_f32_2(val: &Vec<u8>) -> [f32;2] {
    if val.len() >= 8 {
        let mut temp: [u8;8] = [0;8];
        for i in 0..8 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;8], [f32;2]>(temp)
        }
    } else {
        [0., 0.]
    }
}

pub fn vec_u8_to_f32(val: &Vec<u8>) -> f32 {
    if val.len() >= 4 {
        let mut temp: [u8;4] = [0;4];
        for i in 0..4 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;4], f32>(temp)
        }
    } else {
        0.
    }
}

pub fn vec_u8_to_i32(val: &Vec<u8>) -> i32 {
    if val.len() >= 4 {
        let mut temp: [u8;4] = [0;4];
        for i in 0..4 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;4], i32>(temp)
        }
    } else {
        0
    }
}

pub fn vec_u8_to_u32(val: &Vec<u8>) -> u32 {
    if val.len() >= 4 {
        let mut temp: [u8;4] = [0;4];
        for i in 0..4 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;4], u32>(temp)
        }
    } else {
        0
    }
}
