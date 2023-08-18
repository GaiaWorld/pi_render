use std::{sync::Arc, hash::Hash};

use derive_deref_rs::Deref;
use pi_atom::Atom;

use crate::renderer::{buildin_data::{EDefaultTexture, DefaultTexture}, sampler::KeySampler, shader_stage::EShaderStage, texture::EKeyTexture};

use super::{UniformPropertyName, TBindDescToShaderCode, ShaderSetBind, TUnifromShaderProperty};


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UniformSamplerDesc {
    pub slotname: UniformPropertyName,
    pub ty: wgpu::SamplerBindingType,
    pub stage: EShaderStage,
}
impl UniformSamplerDesc {
    pub fn base(texture: &UniformTexture2DDesc) -> Arc<Self> {
        Arc::new(
            Self {
                slotname: UniformPropertyName::from(String::from("sampler") + texture.slotname.as_str()),
                ty: wgpu::SamplerBindingType::Filtering,
                stage: texture.stage
            }
        )
    }
    fn _ty_code(&self) -> String {
        match self.ty {
            wgpu::SamplerBindingType::Filtering => String::from(" sampler "),
            wgpu::SamplerBindingType::NonFiltering => String::from(" sampler "),
            wgpu::SamplerBindingType::Comparison => String::from(" sampler_comparison "),
        }
    }
    fn _code(&self, set: u32, bind: u32) -> String {

        // layout(set = 2, binding = 0) uniform texture2D _MainTex;
        let mut result = ShaderSetBind::code_set_bind_head(set, bind);
        result += self._ty_code().as_str();
        result += self.slotname.as_str();
        result += ";\r\n";

        result
    }
}
impl TBindDescToShaderCode for UniformSamplerDesc {
    fn vs_code(&self, set: u32, bind: u32) -> String {
        if self.stage.mode() & wgpu::ShaderStages::VERTEX == wgpu::ShaderStages::VERTEX {
            self._code(set, bind)
        } else {
            String::from("")
        }
    }

    fn fs_code(&self, set: u32, bind: u32) -> String {
        if self.stage.mode() & wgpu::ShaderStages::FRAGMENT == wgpu::ShaderStages::FRAGMENT {
            self._code(set, bind)
        } else {
            String::from("")
        }
    }
}

#[derive(Debug, Clone, Hash)]
pub struct UniformTexture2DDesc {
    pub slotname: UniformPropertyName,
    pub tex_sampler_type: wgpu::TextureSampleType,
    pub multisampled: bool,
    pub stage: EShaderStage,
    pub initial: EDefaultTexture,
}
impl UniformTexture2DDesc {
    pub fn new(
        slotname: UniformPropertyName,
        tex_sampler_type: wgpu::TextureSampleType,
        multisampled: bool,
        stage: EShaderStage,
        initial: EDefaultTexture,
    ) -> Self {
        Self {
            slotname,
            tex_sampler_type,
            multisampled,
            stage,
            initial
        }
    }
    pub fn new2d(
        slotname: UniformPropertyName,
        stage: EShaderStage,
    ) -> Arc<Self> {
        Arc::new(
            Self {
                slotname,
                tex_sampler_type: wgpu::TextureSampleType::Float { filterable: true },
                multisampled: false,
                stage,
                initial: EDefaultTexture::White,
            }
        )
    }
    pub fn size(&self) -> usize {
        self.slotname.as_bytes().len() + 1 + 1 + 1 + 1
    }
    pub fn sampler_type(&self) -> wgpu::SamplerBindingType {
        match self.tex_sampler_type {
            wgpu::TextureSampleType::Float { filterable } => if filterable { wgpu::SamplerBindingType::Filtering } else { wgpu::SamplerBindingType::NonFiltering } ,
            wgpu::TextureSampleType::Depth => wgpu::SamplerBindingType::Filtering,
            wgpu::TextureSampleType::Sint => wgpu::SamplerBindingType::NonFiltering,
            wgpu::TextureSampleType::Uint => wgpu::SamplerBindingType::NonFiltering,
        }
    }
    fn _ty_code(&self) -> String {
        match self.tex_sampler_type {
            wgpu::TextureSampleType::Float { .. } => String::from(" texture2D "),
            wgpu::TextureSampleType::Depth => String::from(" texture2DShadow "),
            wgpu::TextureSampleType::Sint => String::from(" itexture2D "),
            wgpu::TextureSampleType::Uint => String::from(" utexture2D "),
        }
    }
    fn _code(&self, set: u32, bind: u32) -> String {

        // layout(set = 2, binding = 0) uniform texture2D _MainTex;
        let mut result = ShaderSetBind::code_set_bind_head(set, bind);
        result += self._ty_code().as_str();
        result += self.slotname.as_str();
        result += ";\r\n";

        result
    }
}
impl TBindDescToShaderCode for UniformTexture2DDesc {
    fn vs_code(&self, set: u32, bind: u32) -> String {
        if self.stage.mode() & wgpu::ShaderStages::VERTEX == wgpu::ShaderStages::VERTEX {
            self._code(set, bind)
        } else {
            String::from("")
        }
    }

    fn fs_code(&self, set: u32, bind: u32) -> String {
        if self.stage.mode() & wgpu::ShaderStages::FRAGMENT == wgpu::ShaderStages::FRAGMENT {
            self._code(set, bind)
        } else {
            String::from("")
        }
    }
}
impl TUnifromShaderProperty for UniformTexture2DDesc {
    fn tag(&self) -> &UniformPropertyName {
        &self.slotname
    }
}
impl PartialEq for UniformTexture2DDesc {
    fn eq(&self, other: &Self) -> bool {
        self.tag().eq(other.tag())
    }
}
impl Eq for UniformTexture2DDesc {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialOrd for UniformTexture2DDesc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tag().partial_cmp(other.tag())
    }
}
impl Ord for UniformTexture2DDesc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// * 材质的纹理设置参数
#[derive(Clone, Debug, Hash)]
pub struct UniformTextureWithSamplerParam {
    pub slotname: UniformPropertyName,
    pub filter: bool,
    pub sample: KeySampler,
    pub url: EKeyTexture,
}
impl PartialEq for UniformTextureWithSamplerParam {
    fn eq(&self, other: &Self) -> bool {
        self.slotname.eq(&other.slotname)
    }
}
impl Eq for UniformTextureWithSamplerParam {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialOrd for UniformTextureWithSamplerParam {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.slotname.partial_cmp(&other.slotname)
    }
}
impl Ord for UniformTextureWithSamplerParam {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// * 根据 shader 描述 & 设置的效果纹理参数 构建的纹理使用信息
/// * 数据放在渲染物体上
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct EffectUniformTextureWithSamplerUseinfo(pub Vec<(Arc<UniformTextureWithSamplerParam>, Arc<UniformTexture2DDesc>, Arc<UniformSamplerDesc>)>);

/// * 从 shader 描述生成的 纹理描述数组,
/// * 能通过 纹理属性名称 获取 纹理槽位序号
/// * 能通过 纹理的使用信息 生成 纹理的Uniform描述数组(数组序号对应纹理槽位序号)
/// * 如果某个槽位没有设置 则 根据 shader 描述中对应声明使用默认纹理设置
#[derive(Debug, Clone, Deref)]
pub struct EffectUniformTexture2DDescs(pub Vec<Arc<UniformTexture2DDesc>>);
impl From<Vec<Arc<UniformTexture2DDesc>>> for EffectUniformTexture2DDescs {
    fn from(mut value: Vec<Arc<UniformTexture2DDesc>>) -> Self {
        value.sort_by(|a, b| { a.slotname.cmp(&b.slotname) });

        Self (value)
    }
}
impl EffectUniformTexture2DDescs {
    /// * 根据用户的纹理设置数组, 填补未设置的槽, 以补全所有需要的纹理设置
    /// * 允许用户不设置纹理,自动使用默认纹理
    pub fn use_info(&self, mut param: Vec<Arc<UniformTextureWithSamplerParam>>) -> EffectUniformTextureWithSamplerUseinfo {
        param.sort_by(|a, b| { a.slotname.cmp(&b.slotname) });

        let mut result = EffectUniformTextureWithSamplerUseinfo::default();
        // 某个槽位没有设置 则 根据 shader 描述中对应声明使用默认纹理设置
        self.0.iter().for_each(|item| {
            let slotname = &item.slotname;
            let useinfo = match param.binary_search_by(|probe| probe.slotname.cmp(slotname)) {
                Ok(index) => {
                    let param = param.get(index).unwrap();
                    let sampler = Arc::new(
                        UniformSamplerDesc {
                            slotname: UniformPropertyName::from(String::from("sampler") + slotname.as_str()),
                            ty: if param.filter { wgpu::SamplerBindingType::Filtering } else { wgpu::SamplerBindingType::NonFiltering },
                            stage: item.stage,
                        }
                    );
                    (param.clone(), item.clone(), sampler)
                },
                Err(_) => {
                    let param = UniformTextureWithSamplerParam {
                        slotname: slotname.clone(),
                        filter: true,
                        sample: KeySampler::default(),
                        url: EKeyTexture::Tex(Atom::from(DefaultTexture::path(item.initial, wgpu::TextureDimension::D2))),
                    };
                    (Arc::new(param), item.clone(), UniformSamplerDesc::base(item))
                },
            };

            result.0.push(useinfo);
        });

        result
    }
}
