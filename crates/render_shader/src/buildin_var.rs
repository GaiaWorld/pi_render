
pub struct ShaderVarUniform;
impl ShaderVarUniform {
    pub const VIEW_MATRIX               : &'static str = "PI_MATRIX_V"          ;
    pub const PROJECT_MATRIX            : &'static str = "PI_MATRIX_P"          ;
    pub const VIEW_PROJECT_MATRIX       : &'static str = "PI_MATRIX_VP"         ;
    pub const CAMERA_POSITION           : &'static str = "PI_CAMERA_POSITION"   ;
    pub const CAMERA_DIRECTION          : &'static str = "PI_VIEW_DIRECTION"    ;

    pub const TIME                      : &'static str = "PI_Time"              ;
    pub const DELTA_TIME                : &'static str = "PI_DeltaTime"         ;

    pub const FOG_INFO                  : &'static str = "PI_FogInfo"           ;
    pub const FOG_PARAM                 : &'static str = "PI_FogParam"          ;

    pub const WORLD_MATRIX              : &'static str = "PI_ObjectToWorld"     ;
    pub const WORLD_MATRIX_INV          : &'static str = "PI_WorldToObject"     ;

    pub(crate) const _WORLD_MATRIX      : &'static str = "U_PI_ObjectToWorld"   ;
    pub(crate) const _WORLD_MATRIX_INV  : &'static str = "U_PI_WorldToObject"   ;
    
    pub const BONE_TEX_SIZE             : &'static str = "bondTexSize"          ;
    pub const BONE_TEX                  : &'static str = "_boneTex"             ;
}
