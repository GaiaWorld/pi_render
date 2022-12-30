
pub enum ESceneAboutBind {
    Camera,
    Fog,
    Time,
    BRDFTexture,
    BRDFSampler,
    EnvInfo,
    EnvTexture,
    EnvSampler,
}

pub enum EModelAboutBind {
    ModelMatrix,
    SkinInfo,
    SkinTexture,
    SkinSampler,
}

pub enum EOtherAboutBind {
    LightInfo,
}

pub struct ShaderSetBind;
impl ShaderSetBind {
    pub const SET_SCENE_ABOUT: u32 = 0;
    pub const SET_EFFECT_ABOUT: u32 = 1;
    pub const SET_MODEL_ABOUT: u32 = 2;
    pub const SET_OTHER: u32 = 3;
    // pub const 
    pub fn code_uniform(kind: &str, name: &str) -> String {
        String::from(kind) + " " + name + ";\r\n"
    }
    pub fn code_set_bind_head(set: u32, bind: u32) -> String {
        let mut result = String::from("layout(set = ");
        result += set.to_string().as_str();
        result += ", binding = ";
        result += bind.to_string().as_str();
        result += ") uniform ";

        result
    }
    pub fn code_set_bind_texture2d(set: u32, bind: u32, name: &str) -> String {
        Self::code_set_bind_head(set, bind) + Self::code_uniform("texture2D", name).as_str()
    }
    pub fn code_set_bind_sampler(set: u32, bind: u32, tex_name: &str) -> String {
        let name = String::from("sampler") + tex_name;
        Self::code_set_bind_head(set, bind) + Self::code_uniform("sampler", &name).as_str()
    }
}
