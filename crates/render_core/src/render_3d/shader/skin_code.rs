use std::sync::Arc;

use pi_atom::Atom;

use crate::renderer::{buildin_var::ShaderVarUniform, buildin_data::EDefaultTexture, shader_stage::EShaderStage};

use super::uniform_texture::{UniformTexture2DDesc, UniformSamplerDesc};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ESkinBonesPerVertex {
    One,
    Two,
    Three,
    Four,
}
impl ESkinBonesPerVertex {
    pub fn define_code_for_ubo(&self) -> String {
        String::from("")
    }
    pub fn running_code_for_ubo(&self) -> String {
        match self {
            ESkinBonesPerVertex::One =>  {
                String::from("

    mat4 influence = boneMatrices[A_JOINT_INC1[0]];
    PI_ObjectToWorld = PI_ObjectToWorld * influence; 

                ")
            },
            ESkinBonesPerVertex::Two =>  {
                String::from("

    mat4 influence   = boneMatrices[A_JOINT_INC2[0]] * A_JOINT_WEG2[0];
    influence       += boneMatrices[A_JOINT_INC2[1]] * A_JOINT_WEG2[1];
    PI_ObjectToWorld = PI_ObjectToWorld * influence; 

                ")
            },
            ESkinBonesPerVertex::Three =>  {
                String::from("

    mat4 influence   = boneMatrices[A_JOINT_INC3[0]] * A_JOINT_WEG3[0];
    influence       += boneMatrices[A_JOINT_INC3[0]] * A_JOINT_WEG3[1];
    influence       += boneMatrices[A_JOINT_INC3[0]] * A_JOINT_WEG3[2];
    PI_ObjectToWorld = PI_ObjectToWorld * influence; 

                ")
            },
            ESkinBonesPerVertex::Four => {
                String::from("

    mat4 influence   = boneMatrices[A_JOINT_INC[0]] * A_JOINT_WEG[0];
    influence       += boneMatrices[A_JOINT_INC[1]] * A_JOINT_WEG[1];
    influence       += boneMatrices[A_JOINT_INC[2]] * A_JOINT_WEG[2];
    influence       += boneMatrices[A_JOINT_INC[3]] * A_JOINT_WEG[3];
    PI_ObjectToWorld = PI_ObjectToWorld * influence;

                ")
            },
        }
    }
    pub fn define_code_for_tex(&self) -> String {
        String::from("
#define inline
mat4 readMatrixFromTex(texture2D tex, sampler samp, float index, float texWidth, float row, float texHeight) {
    float offset = index * 4.0;
    float dx = 1. / texWidth;
    float dy = row * 1. / texHeight;
    vec4 m0 = texture(sampler2D(tex, samp), vec2(dx * (offset + 0.5), dy));
    vec4 m1 = texture(sampler2D(tex, samp), vec2(dx * (offset + 1.5), dy));
    vec4 m2 = texture(sampler2D(tex, samp), vec2(dx * (offset + 2.5), dy));
    vec4 m3 = texture(sampler2D(tex, samp), vec2(dx * (offset + 3.5), dy));

    return mat4(m0, m1, m2, m3);
}

        ")
    }
    pub fn running_code_for_tex(&self) -> String {
        match self {
            ESkinBonesPerVertex::One =>  {
                String::from("

    // mat4 influence = readMatrixFromTex(_boneTex, sampler_boneTex, A_JOINT_INC1.x * 1.0, bondTexSize.x, 0., bondTexSize.y);
    // PI_ObjectToWorld = PI_ObjectToWorld * influence; 

                ")
            },
            ESkinBonesPerVertex::Two =>  {
                String::from("

    mat4 influence   = readMatrixFromTex(sampler2D(_boneTex, sampler_boneTex), A_JOINT_INC2[0], bondTexSize.x, 0., bondTexSize.y)  * A_JOINT_WEG2[0];
    influence       += readMatrixFromTex(sampler2D(_boneTex, sampler_boneTex), A_JOINT_INC2[1], bondTexSize.x, 0., bondTexSize.y) * A_JOINT_WEG2[1];
    PI_ObjectToWorld = PI_ObjectToWorld * influence; 

                ")
            },
            ESkinBonesPerVertex::Three =>  {
                String::from("

    mat4 influence   = readMatrixFromTex(sampler2D(_boneTex, sampler_boneTex), A_JOINT_INC3[0], bondTexSize.x, 0., bondTexSize.y)  * A_JOINT_WEG3[0];
    influence       += readMatrixFromTex(sampler2D(_boneTex, sampler_boneTex), A_JOINT_INC3[1], bondTexSize.x, 0., bondTexSize.y) * A_JOINT_WEG3[1];
    influence       += readMatrixFromTex(sampler2D(_boneTex, sampler_boneTex), A_JOINT_INC3[2], bondTexSize.x, 0., bondTexSize.y) * A_JOINT_WEG3[2];
    PI_ObjectToWorld = PI_ObjectToWorld * influence; 

                ")
            },
            ESkinBonesPerVertex::Four => {
                String::from("

    mat4 influence   = readMatrixFromTex(sampler2D(_boneTex, sampler_boneTex), A_JOINT_INC[0], bondTexSize.x, 0., bondTexSize.y)  * A_JOINT_WEG[0];
    influence       += readMatrixFromTex(sampler2D(_boneTex, sampler_boneTex), A_JOINT_INC[1], bondTexSize.x, 0., bondTexSize.y) * A_JOINT_WEG[1];
    influence       += readMatrixFromTex(sampler2D(_boneTex, sampler_boneTex), A_JOINT_INC[2], bondTexSize.x, 0., bondTexSize.y) * A_JOINT_WEG[2];
    influence       += readMatrixFromTex(sampler2D(_boneTex, sampler_boneTex), A_JOINT_INC[3], bondTexSize.x, 0., bondTexSize.y) * A_JOINT_WEG[3];
    PI_ObjectToWorld = PI_ObjectToWorld * influence;

                ")
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ESkinCode {
    None,
    UBO(ESkinBonesPerVertex, EBoneCount),
    RowTexture(ESkinBonesPerVertex),
    FramesTexture(ESkinBonesPerVertex),
}
impl Default for ESkinCode {
    fn default() -> Self {
        Self::None
    }
}
impl ESkinCode {
    pub fn define_code(&self) -> String {
        match self {
            ESkinCode::None => String::from(""),
            ESkinCode::UBO(temp, _) => temp.define_code_for_ubo(),
            ESkinCode::RowTexture(temp) => temp.define_code_for_tex(),
            ESkinCode::FramesTexture(temp) => temp.define_code_for_tex(),
        }
    }
    pub fn running_code(&self) -> String {
        match self {
            ESkinCode::None => String::from(""),
            ESkinCode::UBO(temp, _) => temp.running_code_for_ubo(),
            ESkinCode::RowTexture(temp) => temp.running_code_for_tex(),
            ESkinCode::FramesTexture(temp) => temp.running_code_for_tex(),
        }
    }
    pub fn uniform_desc_tex() -> UniformTexture2DDesc {
        UniformTexture2DDesc::new(
            Atom::from(ShaderVarUniform::BONE_TEX),
            wgpu::TextureSampleType::Float { filterable: false },
            false,
            EShaderStage::VERTEX,
            EDefaultTexture::White,
        )
    }
    pub fn uniform_desc_sampler() -> UniformSamplerDesc {
        UniformSamplerDesc {
            slotname: Atom::from(ShaderVarUniform::BONE_TEX_SAMPLER),
            ty: wgpu::SamplerBindingType::NonFiltering,
            stage: EShaderStage::VERTEX,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EBoneCount {
    N16 = 16,
    N32 = 32,
    N64 = 64,
    N128 = 128,
    N256 = 256,
}
impl EBoneCount {
    pub fn new(bone_count: u8) -> Self {
        if bone_count <= 16 {
            Self::N16
        }
        else if bone_count <= 32 {
            Self::N32
        }
        else if bone_count <= 64 {
            Self::N64
        }
        else if bone_count <= 128 {
            Self::N128
        }
        else {
            Self::N256
        }
    }
    pub fn use_bytes(&self) -> usize {
        match self {
            EBoneCount::N16 => 16 * 16 * 4,
            EBoneCount::N32 => 32 * 16 * 4,
            EBoneCount::N64 => 64 * 16 * 4,
            EBoneCount::N128 => 128 * 16 * 4,
            EBoneCount::N256 => 256 * 16 * 4,
        }
    }
    pub fn count(&self) -> u32 {
        match self {
            EBoneCount::N16     =>  16,
            EBoneCount::N32     =>  32,
            EBoneCount::N64     =>  64,
            EBoneCount::N128    => 128,
            EBoneCount::N256    => 256,
        }
    }
}