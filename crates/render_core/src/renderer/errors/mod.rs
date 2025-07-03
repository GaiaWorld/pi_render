use crate::depend_graph::GraphError;

pub type EError = u32;

pub struct ErrorRecord(pub Vec<u32>, pub bool);
impl ErrorRecord {
    pub fn record(&mut self, _entity: usize, error: EError) {
        if self.1 { 
            // self.0.push(entity::index());
            self.0.push(error);
        }
    }
    pub fn drain(&mut self, mut count: usize) -> std::vec::Drain<'_, u32> {
        count = count.min(self.0.len());
        self.0.drain(0..count)
    }

    pub fn graphic(&mut self, node: usize, err: GraphError) {
        let error = match err {
            GraphError::NoneNGraph(_) => Self::ERROR_GRAPHIC_NONE_NGRAPHIC,
            GraphError::NoneNode(_) => Self::ERROR_GRAPHIC_NONE_NODE,
            GraphError::ExitNode(_) => Self::ERROR_GRAPHIC_EXTI_NODE,
            GraphError::RunNGraphError(_) => Self::ERROR_GRAPHIC_RUN_ERR,
            GraphError::BuildError(_) => Self::ERROR_GRAPHIC_BUILD_ERR,
            GraphError::SubGraphInputError => Self::ERROR_GRAPHIC_INPUT_ERR,
            GraphError::SubGraphOutputError => Self::ERROR_GRAPHIC_OUTPUT_ERR,
            GraphError::CustomBuildError(_) => Self::ERROR_GRAPHIC_CUSTOM_BUILD_ERR,
            GraphError::CustomRunError(_) => Self::ERROR_GRAPHIC_CUSTOM_RUN_ERR,
            GraphError::WrongNodeType => Self::ERROR_GRAPHIC_WRONG_NODE_TYPE,
            GraphError::MismatchedParam => Self::ERROR_GRAPHIC_MISMATCH_PARAM,
            GraphError::CrossGraphDepend(_, _) => Self::ERROR_GRAPHIC_BUILD_ERR,
            GraphError::ParamFillRepeat(_, _, _) => Self::ERROR_GRAPHIC_BUILD_ERR,
        };

        self.record(node, error);
    }
        
    pub const ERROR_UNKOWN: EError                          = 00001;
    pub const ERROR_VERTEX_BUFFER_CREATE_FAIL: EError       = 00002;
    pub const ERROR_BIND_BUFFER_CREATE_FAIL: EError         = 00003;
    pub const ERROR_BIND_GROUP_CREATE_FAIL: EError          = 00004;
    pub const ERROR_SHADER_CREATE_FAIL: EError              = 00005;
    pub const ERROR_PIPELINE_CREATE_FAIL: EError            = 00006;
    pub const ERROR_TEXTURE_CREATE_FAIL: EError             = 00007;
    pub const ERROR_TEXTURE_VIEW_CREATE_FAIL: EError        = 00008;
    pub const ERROR_SAMPER_CREATE_FAIL: EError              = 00009;
    pub const ERROR_BIND_VIEWER_CREATE_FAIL: EError         = 00010;

    pub const ERROR_BIND_EFFECT_CREATE_FAIL: EError         = 00011;
    pub const ERROR_MODIFY_ERROR_MATERIAL_TEXTURE: EError   = 00012;
    pub const ERROR_MATERIAL_SHADER_NOTFOUND: EError        = 00013;
    pub const ERROR_USE_MATERIAL_NULL_MAT: EError           = 00014;
    pub const ERROR_USE_MATERIAL_NULL_TARGET: EError        = 00015;
    pub const ERROR_TEXTURE_CACHE_FAIL: EError              = 00016;
    pub const ERROR_TEXTURE_CANT_LOAD_FROM_DATA: EError     = 00017;
    pub const ERROR_TEXTURE_LOAD_FAIL: EError               = 00018;
    pub const ERROR_TEXTURE_COMBINE_FAIL: EError            = 00019;
    pub const ERROR_TEXTURE_FROM_KTX_FAIL: EError           = 00019;
    
    pub const ERROR_ANIMATION_START_FAIL: EError            = 00100;
    pub const ERROR_ANIMATION_PAUSE_FAIL: EError            = 00101;
    pub const ERROR_ANIMATION_STOP_FAIL: EError             = 00102;
    pub const ERROR_ADD_TARGET_ANIMATION_FAIL: EError       = 00103;
    
    pub const ERROR_GRAPHIC_NONE_NGRAPHIC: EError           = 00100;
    pub const ERROR_GRAPHIC_NONE_NODE: EError               = 00101;
    pub const ERROR_GRAPHIC_EXTI_NODE: EError               = 00102;
    pub const ERROR_GRAPHIC_RUN_ERR: EError                 = 00103;
    pub const ERROR_GRAPHIC_BUILD_ERR: EError               = 00104;
    pub const ERROR_GRAPHIC_INPUT_ERR: EError               = 00105;
    pub const ERROR_GRAPHIC_OUTPUT_ERR: EError              = 00106;
    pub const ERROR_GRAPHIC_CUSTOM_BUILD_ERR: EError        = 00107;
    pub const ERROR_GRAPHIC_CUSTOM_RUN_ERR: EError          = 00108;
    pub const ERROR_GRAPHIC_WRONG_NODE_TYPE: EError         = 00109;
    pub const ERROR_GRAPHIC_MISMATCH_PARAM: EError          = 00110;
    pub const ERROR_SUB_GRAPHIC_ERROR: EError               = 00111;
    
    pub const ERROR_GLTF_BIN_LOAD_FAIL: EError              = 00200;
    pub const ERROR_GLTF_BUFFER: EError                     = 00201;
    pub const ERROR_GLTF_ACCESSOR: EError                   = 00202;
    pub const ERROR_GLTF_IMAGE: EError                      = 00203;
    pub const ERROR_GLTF_GLTF_LOAD: EError                  = 00204;
    pub const ERROR_GLTF_GLTF_PARSE: EError                 = 00205;
    pub const ERROR_GLTF_GLTF_CACHE: EError                 = 00207;
    pub const ERROR_GLTF_VERTEX_BUFFER: EError              = 00208;
    pub const ERROR_GLTF_ANIMATION: EError                  = 00209;

    pub const ERROR_ENTITY_NONE: EError                     = 10001;
    pub const ERROR_ENTITY_DISPOSED: EError                 = 10002;

    pub const ERROR_SCENE_NONE: EError                      = 20001;
    pub const ERROR_SCENE_BIND_FAIL: EError                 = 20002;
    pub const ERROR_ENVIRONMENT_INFO_PARSE: EError          = 20003;
    pub const ERROR_ENVIRONMENT_INFO_MAGICNUMBER: EError    = 20004;
    
    
    pub const ERROR_RENDERER_NOT_FOUND: EError              = 20100;

    pub const ERROR_PASS_BIND_SCENE_NONE: EError            = 50000;
    pub const ERROR_PASS_BIND_VIEWER_NONE: EError           = 50001;
    pub const ERROR_PASS_SET0_FAIL: EError                  = 50002;
    pub const ERROR_PASS_BIND_MODEL_NONE: EError            = 50003;
    pub const ERROR_PASS_BIND_EFFECT_VALUE_NONE: EError     = 50004;
    pub const ERROR_PASS_BIND_LIGHTING_NONE: EError         = 50005;
    pub const ERROR_PASS_BIND_SKIN_NONE: EError             = 50006;
    pub const ERROR_PASS_SET1_FAIL: EError                  = 50007;
    pub const ERROR_PASS_SET2_FAIL: EError                  = 50008;
    pub const ERROR_PASS_BIND_SHADOW_NONE: EError           = 50010;
    pub const ERROR_PASS_BIND_BRDF_NONE: EError             = 50011;
    pub const ERROR_PASS_BIND_CAMERA_OPAQUE_NONE: EError    = 50012;
    pub const ERROR_PASS_BIND_CAMERA_DEPTH_NONE: EError     = 50013;
    pub const ERROR_PASS_BIND_ENV_NONE: EError              = 50014;
    pub const ERROR_PASS_SET3_FAIL: EError                  = 50015;
    pub const ERROR_PASS_BIND_GROUPS_FAIL: EError           = 50016;
    pub const ERROR_PASS_SHADER_FAIL: EError                = 50017;
    pub const ERROR_PASS_PIPELINE_FAIL: EError              = 50018;
    pub const ERROR_PASS_DRAW_FAIL: EError                  = 50019;
    pub const ERROR_PASS_BIND_VELOCITY_NONE: EError         = 50020;
    pub const ERROR_PASS_BIND_MODEL_INV_NONE: EError        = 50021;
    pub const ERROR_PASS_BIND_MORPH_NONE: EError            = 50022;
    pub const ERROR_PASS_BIND_SKININS_NONE: EError          = 50023;

}

