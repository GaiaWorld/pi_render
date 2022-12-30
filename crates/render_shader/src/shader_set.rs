use crate::{set_bind::ShaderSetBind, skin_code::ESkinCode, shader_bind::{ShaderBindSceneAboutCamera, ShaderBindSceneAboutTime, ShaderBindSceneAboutFog, ShaderBindModelAboutMatrix, ShaderBindModelAboutSkinFramesTex, ShaderBindModelAboutSkinRowTex}};


pub trait ShaderSet {
    fn define_code(&self) -> String;
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShaderSetSceneAbout {
    set: u32,
    time: bool,
    fog: bool,
    brdf: bool,
    env: bool,
    bind_camera: u32,
    bind_time: u32,
    bind_fog: u32,
    bind_brdf_texture: u32,
    bind_brdf_sampler: u32,
    bind_env_param: u32,
    bind_env_texture: u32,
    bind_env_sampler: u32,
}
impl ShaderSetSceneAbout {
    pub fn new(
        set: u32,
        scene_time: bool,
        scene_fog: bool,
        scene_brdf: bool,
        scene_env: bool,
    ) -> Self {
        let mut bind = 0;

        let bind_camera = bind; bind += 1;
        let bind_time = if scene_time { 
            let result = bind; bind += 1;
            result
        } else {
            u32::MAX
        };

        let bind_fog = if scene_fog {
            let result = bind; bind += 1;
            result
        } else {
            u32::MAX
        };

        let (bind_brdf_texture, bind_brdf_sampler) = if scene_brdf {
            let result = bind; bind += 2;
            (result, result + 1)
        } else {
            (u32::MAX, u32::MAX)
        };

        let (bind_env_param, bind_env_texture, bind_env_sampler) = if scene_brdf {
            let result = bind; bind += 3; (result, result + 1, result + 2)
        } else {
            (u32::MAX, u32::MAX, u32::MAX)
        };

        Self {
            set,
            time: scene_time,
            fog: scene_fog,
            brdf: scene_brdf,
            env: scene_env,
            bind_camera,
            bind_time,
            bind_fog,
            bind_brdf_texture,
            bind_brdf_sampler,
            bind_env_param,
            bind_env_texture,
            bind_env_sampler,
        }
    }
    pub fn bind_camera(&self) -> u32 {
        self.bind_camera 
    }
    pub fn bind_time(&self) -> u32 {
        self.bind_time 
    }
    pub fn bind_fog(&self) -> u32 {
        self.bind_fog 
    }
    pub fn bind_brdf_texture(&self) -> u32 {
        self.bind_brdf_texture 
    }
    pub fn bind_brdf_sampler(&self) -> u32 {
        self.bind_brdf_sampler 
    }
    pub fn bind_env_param(&self) -> u32 {
        self.bind_env_param 
    }
    pub fn bind_env_texture(&self) -> u32 {
        self.bind_env_texture 
    }
    pub fn bind_env_sampler(&self) -> u32 {
        self.bind_env_sampler 
    }
    pub fn define_code(&self) -> String {
        let mut result = String::from("");

        result += ShaderSetBind::code_set_bind_head(self.set, self.bind_camera).as_str();
        result += " Camera {\r\n";
        result += ShaderSetBind::code_uniform("mat4", ShaderBindSceneAboutCamera::VAR_VIEW_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderBindSceneAboutCamera::VAR_PROJECT_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderBindSceneAboutCamera::VAR_VIEW_PROJECT_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderBindSceneAboutCamera::VAR_CAMERA_POSITION).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderBindSceneAboutCamera::VAR_CAMERA_DIRECTION).as_str();
        result += "};\r\n";

        if self.time {
            result += ShaderSetBind::code_set_bind_head(self.set, self.bind_time).as_str();
            result += " Time {\r\n";
            result += ShaderSetBind::code_uniform("vec4", ShaderBindSceneAboutTime::VAR_TIME).as_str();
            result += ShaderSetBind::code_uniform("vec4", ShaderBindSceneAboutTime::VAR_DELTA_TIME).as_str();
            result += "};\r\n";
        }
        
        if self.fog {
            result += ShaderSetBind::code_set_bind_head(self.set, self.bind_fog).as_str();
            result += " Fog {\r\n";
            result += ShaderSetBind::code_uniform("vec4", ShaderBindSceneAboutFog::VAR_FOG_INFO).as_str();
            result += ShaderSetBind::code_uniform("vec4", ShaderBindSceneAboutFog::VAR_FOG_PARAM).as_str();
            result += "};\r\n";
        }

        result
    }

    pub fn vs_running_code(&self) -> String {
         String::from("")
    }

    pub fn fs_running_code(&self) -> String {
         String::from("")
    }
}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShaderSetModelAbout {
    set: u32,
    skin: Option<ESkinCode>,
    bind_matrix: u32,
    bind_bone_size: u32,
    bind_bone_texture: u32,
    bind_bone_sampler: u32,
}
impl ShaderSetModelAbout {
    pub fn new(set: u32, skin: Option<ESkinCode>) -> Self {
        let mut bind = 0;

        let bind_matrix = bind;
        bind += 1;

        let (bind_bone_size, bind_bone_texture, bind_bone_sampler) = if let Some(skin) = &skin {
            match skin {
                ESkinCode::None => {
                    (u32::MAX, u32::MAX, u32::MAX)
                },
                ESkinCode::RowTexture(_) => {
                    let result = bind; bind += 3;
                    (result, result + 1, result + 2)
                },
                ESkinCode::FramesTextureInstance(_) => {
                    let result = bind; bind += 3;
                    (result, result + 1, result + 2)
                },
            }
        } else {
            (u32::MAX, u32::MAX, u32::MAX)
        };
        
        Self { 
            set,
            skin,
            bind_matrix,
            bind_bone_size,
            bind_bone_texture,
            bind_bone_sampler
        }
    }
    pub fn set(&self) -> u32 {
        self.set
    }
    pub fn bind_matrix(&self) -> u32 {
        self.bind_matrix
    }
    pub fn bind_bone_size(&self) -> u32 {
        self.bind_bone_size
    }
    pub fn bind_bone_texture(&self) -> u32 {
        self.bind_bone_texture
    }
    pub fn bind_bone_sampler(&self) -> u32 {
        self.bind_bone_sampler
    }
    pub fn define_code(&self) -> String {
        let mut result = String::from("");

        result += ShaderSetBind::code_set_bind_head(self.set, self.bind_matrix).as_str();
        result += " Model {\r\n";
        result += ShaderSetBind::code_uniform("mat4", ShaderBindModelAboutMatrix::_VAR_WORLD_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderBindModelAboutMatrix::_VAR_WORLD_MATRIX_INV).as_str();
        result += "};\r\n";

        if let Some(skin) = &self.skin {
            match skin {
                ESkinCode::None => {
                    
                },
                ESkinCode::RowTexture(_) => {
                    result += ShaderSetBind::code_set_bind_head(self.set, self.bind_bone_size).as_str();
                    result += " Bone {\r\n";
                    result += ShaderSetBind::code_uniform("vec4", ShaderBindModelAboutSkinRowTex::VAR_BONE_TEX_SIZE).as_str();
                    result += "};\r\n";
                    
                    result += ShaderSetBind::code_set_bind_texture2d(self.set, self.bind_bone_texture, ShaderBindModelAboutSkinRowTex::VAR_BONE_TEX).as_str();
        
                    result += ShaderSetBind::code_set_bind_sampler(self.set, self.bind_bone_sampler, ShaderBindModelAboutSkinRowTex::VAR_BONE_TEX).as_str();

                    result += skin.define_code().as_str();
                },
                ESkinCode::FramesTextureInstance(_) => {
                    result += ShaderSetBind::code_set_bind_head(self.set, self.bind_bone_size).as_str();
                    result += " Bone {\r\n";
                    result += ShaderSetBind::code_uniform("vec4", ShaderBindModelAboutSkinFramesTex::VAR_BONE_TEX_SIZE).as_str();
                    result += "};\r\n";
                    
                    result += ShaderSetBind::code_set_bind_texture2d(self.set, self.bind_bone_texture, ShaderBindModelAboutSkinFramesTex::VAR_BONE_TEX).as_str();
        
                    result += ShaderSetBind::code_set_bind_sampler(self.set, self.bind_bone_sampler, ShaderBindModelAboutSkinFramesTex::VAR_BONE_TEX).as_str();
                    
                    result += skin.define_code().as_str();
                },
            }
        }

        result
    }
    
    pub fn running_code(&self) -> String {
        match &self.skin {
            Some(skin) => match skin {
                ESkinCode::None => String::from(""),
                ESkinCode::RowTexture(_) => {
                    skin.running_code()
                },
                ESkinCode::FramesTextureInstance(_) => {
                    skin.running_code()
                },
            },
            None => String::from(""),
        }
   }
}