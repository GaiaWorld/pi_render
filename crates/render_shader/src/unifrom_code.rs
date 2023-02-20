use std::{sync::Arc, hash::Hash};

use pi_atom::Atom;
use render_core::rhi::shader::{BindingExpandDesc};

use crate::{shader_data_kind::AsShaderDataKind, set_bind::ShaderSetBind, texture_sampler_code::SamplerDesc, buildin_data::{EDefaultTexture, DefaultTexture}};

pub enum ErrorUniformSlot {
    NotFoundProperty
}

pub enum UniformValueKind {
    Mat4,
    Mat2,
    Vec4,
    Vec2,
    Float,
    Int,
    Uint,
    TextureD1,
    TextureD2,
    TextureD3,
}

impl AsShaderDataKind for UniformValueKind {
    fn code_kind(&self) -> String {
        match self {
            UniformValueKind::Mat4              => String::from("mat4"),
            UniformValueKind::Mat2              => String::from("mat2"),
            UniformValueKind::Vec4              => String::from("vec4"),
            UniformValueKind::Vec2              => String::from("vec2"),
            UniformValueKind::Float             => String::from("float"),
            UniformValueKind::Int               => String::from("int"),
            UniformValueKind::Uint              => String::from("uint"),
            UniformValueKind::TextureD1         => String::from("texture2D"),
            UniformValueKind::TextureD2         => String::from("texture2D"),
            UniformValueKind::TextureD3         => String::from("textureCube"),
        }
    }
}

pub trait TUnifromShaderProperty {
    fn tag(&self) -> &UniformPropertyName;
}

impl TUnifromShaderProperty for BindingExpandDesc {
    fn tag(&self) -> &UniformPropertyName {
        &self.name
    }
}

pub type UniformPropertyName = Atom;

#[derive(Debug, Clone)]
pub struct UniformPropertyMat4(pub UniformPropertyName, pub [f32;16]);
impl TUnifromShaderProperty for UniformPropertyMat4 {
    fn tag(&self) -> &UniformPropertyName {
        &self.0
    }
}
impl Hash for UniformPropertyMat4 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for UniformPropertyMat4 {
    fn eq(&self, other: &Self) -> bool {
        self.tag().eq(other.tag())
    }
}
impl Eq for UniformPropertyMat4 {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialOrd for UniformPropertyMat4 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tag().partial_cmp(other.tag())
    }
}
impl Ord for UniformPropertyMat4 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct UniformPropertyMat2(pub UniformPropertyName, pub [f32;4]);
impl TUnifromShaderProperty for UniformPropertyMat2 {
    fn tag(&self) -> &UniformPropertyName {
        &self.0
    }
}
impl Hash for UniformPropertyMat2 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for UniformPropertyMat2 {
    fn eq(&self, other: &Self) -> bool {
        self.tag().eq(other.tag())
    }
}
impl Eq for UniformPropertyMat2 {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialOrd for UniformPropertyMat2 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tag().partial_cmp(other.tag())
    }
}
impl Ord for UniformPropertyMat2 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct UniformPropertyVec4(pub UniformPropertyName, pub [f32;4]);
impl TUnifromShaderProperty for UniformPropertyVec4 {
    fn tag(&self) -> &UniformPropertyName {
        &self.0
    }
}
impl Hash for UniformPropertyVec4 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for UniformPropertyVec4 {
    fn eq(&self, other: &Self) -> bool {
        self.tag().eq(other.tag())
    }
}
impl Eq for UniformPropertyVec4 {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialOrd for UniformPropertyVec4 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tag().partial_cmp(other.tag())
    }
}
impl Ord for UniformPropertyVec4 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct UniformPropertyVec2(pub UniformPropertyName, pub [f32;2]);
impl TUnifromShaderProperty for UniformPropertyVec2 {
    fn tag(&self) -> &UniformPropertyName {
        &self.0
    }
}
impl Hash for UniformPropertyVec2 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for UniformPropertyVec2 {
    fn eq(&self, other: &Self) -> bool {
        self.tag().eq(other.tag())
    }
}
impl Eq for UniformPropertyVec2 {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialOrd for UniformPropertyVec2 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tag().partial_cmp(other.tag())
    }
}
impl Ord for UniformPropertyVec2 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct UniformPropertyFloat(pub UniformPropertyName, pub f32);
impl TUnifromShaderProperty for UniformPropertyFloat {
    fn tag(&self) -> &UniformPropertyName {
        &self.0
    }
}
impl Hash for UniformPropertyFloat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for UniformPropertyFloat {
    fn eq(&self, other: &Self) -> bool {
        self.tag().eq(other.tag())
    }
}
impl Eq for UniformPropertyFloat {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialOrd for UniformPropertyFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tag().partial_cmp(other.tag())
    }
}
impl Ord for UniformPropertyFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct UniformPropertyInt(pub UniformPropertyName, pub i32);
impl TUnifromShaderProperty for UniformPropertyInt {
    fn tag(&self) -> &UniformPropertyName {
        &self.0
    }
}
impl Hash for UniformPropertyInt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for UniformPropertyInt {
    fn eq(&self, other: &Self) -> bool {
        self.tag().eq(other.tag())
    }
}
impl Eq for UniformPropertyInt {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialOrd for UniformPropertyInt {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tag().partial_cmp(other.tag())
    }
}
impl Ord for UniformPropertyInt {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct UniformPropertyUint(pub UniformPropertyName, pub u32);
impl TUnifromShaderProperty for UniformPropertyUint {
    fn tag(&self) -> &UniformPropertyName {
        &self.0
    }
}
impl Hash for UniformPropertyUint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for UniformPropertyUint {
    fn eq(&self, other: &Self) -> bool {
        self.tag().eq(other.tag())
    }
}
impl Eq for UniformPropertyUint {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialOrd for UniformPropertyUint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tag().partial_cmp(other.tag())
    }
}
impl Ord for UniformPropertyUint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub trait TBindDescToShaderCode {
    fn vs_code(&self, set: u32, bind: u32) -> String;
    fn fs_code(&self, set: u32, bind: u32) -> String;
}

#[derive(Debug, Clone, Hash)]
pub struct UniformTextureDesc {
    pub slotname: UniformPropertyName,
    pub tex_sampler_type: wgpu::TextureSampleType,
    pub dimension: wgpu::TextureViewDimension,
    pub multisampled: bool,
    pub stage: wgpu::ShaderStages,
    pub initial: EDefaultTexture,
}
impl UniformTextureDesc {
    pub fn new(
        slotname: UniformPropertyName,
        tex_sampler_type: wgpu::TextureSampleType,
        dimension: wgpu::TextureViewDimension,
        multisampled: bool,
        stage: wgpu::ShaderStages,
        initial: EDefaultTexture,
    ) -> Self {
        Self {
            slotname,
            tex_sampler_type,
            dimension,
            multisampled,
            stage,
            initial
        }
    }
    pub fn new2d(
        slotname: UniformPropertyName,
        stage: wgpu::ShaderStages,
    ) -> Arc<Self> {
        Arc::new(
            Self {
                slotname,
                tex_sampler_type: wgpu::TextureSampleType::Float { filterable: true },
                dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
                stage,
                initial: EDefaultTexture::White,
            }
        )
    }
    pub fn size(&self) -> usize {
        self.slotname.as_bytes().len() + 1 + 1 + 1 + 1
    }
    fn _ty_code(&self) -> String {
        match self.tex_sampler_type {
            wgpu::TextureSampleType::Float { filterable } => match self.dimension {
                wgpu::TextureViewDimension::D1          => String::from(" texture1D "),
                wgpu::TextureViewDimension::D2          => String::from(" texture2D "),
                wgpu::TextureViewDimension::D2Array     => String::from(" texture2DArray "),
                wgpu::TextureViewDimension::Cube        => String::from(" textureCube "),
                wgpu::TextureViewDimension::CubeArray   => String::from(" textureCubeArray "),
                wgpu::TextureViewDimension::D3          => String::from(" texture3D "),
            },
            wgpu::TextureSampleType::Depth => match self.dimension {
                wgpu::TextureViewDimension::D1          => String::from(" texture1DShadow "),
                wgpu::TextureViewDimension::D2          => String::from(" texture2DShadow "),
                wgpu::TextureViewDimension::D2Array     => String::from(" texture2DShadowArray "),
                wgpu::TextureViewDimension::Cube        => String::from(" textureCubeShadow "),
                wgpu::TextureViewDimension::CubeArray   => String::from(" textureCubeShadowArray "),
                wgpu::TextureViewDimension::D3          => String::from(" texture3DShadow "),
            },
            wgpu::TextureSampleType::Sint => match self.dimension {
                wgpu::TextureViewDimension::D1          => String::from(" itexture1D "),
                wgpu::TextureViewDimension::D2          => String::from(" itexture2D "),
                wgpu::TextureViewDimension::D2Array     => String::from(" itexture2DArray "),
                wgpu::TextureViewDimension::Cube        => String::from(" itextureCube "),
                wgpu::TextureViewDimension::CubeArray   => String::from(" itextureCubeArray "),
                wgpu::TextureViewDimension::D3          => String::from(" itexture3D "),
            },
            wgpu::TextureSampleType::Uint => match self.dimension {
                wgpu::TextureViewDimension::D1          => String::from(" utexture1D "),
                wgpu::TextureViewDimension::D2          => String::from(" utexture2D "),
                wgpu::TextureViewDimension::D2Array     => String::from(" utexture2DArray "),
                wgpu::TextureViewDimension::Cube        => String::from(" utextureCube "),
                wgpu::TextureViewDimension::CubeArray   => String::from(" utextureCubeArray "),
                wgpu::TextureViewDimension::D3          => String::from(" utexture3D "),
            },
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
impl TBindDescToShaderCode for UniformTextureDesc {
    fn vs_code(&self, set: u32, bind: u32) -> String {
        if self.stage & wgpu::ShaderStages::VERTEX == wgpu::ShaderStages::VERTEX {
            self._code(set, bind)
        } else {
            String::from("")
        }
    }

    fn fs_code(&self, set: u32, bind: u32) -> String {
        if self.stage & wgpu::ShaderStages::FRAGMENT == wgpu::ShaderStages::FRAGMENT {
            self._code(set, bind)
        } else {
            String::from("")
        }
    }
}
impl TUnifromShaderProperty for UniformTextureDesc {
    fn tag(&self) -> &UniformPropertyName {
        &self.slotname
    }
}
impl PartialEq for UniformTextureDesc {
    fn eq(&self, other: &Self) -> bool {
        self.tag().eq(other.tag())
    }
}
impl Eq for UniformTextureDesc {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialOrd for UniformTextureDesc {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.tag().partial_cmp(other.tag())
    }
}
impl Ord for UniformTextureDesc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// * 从 shader 描述生成的 纹理描述数组,
/// * 能通过 纹理属性名称 获取 纹理槽位序号
/// * 能通过 纹理的使用信息 生成 纹理的Uniform描述数组(数组序号对应纹理槽位序号)
/// * 如果某个槽位没有设置 则 根据 shader 描述中对应声明使用默认纹理设置
#[derive(Debug, Clone)]
pub struct EffectUniformTextureDescs {
    pub descs: Vec<Arc<UniformTextureDesc>>,
}
impl From<Vec<Arc<UniformTextureDesc>>> for EffectUniformTextureDescs {
    fn from(mut value: Vec<Arc<UniformTextureDesc>>) -> Self {
        value.sort_by(|a, b| { a.slotname.cmp(&b.slotname) });

        Self { descs: value }
    }
}
impl EffectUniformTextureDescs {
    pub fn use_info(&self, mut param: Vec<Arc<UniformTextureWithSamplerParam>>) -> EffectUniformTextureWithSamplerUseinfo {
        param.sort_by(|a, b| { a.slotname.cmp(&b.slotname) });

        let mut result = EffectUniformTextureWithSamplerUseinfo::default();
        // 某个槽位没有设置 则 根据 shader 描述中对应声明使用默认纹理设置
        self.descs.iter().for_each(|item| {
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
                        sample: SamplerDesc::default(),
                        url: Atom::from(DefaultTexture::path(item.initial, wgpu::TextureDimension::D2)),
                    };
                    (Arc::new(param), item.clone(), UniformSamplerDesc::base(item))
                },
            };

            result.0.push(useinfo);
        });

        result
    }
}

// pub struct EffectUniformValues {
//     pub desc: MaterialValueBindDesc,
//     pub slotnames: Vec<Atom>,
// }

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UniformSamplerDesc {
    pub slotname: UniformPropertyName,
    pub ty: wgpu::SamplerBindingType,
    pub stage: wgpu::ShaderStages,
}
impl UniformSamplerDesc {
    pub fn base(texture: &UniformTextureDesc) -> Arc<Self> {
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
        if self.stage & wgpu::ShaderStages::VERTEX == wgpu::ShaderStages::VERTEX {
            self._code(set, bind)
        } else {
            String::from("")
        }
    }

    fn fs_code(&self, set: u32, bind: u32) -> String {
        if self.stage & wgpu::ShaderStages::FRAGMENT == wgpu::ShaderStages::FRAGMENT {
            self._code(set, bind)
        } else {
            String::from("")
        }
    }
}

/// * 材质的纹理设置参数
#[derive(Clone, Debug, Hash)]
pub struct UniformTextureWithSamplerParam {
    pub slotname: UniformPropertyName,
    pub filter: bool,
    pub sample: SamplerDesc,
    pub url: Atom,
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
pub struct EffectUniformTextureWithSamplerUseinfo(pub Vec<(Arc<UniformTextureWithSamplerParam>, Arc<UniformTextureDesc>, Arc<UniformSamplerDesc>)>);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MaterialValueBindDesc {
    pub stage: wgpu::ShaderStages,
    pub mat4_list: Vec<UniformPropertyMat4>,
    pub mat2_list: Vec<UniformPropertyMat2>,
    pub vec4_list: Vec<UniformPropertyVec4>,
    pub vec2_list: Vec<UniformPropertyVec2>,
    pub float_list: Vec<UniformPropertyFloat>,
    pub int_list: Vec<UniformPropertyInt>,
    pub uint_list: Vec<UniformPropertyUint>,
}
impl Default for MaterialValueBindDesc {
    fn default() -> Self {
        Self { stage: wgpu::ShaderStages::VERTEX_FRAGMENT, mat4_list: vec![], mat2_list: vec![], vec4_list: vec![], vec2_list: vec![], float_list: vec![], int_list: vec![], uint_list: vec![] }
    }
}
impl MaterialValueBindDesc {
    pub fn none(stage: wgpu::ShaderStages) -> Self {
        Self { stage, mat4_list: vec![], mat2_list: vec![], vec4_list: vec![], vec2_list: vec![], float_list: vec![], int_list: vec![], uint_list: vec![] }
    }
    // pub fn has_value(&self) -> bool {
    //     self.mat4_list.len() > 0
    //     || self.mat4_list.len() > 0
    //     || self.mat2_list.len() > 0
    //     || self.vec4_list.len() > 0
    //     || self.vec2_list.len() > 0
    //     || self.float_list.len() > 0
    //     || self.int_list.len() > 0
    //     || self.uint_list.len() > 0
    // }
    pub fn sort(&mut self) {
        self.mat4_list.sort_by(|a, b| { a.0.cmp(&b.0) });
        self.mat2_list.sort_by(|a, b| { a.0.cmp(&b.0) });
        self.vec4_list.sort_by(|a, b| { a.0.cmp(&b.0) });
        self.vec2_list.sort_by(|a, b| { a.0.cmp(&b.0) });
        self.float_list.sort_by(|a, b| { a.0.cmp(&b.0) });
        self.int_list.sort_by(|a, b| { a.0.cmp(&b.0) });
        self.uint_list.sort_by(|a, b| { a.0.cmp(&b.0) });
    }
    pub fn size(&self) -> usize {
        let mut size = 16; // 基础大小 16 - 避免为 0
        self.mat4_list.iter().for_each(|item| {
            size += item.0.as_bytes().len();
        });
        
        self.mat2_list.iter().for_each(|item| {
            size += item.0.as_bytes().len();
        });
        
        self.vec4_list.iter().for_each(|item| {
            size += item.0.as_bytes().len();
        });
        
        self.vec2_list.iter().for_each(|item| {
            size += item.0.as_bytes().len();
        });
        
        self.float_list.iter().for_each(|item| {
            size += item.0.as_bytes().len();
        });
        
        self.int_list.iter().for_each(|item| {
            size += item.0.as_bytes().len();
        });
        
        self.uint_list.iter().for_each(|item| {
            size += item.0.as_bytes().len();
        });

        size
    }
    pub fn label(&self) -> String {
        let mut result = String::from("");

        self.mat4_list.iter().for_each(|name| {
            result += "#";
            result += name.0.as_str();
        });
        
        self.mat2_list.iter().for_each(|name| {
            result += "#";
            result += name.0.as_str();
        });

        self.vec4_list.iter().for_each(|name| {
            result += "#";
            result += name.0.as_str();
        });

        self.vec2_list.iter().for_each(|name| {
            result += "#";
            result += name.0.as_str();
        });

        self.float_list.iter().for_each(|name| {
            result += "#";
            result += name.0.as_str();
        });

        self.uint_list.iter().for_each(|name| {
            result += "#";
            result += name.0.as_str();
        });

        result
    }
    fn _code(&self, set: u32, index: u32) -> String {
        let mut result = String::from("");

        let mut total_num = 0;

        result += "layout(set = ";
        result += set.to_string().as_str();
        result += ", binding = ";
        result += index.to_string().as_str();
        result += ") uniform MatParam {\r\n";

        self.mat4_list.iter().for_each(|name| {
            result += "mat4 ";
            result += &name.0;
            result += ";\r\n";
        });
        total_num += self.mat4_list.len();
        
        self.mat2_list.iter().for_each(|name| {
            result += "mat2 ";
            result += &name.0;
            result += ";\r\n";
        });
        total_num += self.mat2_list.len();
        
        self.vec4_list.iter().for_each(|name| {
            result += "vec4 ";
            result += &name.0;
            result += ";\r\n";
        });
        total_num += self.vec4_list.len();
        
        self.vec2_list.iter().for_each(|name| {
            result += "vec2 ";
            result += &name.0;
            result += ";\r\n";
        });
        total_num += self.vec2_list.len();
        let fill_vec2_count    = self.vec2_list.len() % 2;
        if fill_vec2_count > 0 {
            result += "vec2 _placeholder_vec2_0;\r\n";
        }
        
        self.float_list.iter().for_each(|name| {
            result += "float ";
            result += &name.0;
            result += ";\r\n";
        });
        total_num += self.float_list.len();
        
        self.int_list.iter().for_each(|name| {
            result += "int ";
            result += &name.0;
            result += ";\r\n";
        });
        total_num += self.int_list.len();
        
        self.uint_list.iter().for_each(|name| {
            result += "uint ";
            result += &name.0;
            result += ";\r\n";
        });
        total_num += self.uint_list.len();
        let fill_int_count    = (self.float_list.len() + self.int_list.len() + self.uint_list.len()) % 4;
        if fill_int_count > 0 {
            for i in fill_int_count..4 {
                result += "uint _placeholder_int_";
                result += &i.to_string();
                result += ";\r\n";
            }
        } else {
            // 4 个 占位u32; 对应 ShaderBindEffectValue 中也有处理
            if total_num == 0 {
                for i in 0..4 {
                    result += "uint _placeholder_int_";
                    result += &i.to_string();
                    result += ";\r\n";
                }
            }
        }

        result += "};\r\n";

        result
    }
}
impl TBindDescToShaderCode for MaterialValueBindDesc {
    fn vs_code(&self, set: u32, bind: u32) -> String {
        if self.stage & wgpu::ShaderStages::VERTEX == wgpu::ShaderStages::VERTEX {
            self._code(set, bind)
        } else {
            String::from("")
        }
    }

    fn fs_code(&self, set: u32, bind: u32) -> String {
        if self.stage & wgpu::ShaderStages::FRAGMENT == wgpu::ShaderStages::FRAGMENT {
            self._code(set, bind)
        } else {
            String::from("")
        }
    }
}

pub fn vec_u8_to_f32_16(val: &Vec<u8>) -> [f32;16] {
    if val.len() >= 64 {
        let mut temp: [u8;64] = [0;64];
        for i in 0..64 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;64], [f32;16]>(temp)
        }
    } else {
        [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.]
    }
}

pub fn vec_u8_to_f32_4(val: &Vec<u8>) -> [f32;4] {
    if val.len() >= 16 {
        let mut temp: [u8;16] = [0;16];
        for i in 0..16 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;16], [f32;4]>(temp)
        }
    } else {
        [1., 0., 0., 0.]
    }
}

pub fn vec_u8_to_f32_2(val: &Vec<u8>) -> [f32;2] {
    if val.len() >= 8 {
        let mut temp: [u8;8] = [0;8];
        for i in 0..8 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;8], [f32;2]>(temp)
        }
    } else {
        [0., 0.]
    }
}

pub fn vec_u8_to_f32(val: &Vec<u8>) -> f32 {
    if val.len() >= 4 {
        let mut temp: [u8;4] = [0;4];
        for i in 0..4 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;4], f32>(temp)
        }
    } else {
        0.
    }
}

pub fn vec_u8_to_i32(val: &Vec<u8>) -> i32 {
    if val.len() >= 4 {
        let mut temp: [u8;4] = [0;4];
        for i in 0..4 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;4], i32>(temp)
        }
    } else {
        0
    }
}

pub fn vec_u8_to_u32(val: &Vec<u8>) -> u32 {
    if val.len() >= 4 {
        let mut temp: [u8;4] = [0;4];
        for i in 0..4 {
            temp[i] = val[i];
        }
        unsafe {
            std::mem::transmute::<[u8;4], u32>(temp)
        }
    } else {
        0
    }
}
