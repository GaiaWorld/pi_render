
pub trait ShaderBind {
    fn define_code(&self) -> String;
}

pub struct ShaderBindSceneAboutCamera;
impl ShaderBindSceneAboutCamera {
    pub const VAR_VIEW_MATRIX:              &str = "PI_MATRIX_V";
    pub const VAR_PROJECT_MATRIX:           &str = "PI_MATRIX_P";
    pub const VAR_VIEW_PROJECT_MATRIX:      &str = "PI_MATRIX_VP";
    pub const VAR_CAMERA_POSITION:          &str = "PI_CAMERA_POSITION";
    pub const VAR_CAMERA_DIRECTION:         &str = "PI_VIEW_DIRECTION";

    pub const OFFSET_VIEW_MATRIX:           wgpu::BufferAddress = 0;
    pub const OFFSET_PROJECT_MATRIX:        wgpu::BufferAddress = 16 * 4;
    pub const OFFSET_VIEW_PROJECT_MATRIX:   wgpu::BufferAddress = 16 * 4 + 16 * 4;
    pub const OFFSET_CAMERA_POSITION:       wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4;
    pub const OFFSET_CAMERA_DIRECTION:      wgpu::BufferAddress = 16 * 4 + 16 * 4 + 16 * 4 + 4 * 4;
}


pub struct ShaderBindSceneAboutTime;
impl ShaderBindSceneAboutTime {
    pub const VAR_TIME:                     &str = "PI_Time";
    pub const VAR_DELTA_TIME:               &str = "PI_DeltaTime";

    pub const OFFSET_TIME:                  wgpu::BufferAddress = 0;
    pub const OFFSET_DELTA_TIME:            wgpu::BufferAddress = 4 * 4;
}

pub struct ShaderBindSceneAboutFog;
impl ShaderBindSceneAboutFog {
    pub const VAR_FOG_INFO:                 &str = "PI_FogInfo";
    pub const VAR_FOG_PARAM:                &str = "PI_FogParam";

    pub const OFFSET_FOG_INFO:              wgpu::BufferAddress = 0;
    pub const OFFSET_FOG_PARAM:             wgpu::BufferAddress = 4 * 4;
}


pub struct ShaderBindModelAboutMatrix;
impl ShaderBindModelAboutMatrix {
    pub const VAR_WORLD_MATRIX:             &str = "PI_ObjectToWorld";
    pub const VAR_WORLD_MATRIX_INV:         &str = "PI_WorldToObject";
    
    pub(crate) const _VAR_WORLD_MATRIX:     &str = "U_PI_ObjectToWorld";
    pub(crate) const _VAR_WORLD_MATRIX_INV: &str = "U_PI_WorldToObject";

    pub const OFFSET_WORLD_MATRIX:          wgpu::BufferAddress = 0;
    pub const OFFSET_WORLD_MATRIX_INV:      wgpu::BufferAddress = 16 * 4;
}

pub struct ShaderBindModelAboutSkinRowTex;
impl ShaderBindModelAboutSkinRowTex {
    pub const VAR_BONE_TEX_SIZE:            &str = "bondTexSize";
    pub const VAR_BONE_TEX:                 &str = "_boneTex";

    pub const OFFSET_BONE_TEX_SIZE:         wgpu::BufferAddress = 0;
}

pub struct ShaderBindModelAboutSkinFramesTex;
impl ShaderBindModelAboutSkinFramesTex {
    pub const VAR_BONE_TEX_SIZE:            &str = "bondTexSize";
    pub const VAR_BONE_TEX:                 &str = "_boneTex";

    pub const OFFSET_BONE_TEX_SIZE:         wgpu::BufferAddress = 0;
}