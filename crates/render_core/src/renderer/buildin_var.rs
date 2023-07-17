
pub struct ShaderVarUniform;
impl ShaderVarUniform {
    pub const VIEW_MATRIX               : &'static str = "PI_MATRIX_V"          ;
    pub const VIEW_ROTATION_MATRIX_INV  : &'static str = "PI_MATRIX_V_R_INV"    ;
    pub const PROJECT_MATRIX            : &'static str = "PI_MATRIX_P"          ;
    pub const VIEW_PROJECT_MATRIX       : &'static str = "PI_MATRIX_VP"         ;
    pub const CAMERA_POSITION           : &'static str = "PI_CAMERA_POSITION"   ;
    pub const CAMERA_DIRECTION          : &'static str = "PI_VIEW_DIRECTION"    ;

    pub const TIME                      : &'static str = "PI_Time"              ;
    pub const DELTA_TIME                : &'static str = "PI_DeltaTime"         ;

    pub const FOG_INFO                  : &'static str = "PI_FogInfo"           ;
    pub const FOG_PARAM                 : &'static str = "PI_FogParam"          ;

    pub const AMBIENT_PARAM             : &'static str = "PI_Ambient"           ;

    pub const WORLD_MATRIX              : &'static str = "PI_ObjectToWorld"     ;
    pub const _WORLD_MATRIX             : &'static str = "U_PI_ObjectToWorld"   ;
    pub const WORLD_MATRIX_INV          : &'static str = "PI_WorldToObject"     ;
    pub const _WORLD_MATRIX_INV         : &'static str = "U_PI_WorldToObject"   ;
    pub const VELOCITY                  : &'static str = "PI_ObjectVelocity"    ;
    pub const _VELOCITY                 : &'static str = "U_PI_ObjectVelocity"  ;
    pub const _SKIN_BONE_OFFSET0        : &'static str = "U_PI_SkinBoneOffset0" ;
    pub const _SKIN_BONE_OFFSET1        : &'static str = "U_PI_SkinBoneOffset1" ;
    
    pub const BONE_MATRICES             : &'static str = "boneMatrices"         ;
    pub const BONE_TEX_SIZE             : &'static str = "bondTexSize"          ;
    pub const BONE_TEX                  : &'static str = "_boneTex"             ;
    pub const BONE_TEX_SAMPLER          : &'static str = "sampler_boneTex"      ;

    pub const RENDER_ALIGNMENT          : &'static str = "RENDER_ALIGNMENT"     ;
    pub const RENDER_PIVOT              : &'static str = "RENDER_PIVOT"         ;
}

pub struct ShaderVarVertices;
impl ShaderVarVertices {
    pub const POSITION                  : &'static str = "A_POSITION"          ;
    pub const POSITION2D                : &'static str = "A_POSITION_2D"       ;
    pub const COLOR4                    : &'static str = "A_COLOR4"            ;
    pub const UV                        : &'static str = "A_UV"                ;
    pub const NORMAL                    : &'static str = "A_NORMAL"            ;
    pub const TANGENT                   : &'static str = "A_TANGENT"           ;
    pub const MATRICES_INDICES          : &'static str = "A_JOINT_INC"         ;
    pub const MATRICES_WEIGHTS          : &'static str = "A_JOINT_WEG"         ;
    pub const MATRICES_INDICES_EXTRA    : &'static str = "A_JOINT_INC_EX"      ;
    pub const MATRICES_WEIGHTS_EXTRA    : &'static str = "A_JOINT_WEG_EX"      ;
    pub const UV2                       : &'static str = "A_UV2"               ;
    pub const UV3                       : &'static str = "A_UV3"               ;
    pub const UV4                       : &'static str = "A_UV4"               ;
    pub const UV5                       : &'static str = "A_UV5"               ;
    pub const UV6                       : &'static str = "A_UV6"               ;
    pub const CUSTOM_VEC4_A             : &'static str = "A_CustomV4A"         ;
    pub const CUSTOM_VEC4_B             : &'static str = "A_CustomV4B"         ;
    pub const CUSTOM_VEC3_A             : &'static str = "A_CustomV3A"         ;
    pub const CUSTOM_VEC3_B             : &'static str = "A_CustomV3B"         ;
    pub const CUSTOM_VEC2_A             : &'static str = "A_CustomV2A"         ;
    pub const CUSTOM_VEC2_B             : &'static str = "A_CustomV2B"         ;
    pub const INS_WORLD_ROW1            : &'static str = "A_INS_World1"        ;
    pub const INS_WORLD_ROW2            : &'static str = "A_INS_World2"        ;
    pub const INS_WORLD_ROW3            : &'static str = "A_INS_World3"        ;
    pub const INS_WORLD_ROW4            : &'static str = "A_INS_World4"        ;
    pub const INS_COLOR                 : &'static str = "A_INS_Color"         ;
    pub const INS_TILL_OFFSET1          : &'static str = "A_INS_TillOff1"      ;
    pub const INS_TILL_OFFSET2          : &'static str = "A_INS_TillOff2"      ;
    pub const INS_CUSTOM_VEC4_A         : &'static str = "A_INS_Vec4A"         ;
    pub const INS_CUSTOM_VEC4_B         : &'static str = "A_INS_Vec4B"         ;
    pub const INS_CUSTOM_UVEC4_A        : &'static str = "A_INS_UVec4A"        ;
    pub const INS_CUSTOM_IVEC4_B        : &'static str = "A_INS_IVec4B"        ;
    pub const MATRICES_INDICES1         : &'static str = "A_JOINT_INC1"        ;
    pub const MATRICES_WEIGHTS1         : &'static str = "A_JOINT_WEG1"        ;
    pub const MATRICES_INDICES2         : &'static str = "A_JOINT_INC2"        ;
    pub const MATRICES_WEIGHTS2         : &'static str = "A_JOINT_WEG2"        ;
    pub const MATRICES_INDICES_EXTRA2   : &'static str = "A_JOINT_INC_EX2"     ;
    pub const MATRICES_WEIGHTS_EXTRA2   : &'static str = "A_JOINT_WEG_EX2"     ;
    pub const MATRICES_INDICES3         : &'static str = "A_JOINT_INC3"        ;
    pub const MATRICES_WEIGHTS3         : &'static str = "A_JOINT_WEG3"        ;
    pub const MATRICES_INDICES_EXTRA3   : &'static str = "A_JOINT_INC_EX3"     ;
    pub const MATRICES_WEIGHTS_EXTRA3   : &'static str = "A_JOINT_WEG_EX3"     ;

    pub const INS_VELOCITY              : &'static str = "A_INS_Velocity"      ;
    pub const INS_SKIN_BONE_OFFSET0     : &'static str = "A_INS_SkinBoneOffset0";
    pub const INS_SKIN_BONE_OFFSET1     : &'static str = "A_INS_SkinBoneOffset1";
    pub const PARTICLE_AGE_LIFE         : &'static str = "PARTICLE_AGE_LIFE"    ;
    pub const PARTICLE_POSITION         : &'static str = "PARTICLE_POSITION"    ;
    pub const PARTICLE_SCALING          : &'static str = "PARTICLE_SCALING"     ;
    pub const PARTICLE_ROTATION         : &'static str = "PARTICLE_ROTATION"    ;
    pub const PARTICLE_DIRECTION        : &'static str = "PARTICLE_DIRECTION"   ;
    pub const PARTICLE_COLOR            : &'static str = "PARTICLE_COLOR"       ;
    pub const PARTICLE_TILLOFF          : &'static str = "PARTICLE_TILLOFF"     ;
}

pub struct ShaderDefaultTexture;
impl ShaderDefaultTexture {
    pub const WHITE_2D                  : &'static str = "White2D"              ;
    pub const BLACK_2D                  : &'static str = "Black2D"              ;
}
