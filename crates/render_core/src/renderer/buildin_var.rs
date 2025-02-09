
pub struct ShaderVarUniform;
impl ShaderVarUniform {
    pub const VIEW_MATRIX               : &'static str = "PI_MATRIX_V"          ;
    pub const VIEW_ROTATION_MATRIX_INV  : &'static str = "PI_MATRIX_V_R_INV"    ;
    pub const PROJECT_MATRIX            : &'static str = "PI_MATRIX_P"          ;
    pub const VIEW_PROJECT_MATRIX       : &'static str = "PI_MATRIX_VP"         ;
    pub const CAMERA_POSITION           : &'static str = "PI_CAMERA_POSITION"   ;
    pub const CAMERA_DIRECTION          : &'static str = "PI_VIEW_DIRECTION"    ;

    pub const BRDF_TEXUTRE              : &'static str = "_BRDFTexture"         ;
    pub const CAMERA_OPAQUE_TEXUTRE     : &'static str = "_CameraOpaqueTexture" ;
    pub const CAMERA_DEPTH_TEXUTRE      : &'static str = "_CameraDepthTexture"  ;
    
    pub const ENVIRONMENT_TEXUTRE       : &'static str = "_EnvironmentTexture"  ;
    pub const IBL_X                     : &'static str = "_SphericalX"          ;
    pub const IBL_Y                     : &'static str = "_SphericalY"          ;
    pub const IBL_Z                     : &'static str = "_SphericalZ"          ;
    pub const IBL_XY                    : &'static str = "_SphericalXY"         ;
    pub const IBL_YZ                    : &'static str = "_SphericalYZ"         ;
    pub const IBL_ZX                    : &'static str = "_SphericalZX"         ;
    pub const IBL_XX_ZZ                 : &'static str = "_SphericalXX_ZZ"      ;
    pub const IBL_YY_ZZ                 : &'static str = "_SphericalYY_ZZ"      ;
    pub const IBL_ZZ                    : &'static str = "_SphericalZZ"         ;

    pub const LIGHTING_INFOS            : &'static str = "_LightingInfos"       ;
    pub const DIRECT_LIGHT_DIRECTION    : &'static str = "_DirectLightDirection";
    pub const DIRECT_LIGHT_COLOR        : &'static str = "_DirectLightColor"    ;
    pub const POINT_LIGHT_POSITION      : &'static str = "_PointLightPosition"  ;
    pub const POINT_LIGHT_COLOR         : &'static str = "_PointLightColor"     ;
    pub const POINT_LIGHT_DATA          : &'static str = "_PointLightData"      ;
    pub const SPOT_LIGHT_POSITION       : &'static str = "_SpotLightPosition"   ;
    pub const SPOT_LIGHT_COLOR          : &'static str = "_SpotLightColor"      ;
    pub const SPOT_LIGHT_DATA           : &'static str = "_SpotLightData"       ;
    pub const SPOT_LIGHT_DIRECTION      : &'static str = "_SpotLightDirection"  ;
    pub const HEMI_LIGHT_POSITION       : &'static str = "_HemiLightPosition"   ;
    pub const HEMI_LIGHT_COLOR          : &'static str = "_HemiLightColor"      ;
    pub const HEMI_LIGHT_DATA           : &'static str = "_HemiLightData"       ;
    pub const HEMI_LIGHT_DIRECTION      : &'static str = "_HemiLightDirection"  ;
    pub const SHADOWMAP_TEXTURE         : &'static str = "_ShadowMap"           ;

    pub const MODEL_LIGHTS_COUNT        : &'static str = "_MLightsCount"        ;
    pub const MODEL_LIGHTS_INDEXS       : &'static str = "_MLightsIndexs"       ;
    pub const MODEL_DIRECTLIGHT_COUNT   : &'static str = "_MDirectLightCount"   ;
    pub const MODEL_POINTLIGHT_COUNT    : &'static str = "_MPointLightCount"    ;
    pub const MODEL_SPOTLIGHT_COUNT     : &'static str = "_MSpotLightCount"     ;
    pub const MODEL_HEMILIGHT_COUNT     : &'static str = "_MHemiLightCount"     ;
    pub const DIRECT_LIGHT_INDEXS       : &'static str = "_DirectLightingIdxs"  ;
    pub const POINT_LIGHT_INDEXS        : &'static str = "_PointLightingIdxs"   ;
    pub const SPOT_LIGHT_INDEXS         : &'static str = "_SpotLightingIdxs"    ;
    pub const HEMI_LIGHT_INDEXS         : &'static str = "_HemiLightingIdxs"    ;

    pub const SHADOWMAP_LIGHT_INDEXS    : &'static str = "_ShadowLightIdxs"     ;
    pub const SHADOWMAP_MATRIX          : &'static str = "_ShadowMapMatrix"     ;
    pub const SHADOWMAP_BIAS_ANS_SCALE  : &'static str = "_BiasAndScaleSM"      ;
    pub const SHADOWMAP_DEPTH_VALUES    : &'static str = "_DepthValuesSM"       ;
    pub const SHADOWMAP_TILLOFF         : &'static str = "_ShadowMapTilloff"    ;

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
    pub const _MATIDX                   : &'static str = "U_PI_MatIdxs"         ;
    pub const MATIDX                    : &'static str = "PI_MatIdxs"           ;
    
    pub const BONE_MATRICES             : &'static str = "boneMatrices"         ;
    pub const BONE_TEX_SIZE             : &'static str = "bondTexSize"          ;
    pub const BONE_TEX                  : &'static str = "_boneTex"             ;
    pub const BONE_TEX_SAMPLER          : &'static str = "sampler_boneTex"      ;

    pub const RENDER_ALIGNMENT          : &'static str = "RENDER_ALIGNMENT"     ;
    pub const RENDER_PIVOT              : &'static str = "RENDER_PIVOT"         ;
}

pub struct ShaderVarVertices;
impl ShaderVarVertices {
    pub const OBJECT_INDEX              : &'static str = "PI_ObjectIndex"      ;
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
    pub const CUSTOM_VEC4_C             : &'static str = "A_CustomV4C"         ;
    pub const CUSTOM_VEC4_D             : &'static str = "A_CustomV4D"         ;
    pub const CUSTOM_VEC3_A             : &'static str = "A_CustomV3A"         ;
    pub const CUSTOM_VEC3_B             : &'static str = "A_CustomV3B"         ;
    pub const CUSTOM_VEC2_A             : &'static str = "A_CustomV2A"         ;
    pub const CUSTOM_VEC2_B             : &'static str = "A_CustomV2B"         ;
    pub const INSTANCE_INDEX            : &'static str = "A_INS_INDEX"         ;
    pub const INS_WORLD_ROW1            : &'static str = "A_INS_World1"        ;
    pub const INS_WORLD_ROW2            : &'static str = "A_INS_World2"        ;
    pub const INS_WORLD_ROW3            : &'static str = "A_INS_World3"        ;
    pub const INS_WORLD_ROW4            : &'static str = "A_INS_World4"        ;
    pub const INS_COLOR                 : &'static str = "A_INS_Color"         ;
    pub const INS_TILL_OFFSET1          : &'static str = "A_INS_TillOff1"      ;
    pub const INS_TILL_OFFSET2          : &'static str = "A_INS_TillOff2"      ;
    pub const INS_CUSTOM_VEC4_A         : &'static str = "A_INS_F_Vec4A"       ;
    pub const INS_CUSTOM_VEC4_B         : &'static str = "A_INS_F_Vec4B"       ;
    pub const INS_CUSTOM_VEC4_C         : &'static str = "A_INS_F_Vec4C"       ;
    pub const INS_CUSTOM_VEC4_D         : &'static str = "A_INS_F_Vec4D"       ;
    pub const INS_MAT_IDX               : &'static str = "A_INS_MATIDX"        ;
    pub const INS_MODEL_MAT_SKIN        : &'static str = "A_INS_MODEL_MAT_SKIN";
    
    pub const INS_VEC4_A                : &'static str = "A_INS_Vec4A"         ;
    pub const INS_VEC4_B                : &'static str = "A_INS_Vec4B"         ;
    pub const INS_VEC4_C                : &'static str = "A_INS_Vec4C"         ;
    pub const INS_VEC4_D                : &'static str = "A_INS_Vec4D"         ;
    pub const INS_VEC4_E                : &'static str = "A_INS_Vec4E"         ;
    pub const INS_VEC4_F                : &'static str = "A_INS_Vec4F"         ;
    pub const INS_VEC4_G                : &'static str = "A_INS_Vec4G"         ;
    pub const INS_VEC4_H                : &'static str = "A_INS_Vec4H"         ;

    pub const INS_VEC3_A                : &'static str = "A_INS_Vec3A"         ;
    pub const INS_VEC3_B                : &'static str = "A_INS_Vec3B"         ;
    pub const INS_VEC3_C                : &'static str = "A_INS_Vec3C"         ;
    pub const INS_VEC3_D                : &'static str = "A_INS_Vec3D"         ;
    pub const INS_VEC3_E                : &'static str = "A_INS_Vec3E"         ;
    pub const INS_VEC3_F                : &'static str = "A_INS_Vec3F"         ;
    pub const INS_VEC3_G                : &'static str = "A_INS_Vec3G"         ;
    pub const INS_VEC3_H                : &'static str = "A_INS_Vec3H"         ;

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
    pub const TRAIL_AXIS_X              : &'static str = "TRAIL_AXIS_X"     ;
    pub const TRAIL_AXIS_Z              : &'static str = "TRAIL_AXIS_Z"     ;
    pub const TRAIL_INFO                : &'static str = "TRAIL_INFO"     ;
}

pub struct ShaderVarSurface;
impl ShaderVarSurface {
    pub const POSITION                  : &'static str = "P"            ;
    pub const DIFFUSE                   : &'static str = "Diffuse"      ;
    pub const SPECULAR                  : &'static str = "Specular"     ;
    pub const EMISSIVE                  : &'static str = "Emissive"     ;
    pub const AMBIENT                   : &'static str = "Ambient"      ;
    pub const NORMAL                    : &'static str = "N"            ;
    pub const VIEW                      : &'static str = "V"            ;
    pub const N_DOT_V                   : &'static str = "NdotV"        ;
    pub const GLOSSINESS                : &'static str = "Glossiness"   ;
    pub const LIGHTMAP                  : &'static str = "LightMap"     ;
}

/// Varying 变量
pub struct ShaderVarVarying;
impl ShaderVarVarying {
    pub const POSITION                  : &'static str = "v_pos"        ;
    pub const NORMAL                    : &'static str = "v_normal"     ;
    pub const TANGENT                   : &'static str = "v_tangent"    ;
    pub const COLOR                     : &'static str = "v_color"      ;
    pub const UV                        : &'static str = "v_uv"         ;
    pub const UV2                       : &'static str = "v_uv2"        ;
    pub const UV3                       : &'static str = "v_uv3"        ;
    pub const UV4                       : &'static str = "v_uv4"        ;
    pub const UV5                       : &'static str = "v_uv5"        ;
    pub const UV6                       : &'static str = "v_uv6"        ;
    pub const UV7                       : &'static str = "v_uv7"        ;
    pub const UV8                       : &'static str = "v_uv8"        ;
}


pub struct ShaderDefaultTexture;
impl ShaderDefaultTexture {
    pub const WHITE_2D                  : &'static str = "White2D"              ;
    pub const BLACK_2D                  : &'static str = "Black2D"              ;
}
