use crate::{unifrom_code::{UniformTextureDesc, UniformPropertyName}, set_bind::ShaderSetBind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ESkinBonesPerVertex {
    One,
    Two,
    Three,
    Four,
}
impl ESkinBonesPerVertex {
    pub fn define_code(&self) -> String {
        String::from("
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
    pub fn running_code(&self) -> String {
        match self {
            ESkinBonesPerVertex::One =>  {
                String::from("
                mat4 influence;
                influence = readMatrixFromTex(_boneTex, sampler_boneTex, matricesIndices[0], bondTexSize.x, 0., bondTexSize.y)  * matricesWeights[0];
                PI_ObjectToWorld = PI_ObjectToWorld * influence; 
                ")
            },
            ESkinBonesPerVertex::Two =>  {
                String::from("
                mat4 influence;
                influence = readMatrixFromTex(_boneTex, sampler_boneTex, matricesIndices[0], bondTexSize.x, 0., bondTexSize.y)  * matricesWeights[0];
                influence += readMatrixFromTex(_boneTex, sampler_boneTex, matricesIndices[1], bondTexSize.x, 0., bondTexSize.y) * matricesWeights[1];
                PI_ObjectToWorld = PI_ObjectToWorld * influence; 
                ")
            },
            ESkinBonesPerVertex::Three =>  {
                String::from("
                mat4 influence;
                influence = readMatrixFromTex(_boneTex, sampler_boneTex, matricesIndices[0], bondTexSize.x, 0., bondTexSize.y)  * matricesWeights[0];
                influence += readMatrixFromTex(_boneTex, sampler_boneTex, matricesIndices[1], bondTexSize.x, 0., bondTexSize.y) * matricesWeights[1];
                influence += readMatrixFromTex(_boneTex, sampler_boneTex, matricesIndices[2], bondTexSize.x, 0., bondTexSize.y) * matricesWeights[2];
                PI_ObjectToWorld = PI_ObjectToWorld * influence; 
                ")
            },
            ESkinBonesPerVertex::Four => {
                String::from("
                mat4 influence;
                influence = readMatrixFromTex(_boneTex, sampler_boneTex, matricesIndices[0], bondTexSize.x, 0., bondTexSize.y)  * matricesWeights[0];
                influence += readMatrixFromTex(_boneTex, sampler_boneTex, matricesIndices[1], bondTexSize.x, 0., bondTexSize.y) * matricesWeights[1];
                influence += readMatrixFromTex(_boneTex, sampler_boneTex, matricesIndices[2], bondTexSize.x, 0., bondTexSize.y) * matricesWeights[2];
                influence += readMatrixFromTex(_boneTex, sampler_boneTex, matricesIndices[3], bondTexSize.x, 0., bondTexSize.y) * matricesWeights[3];
                PI_ObjectToWorld = PI_ObjectToWorld * influence; 
                ")
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ESkinCode {
    None,
    RowTexture(ESkinBonesPerVertex),
    FramesTextureInstance(ESkinBonesPerVertex),
}
impl ESkinCode {
    pub fn define_code(&self) -> String {
        match self {
            ESkinCode::None => String::from(""),
            ESkinCode::RowTexture(temp) => temp.define_code(),
            ESkinCode::FramesTextureInstance(temp) => temp.define_code(),
        }
    }
    pub fn running_code(&self) -> String {
        match self {
            ESkinCode::None => String::from(""),
            ESkinCode::RowTexture(temp) => temp.running_code(),
            ESkinCode::FramesTextureInstance(temp) => temp.running_code(),
        }
    }
}