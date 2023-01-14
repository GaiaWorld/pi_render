use std::{mem::{size_of, replace}, ops::{Deref, Range}, fmt::Debug};

use pi_assets::asset::Asset;
use pi_share::Share;
use render_core::rhi::pipeline::VertexBufferLayout;
use render_data_container::{TVertexDataKindKey, TVertexBufferID, vertex_layout_key::{KeyVertexLayouts, KeyVertexLayout}, KeyVertexBuffer};

use crate::{vertex_code::{TVertexShaderCode, TVertexFormatCode}, error::EGeometryError, buildin_var::ShaderVarVertices};

pub const BYTES_VERTEX_DATA_KIND: u8 = 8;
pub const BYTES_VERTEX_FORMAT: u8 = 5;
pub const BYTES_VERTEX_BUFFER_STEP_MODE: u8 = 1;
pub const MAX_VERTEX_ATTRIBUTE_ID: u8 = (KeyVertexLayout::MAX / (BYTES_VERTEX_BUFFER_STEP_MODE + BYTES_VERTEX_DATA_KIND + BYTES_VERTEX_FORMAT) as KeyVertexLayout) as u8;

trait TAsVertexLayoutsKey {
    fn bytes_for_vertex_layouts_key(&self) -> u8;
    fn as_vertex_layouts_key(&self) -> KeyVertexLayout;
}
trait TVertexAttributeSize {
    fn attribute_bytes_size(&self) -> wgpu::BufferAddress ;
}
trait TAsWgpuVertexAtribute {
    fn as_attribute(&self, offset: wgpu::BufferAddress, shader_location: u32) ->wgpu::VertexAttribute;
}

///
/// 预留为支持 64 种顶点数据
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EVertexDataKind {
    Position               ,
    Position2D             ,
    Color4                 ,
    UV                     ,
    Normal                 ,
    Tangent                ,
    MatricesIndices        ,
    MatricesWeights        ,
    MatricesIndicesExtra   ,
    MatricesWeightsExtra   ,
    UV2                    ,
    UV3                    ,
    UV4                    ,
    UV5                    ,
    UV6                    ,
    CustomVec4A            ,
    CustomVec4B            ,
    CustomVec3A            ,
    CustomVec3B            ,
    CustomVec2A            ,
    CustomVec2B            ,
    InsWorldRow1           ,
    InsWorldRow2           ,
    InsWorldRow3           ,
    InsWorldRow4           ,
    InsColor               ,
    InsTillOffset1         ,
    InsTillOffset2         ,
    InsCustomVec4A         ,
    InsCustomVec4B         ,
    InsCustomUVec4A        ,
    InsCustomIVec4B        ,

    MatricesIndices1       ,
    MatricesWeights1       ,

    MatricesIndices2       ,
    MatricesWeights2       ,
    MatricesIndicesExtra2  ,
    MatricesWeightsExtra2  ,

    MatricesIndices3       ,
    MatricesWeights3       ,
    MatricesIndicesExtra3  ,
    MatricesWeightsExtra3  ,
}
impl TAsVertexLayoutsKey for EVertexDataKind {
    fn bytes_for_vertex_layouts_key(&self) -> u8 {
        BYTES_VERTEX_DATA_KIND
    }
    fn as_vertex_layouts_key(&self) -> KeyVertexLayout {
        match self {
            EVertexDataKind::Position               => 00,
            EVertexDataKind::Position2D             => 01,
            EVertexDataKind::Color4                 => 02,
            EVertexDataKind::UV                     => 03,
            EVertexDataKind::Normal                 => 04,
            EVertexDataKind::Tangent                => 05,
            EVertexDataKind::MatricesIndices        => 06,
            EVertexDataKind::MatricesWeights        => 07,
            EVertexDataKind::MatricesIndicesExtra   => 08,
            EVertexDataKind::MatricesWeightsExtra   => 09,
            EVertexDataKind::UV2                    => 10,
            EVertexDataKind::UV3                    => 11,
            EVertexDataKind::UV4                    => 12,
            EVertexDataKind::UV5                    => 13,
            EVertexDataKind::UV6                    => 14,
            EVertexDataKind::CustomVec4A            => 15,
            EVertexDataKind::CustomVec4B            => 16,
            EVertexDataKind::CustomVec3A            => 17,
            EVertexDataKind::CustomVec3B            => 18,
            EVertexDataKind::CustomVec2A            => 19,
            EVertexDataKind::CustomVec2B            => 20,
            EVertexDataKind::InsWorldRow1           => 21,
            EVertexDataKind::InsWorldRow2           => 22,
            EVertexDataKind::InsWorldRow3           => 23,
            EVertexDataKind::InsWorldRow4           => 24,
            EVertexDataKind::InsColor               => 25,
            EVertexDataKind::InsTillOffset1         => 26,
            EVertexDataKind::InsTillOffset2         => 27,
            EVertexDataKind::InsCustomVec4A         => 28,
            EVertexDataKind::InsCustomVec4B         => 29,
            EVertexDataKind::InsCustomUVec4A        => 30,
            EVertexDataKind::InsCustomIVec4B        => 31,
            EVertexDataKind::MatricesIndices1       => 32,
            EVertexDataKind::MatricesWeights1       => 33,
            EVertexDataKind::MatricesIndices2       => 34,
            EVertexDataKind::MatricesWeights2       => 35,
            EVertexDataKind::MatricesIndicesExtra2  => 36,
            EVertexDataKind::MatricesWeightsExtra2  => 37,
            EVertexDataKind::MatricesIndices3       => 38,
            EVertexDataKind::MatricesWeights3       => 39,
            EVertexDataKind::MatricesIndicesExtra3  => 40,
            EVertexDataKind::MatricesWeightsExtra3  => 41,
        }
    }
}
impl EVertexDataKind {
    pub fn vs_code(&self) -> &str {
        match self {
            EVertexDataKind::Position               => ShaderVarVertices::POSITION                  ,
            EVertexDataKind::Position2D             => ShaderVarVertices::POSITION2D                ,
            EVertexDataKind::Color4                 => ShaderVarVertices::COLOR4                    ,
            EVertexDataKind::UV                     => ShaderVarVertices::UV                        ,
            EVertexDataKind::Normal                 => ShaderVarVertices::NORMAL                    ,
            EVertexDataKind::Tangent                => ShaderVarVertices::TANGENT                   ,
            EVertexDataKind::MatricesIndices        => ShaderVarVertices::MATRICES_INDICES          ,
            EVertexDataKind::MatricesWeights        => ShaderVarVertices::MATRICES_WEIGHTS          ,
            EVertexDataKind::MatricesIndicesExtra   => ShaderVarVertices::MATRICES_INDICES_EXTRA    ,
            EVertexDataKind::MatricesWeightsExtra   => ShaderVarVertices::MATRICES_WEIGHTS_EXTRA    ,
            EVertexDataKind::UV2                    => ShaderVarVertices::UV2                       ,
            EVertexDataKind::UV3                    => ShaderVarVertices::UV3                       ,
            EVertexDataKind::UV4                    => ShaderVarVertices::UV4                       ,
            EVertexDataKind::UV5                    => ShaderVarVertices::UV5                       ,
            EVertexDataKind::UV6                    => ShaderVarVertices::UV6                       ,
            EVertexDataKind::CustomVec4A            => ShaderVarVertices::CUSTOM_VEC4_A             ,
            EVertexDataKind::CustomVec4B            => ShaderVarVertices::CUSTOM_VEC4_B             ,
            EVertexDataKind::CustomVec3A            => ShaderVarVertices::CUSTOM_VEC3_A             ,
            EVertexDataKind::CustomVec3B            => ShaderVarVertices::CUSTOM_VEC3_B             ,
            EVertexDataKind::CustomVec2A            => ShaderVarVertices::CUSTOM_VEC2_A             ,
            EVertexDataKind::CustomVec2B            => ShaderVarVertices::CUSTOM_VEC2_B             ,
            EVertexDataKind::InsWorldRow1           => ShaderVarVertices::INS_WORLD_ROW1            ,
            EVertexDataKind::InsWorldRow2           => ShaderVarVertices::INS_WORLD_ROW2            ,
            EVertexDataKind::InsWorldRow3           => ShaderVarVertices::INS_WORLD_ROW3            ,
            EVertexDataKind::InsWorldRow4           => ShaderVarVertices::INS_WORLD_ROW4            ,
            EVertexDataKind::InsColor               => ShaderVarVertices::INS_COLOR                 ,
            EVertexDataKind::InsTillOffset1         => ShaderVarVertices::INS_TILL_OFFSET1          ,
            EVertexDataKind::InsTillOffset2         => ShaderVarVertices::INS_TILL_OFFSET2          ,
            EVertexDataKind::InsCustomVec4A         => ShaderVarVertices::INS_CUSTOM_VEC4_A         ,
            EVertexDataKind::InsCustomVec4B         => ShaderVarVertices::INS_CUSTOM_VEC4_B         ,
            EVertexDataKind::InsCustomUVec4A        => ShaderVarVertices::INS_CUSTOM_UVEC4_A        ,
            EVertexDataKind::InsCustomIVec4B        => ShaderVarVertices::INS_CUSTOM_IVEC4_B        ,
            EVertexDataKind::MatricesIndices1       => ShaderVarVertices::MATRICES_INDICES1         ,
            EVertexDataKind::MatricesWeights1       => ShaderVarVertices::MATRICES_WEIGHTS1         ,
            EVertexDataKind::MatricesIndices2       => ShaderVarVertices::MATRICES_INDICES2         ,
            EVertexDataKind::MatricesWeights2       => ShaderVarVertices::MATRICES_WEIGHTS2         ,
            EVertexDataKind::MatricesIndicesExtra2  => ShaderVarVertices::MATRICES_INDICES_EXTRA2   ,
            EVertexDataKind::MatricesWeightsExtra2  => ShaderVarVertices::MATRICES_WEIGHTS_EXTRA2   ,
            EVertexDataKind::MatricesIndices3       => ShaderVarVertices::MATRICES_INDICES3         ,
            EVertexDataKind::MatricesWeights3       => ShaderVarVertices::MATRICES_WEIGHTS3         ,
            EVertexDataKind::MatricesIndicesExtra3  => ShaderVarVertices::MATRICES_INDICES_EXTRA3   ,
            EVertexDataKind::MatricesWeightsExtra3  => ShaderVarVertices::MATRICES_WEIGHTS_EXTRA3   ,
        }
    }
    pub fn kind(&self) -> &str {
        match self {
            EVertexDataKind::Position               => "vec3",
            EVertexDataKind::Position2D             => "vec2",
            EVertexDataKind::Color4                 => "vec4",
            EVertexDataKind::UV                     => "vec2",
            EVertexDataKind::Normal                 => "vec3",
            EVertexDataKind::Tangent                => "vec4",
            EVertexDataKind::MatricesIndices        => "uvec4",
            EVertexDataKind::MatricesWeights        => "vec4",
            EVertexDataKind::MatricesIndicesExtra   => "uvec4",
            EVertexDataKind::MatricesWeightsExtra   => "vec4",
            EVertexDataKind::UV2                    => "vec2",
            EVertexDataKind::UV3                    => "vec2",
            EVertexDataKind::UV4                    => "vec2",
            EVertexDataKind::UV5                    => "vec2",
            EVertexDataKind::UV6                    => "vec2",
            EVertexDataKind::CustomVec4A            => "vec4",
            EVertexDataKind::CustomVec4B            => "vec4",
            EVertexDataKind::CustomVec3A            => "vec3",
            EVertexDataKind::CustomVec3B            => "vec3",
            EVertexDataKind::CustomVec2A            => "vec2",
            EVertexDataKind::CustomVec2B            => "vec2",
            EVertexDataKind::InsWorldRow1           => "vec4",
            EVertexDataKind::InsWorldRow2           => "vec4",
            EVertexDataKind::InsWorldRow3           => "vec4",
            EVertexDataKind::InsWorldRow4           => "vec4",
            EVertexDataKind::InsColor               => "vec4",
            EVertexDataKind::InsTillOffset1         => "vec4",
            EVertexDataKind::InsTillOffset2         => "vec4",
            EVertexDataKind::InsCustomVec4A         => "vec4",
            EVertexDataKind::InsCustomVec4B         => "vec4",
            EVertexDataKind::InsCustomUVec4A        => "uvec4",
            EVertexDataKind::InsCustomIVec4B        => "ivec4",
            EVertexDataKind::MatricesIndices1       => "uint",
            EVertexDataKind::MatricesWeights1       => "float",
            EVertexDataKind::MatricesIndices2       => "uvec2",
            EVertexDataKind::MatricesWeights2       => "vec2",
            EVertexDataKind::MatricesIndicesExtra2  => "uvec2",
            EVertexDataKind::MatricesWeightsExtra2  => "vec2",
            EVertexDataKind::MatricesIndices3       => "uvec3",
            EVertexDataKind::MatricesWeights3       => "vec3",
            EVertexDataKind::MatricesIndicesExtra3  => "uvec3",
            EVertexDataKind::MatricesWeightsExtra3  => "vec3",
        }
    }
}

pub type VertexSlot = u8;
pub type VertexAttributeOffset = u8;

impl TVertexAttributeSize for wgpu::VertexFormat {
    fn attribute_bytes_size(&self) -> wgpu::BufferAddress {
        match self {
            wgpu::VertexFormat::Uint8x2     => 1 * 2,
            wgpu::VertexFormat::Uint8x4     => 1 * 4,
            wgpu::VertexFormat::Sint8x2     => 1 * 2,
            wgpu::VertexFormat::Sint8x4     => 1 * 4,
            wgpu::VertexFormat::Unorm8x2    => 1 * 2,
            wgpu::VertexFormat::Unorm8x4    => 1 * 4,
            wgpu::VertexFormat::Snorm8x2    => 1 * 2,
            wgpu::VertexFormat::Snorm8x4    => 1 * 4,
            wgpu::VertexFormat::Uint16x2    => 2 * 2,
            wgpu::VertexFormat::Uint16x4    => 2 * 4,
            wgpu::VertexFormat::Sint16x2    => 2 * 2,
            wgpu::VertexFormat::Sint16x4    => 2 * 4,
            wgpu::VertexFormat::Unorm16x2   => 2 * 2,
            wgpu::VertexFormat::Unorm16x4   => 2 * 4,
            wgpu::VertexFormat::Snorm16x2   => 2 * 2,
            wgpu::VertexFormat::Snorm16x4   => 2 * 4,
            wgpu::VertexFormat::Float16x2   => 2 * 2,
            wgpu::VertexFormat::Float16x4   => 2 * 4,
            wgpu::VertexFormat::Float32     => 4 * 1,
            wgpu::VertexFormat::Float32x2   => 4 * 2,
            wgpu::VertexFormat::Float32x3   => 4 * 3,
            wgpu::VertexFormat::Float32x4   => 4 * 4,
            wgpu::VertexFormat::Uint32      => 4 * 1,
            wgpu::VertexFormat::Uint32x2    => 4 * 2,
            wgpu::VertexFormat::Uint32x3    => 4 * 3,
            wgpu::VertexFormat::Uint32x4    => 4 * 4,
            wgpu::VertexFormat::Sint32      => 4 * 1,
            wgpu::VertexFormat::Sint32x2    => 4 * 2,
            wgpu::VertexFormat::Sint32x3    => 4 * 3,
            wgpu::VertexFormat::Sint32x4    => 4 * 4,
            wgpu::VertexFormat::Float64     => 8 * 1,
            wgpu::VertexFormat::Float64x2   => 8 * 2,
            wgpu::VertexFormat::Float64x3   => 8 * 3,
            wgpu::VertexFormat::Float64x4   => 8 * 4,
        }
    }
}
impl TAsVertexLayoutsKey for wgpu::VertexFormat {
    fn bytes_for_vertex_layouts_key(&self) -> u8 {
        BYTES_VERTEX_FORMAT
    }
    fn as_vertex_layouts_key(&self) -> KeyVertexLayout {
        match self {
            wgpu::VertexFormat::Uint8x2     => 00,
            wgpu::VertexFormat::Uint8x4     => 01,
            wgpu::VertexFormat::Sint8x2     => 02,
            wgpu::VertexFormat::Sint8x4     => 03,
            wgpu::VertexFormat::Unorm8x2    => 04,
            wgpu::VertexFormat::Unorm8x4    => 05,
            wgpu::VertexFormat::Snorm8x2    => 06,
            wgpu::VertexFormat::Snorm8x4    => 07,
            wgpu::VertexFormat::Uint16x2    => 08,
            wgpu::VertexFormat::Uint16x4    => 09,
            wgpu::VertexFormat::Sint16x2    => 10,
            wgpu::VertexFormat::Sint16x4    => 11,
            wgpu::VertexFormat::Unorm16x2   => 12,
            wgpu::VertexFormat::Unorm16x4   => 13,
            wgpu::VertexFormat::Snorm16x2   => 14,
            wgpu::VertexFormat::Snorm16x4   => 15,
            wgpu::VertexFormat::Float16x2   => 16,
            wgpu::VertexFormat::Float16x4   => 17,
            wgpu::VertexFormat::Float32     => 18,
            wgpu::VertexFormat::Float32x2   => 19,
            wgpu::VertexFormat::Float32x3   => 20,
            wgpu::VertexFormat::Float32x4   => 21,
            wgpu::VertexFormat::Uint32      => 22,
            wgpu::VertexFormat::Uint32x2    => 23,
            wgpu::VertexFormat::Uint32x3    => 24,
            wgpu::VertexFormat::Uint32x4    => 25,
            wgpu::VertexFormat::Sint32      => 26,
            wgpu::VertexFormat::Sint32x2    => 27,
            wgpu::VertexFormat::Sint32x3    => 28,
            wgpu::VertexFormat::Sint32x4    => 29,
            wgpu::VertexFormat::Float64     => 30,
            wgpu::VertexFormat::Float64x2   => 31,
            wgpu::VertexFormat::Float64x3   => 32,
            wgpu::VertexFormat::Float64x4   => 33,
        }
    }
}
impl TAsVertexLayoutsKey for wgpu::VertexStepMode {
    fn bytes_for_vertex_layouts_key(&self) -> u8 {
        BYTES_VERTEX_BUFFER_STEP_MODE
    }
    fn as_vertex_layouts_key(&self) -> KeyVertexLayout {
        match self {
            wgpu::VertexStepMode::Vertex    => 0,
            wgpu::VertexStepMode::Instance  => 1,
        }
    }
}

#[derive(Debug)]
pub struct VertexLayoutsKeyCalcolator {
    pub key: KeyVertexLayout,
    pub use_bytes: u8,
    pub key2: KeyVertexLayout,
    pub use_bytes2: u8,
}
impl VertexLayoutsKeyCalcolator {
    pub const MAX_BYTES: u8 = 128;

    pub fn isok(&self) -> bool {
        self.use_bytes2 <= VertexLayoutsKeyCalcolator::MAX_BYTES
    }
    pub fn calc(&mut self, v: KeyVertexLayout, bytes: u8) {
        if self.use_bytes < VertexLayoutsKeyCalcolator::MAX_BYTES {
            let diff = KeyVertexLayout::pow(2, self.use_bytes as u32);
            self.key += v * diff;
            self.use_bytes += bytes;
        } else if self.use_bytes2 < VertexLayoutsKeyCalcolator::MAX_BYTES {
            let diff = KeyVertexLayout::pow(2, self.use_bytes2 as u32);
            self.key2 += v * diff;
            self.use_bytes2 += bytes;
        }
    }
}

#[derive(Debug, Clone)]
pub struct VertexAttribute {
    pub kind: EVertexDataKind,
    pub format: wgpu::VertexFormat,
}
impl VertexAttribute {
    fn calc_key(&self, calcolator: &mut VertexLayoutsKeyCalcolator) {
        calcolator.calc(
            self.kind.as_vertex_layouts_key(),
            self.kind.bytes_for_vertex_layouts_key()
        );
    }
}
impl TAsWgpuVertexAtribute for VertexAttribute {
    fn as_attribute(&self, offset: wgpu::BufferAddress, shader_location: u32) -> wgpu::VertexAttribute {
        wgpu::VertexAttribute {
            format: self.format,
            offset,
            shader_location,
        }
    }
}

pub trait TVertexBufferDesc: Debug {
    fn attributes(&self) -> &Vec<VertexAttribute>;
    fn stride(&self) -> wgpu::BufferAddress {
        
        let mut result = 0;
        self.attributes().iter().for_each(|attr| {
            result += attr.format.attribute_bytes_size()
        });

        result
    }
    fn step_mode(&self) -> wgpu::VertexStepMode;
    fn calc_key(&self, calcolator: &mut VertexLayoutsKeyCalcolator) {
        calcolator.calc(
            self.step_mode().as_vertex_layouts_key(),
            self.step_mode().bytes_for_vertex_layouts_key()
        );

        self.attributes().iter().for_each(|attribute| {
            attribute.calc_key(calcolator);
        });
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EInstanceKind {
    None,
    WorldMatrix,
    Color,
    TillOffset,
}

#[derive(Debug, Clone)]
///
/// 
/// Range<wgpu::BufferAddress> : byte数据范围
pub struct  VertexBufferDesc {
    key: KeyVertexBuffer,
    range: Option<Range<wgpu::BufferAddress>>,
    attrs: Vec<VertexAttribute>,
    step_mode: wgpu::VertexStepMode,
    kind: EInstanceKind,
}
impl VertexBufferDesc {
    pub fn update_range(&mut self, value: Option<Range<wgpu::BufferAddress>>) {
        let _ = replace(&mut self.range, value);
    }
    pub fn instance_tilloff() -> Self {
        Self {
            key: KeyVertexBuffer::from("NullIntanceTillOff"),
            range: None,
            attrs: vec![
                VertexAttribute { kind: EVertexDataKind::InsTillOffset1, format: wgpu::VertexFormat::Float32x4 },
            ],
            step_mode: wgpu::VertexStepMode::Instance,
            kind: EInstanceKind::TillOffset,
        }
    }
    pub fn instance_color() -> Self {
        Self {
            key: KeyVertexBuffer::from("NullIntanceColor"),
            range: None,
            attrs: vec![
                VertexAttribute { kind: EVertexDataKind::InsColor, format: wgpu::VertexFormat::Float32x4 },
            ],
            step_mode: wgpu::VertexStepMode::Instance,
            kind: EInstanceKind::Color,
        }
    }
    pub fn instance_world_matrix() -> Self {
        Self {
            key: KeyVertexBuffer::from("NullIntanceWM"),
            range: None,
            attrs: vec![
                VertexAttribute { kind: EVertexDataKind::InsWorldRow1, format: wgpu::VertexFormat::Float32x4 },
                VertexAttribute { kind: EVertexDataKind::InsWorldRow2, format: wgpu::VertexFormat::Float32x4 },
                VertexAttribute { kind: EVertexDataKind::InsWorldRow3, format: wgpu::VertexFormat::Float32x4 },
                VertexAttribute { kind: EVertexDataKind::InsWorldRow4, format: wgpu::VertexFormat::Float32x4 },
            ],
            step_mode: wgpu::VertexStepMode::Instance,
            kind: EInstanceKind::WorldMatrix,
        }
    }
    pub fn vertices(bufferkey: KeyVertexBuffer, range: Option<Range<wgpu::BufferAddress>>, attrs: Vec<VertexAttribute>) -> Self {
        Self {
            key: bufferkey,
            range,
            attrs,
            step_mode: wgpu::VertexStepMode::Vertex,
            kind: EInstanceKind::None,
        }
    }
    pub fn bufferkey(&self) -> &KeyVertexBuffer {
        &self.key
    }
    pub fn range(&self) -> &Option<Range<wgpu::BufferAddress>> {
        &self.range
    }
    pub fn instance_kind(&self) -> EInstanceKind {
        self.kind
    }
}
impl TVertexBufferDesc for VertexBufferDesc {
    fn attributes(&self) -> &Vec<VertexAttribute> {
        &self.attrs
    }

    fn step_mode(&self) -> wgpu::VertexStepMode {
        self.step_mode
    }
}


#[derive(Debug, Clone)]
pub struct ResVertexBufferLayout {
    pub list: Vec<wgpu::VertexAttribute>,
    pub kinds: Vec<EVertexDataKind>,
    pub stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
}
impl TVertexShaderCode for ResVertexBufferLayout {
    fn vs_defines_code(&self) -> String {
        let mut result = String::from("");
        let mut index = 0;
        self.list.iter().for_each(|attribute| {
            result += attribute.vs_code().as_str();

            result += "V";
            result += self.kinds.get(index).unwrap().vs_code();

            result += ";\r\n";

            index += 1;
        });

        result
    }
    fn vs_running_code(&self) -> String {
        let mut result = String::from("");
        let mut index = 0;
        self.list.iter().for_each(|attribute| {
            let kind = self.kinds.get(index).unwrap();
            if *kind != EVertexDataKind::Color4 && *kind != EVertexDataKind::Normal {
                result += attribute.format.vs_code().as_str();
                result += " ";
            }
            result += self.kinds.get(index).unwrap().vs_code();
            result += " = V";
            result += self.kinds.get(index).unwrap().vs_code();
            result += ";\r\n";

            index += 1;
        });

        result
    }
}
impl ResVertexBufferLayout {
    pub fn size(&self) -> usize {
        self.kinds.len() * size_of::<EVertexDataKind>()
        + self.list.len() * size_of::<wgpu::VertexAttribute>()
        + size_of::<wgpu::BufferAddress>()
        + size_of::<wgpu::VertexStepMode>()
    }
}

#[derive(Debug, Clone)]
pub struct VertexBufferLayouts {
    pub layout_list: Vec<ResVertexBufferLayout>,
    pub size: usize,
}
impl<T: TVertexBufferDesc> From<&Vec<T>> for VertexBufferLayouts {
    fn from(value: &Vec<T>) -> Self {
        let mut layouts = vec![];
        let mut shader_location = 0;

        let mut datasize = 0;
        value.iter().for_each(|buffer_desc| {
            let mut temp_attributes = ResVertexBufferLayout { list: vec![], kinds: vec![], stride: 0, step_mode: buffer_desc.step_mode() };

            buffer_desc.attributes().iter().for_each(|attribute| {
                let temp = attribute.as_attribute(temp_attributes.stride, shader_location);

                shader_location += 1;
                temp_attributes.kinds.push(attribute.kind);
                temp_attributes.list.push(temp);
                temp_attributes.stride += attribute.format.attribute_bytes_size();
            });

            datasize += temp_attributes.size();
            layouts.push(temp_attributes);
        });

        Self { layout_list: layouts, size: datasize }
    }
}
impl TVertexShaderCode for VertexBufferLayouts {
    fn vs_defines_code(&self) -> String {
        let mut result = String::from("");

        self.layout_list.iter().for_each(|layout| {
            result += layout.vs_defines_code().as_str();
        });

        result
    }

    fn vs_running_code(&self) -> String {
        let mut result = String::from("");

        self.layout_list.iter().for_each(|layout| {
            result += layout.vs_running_code().as_str();
        });

        result
    }
}
impl VertexBufferLayouts {
    pub fn calc_key<T: TVertexBufferDesc>(layouts: &Vec<T>) -> Result<KeyVertexLayouts, EGeometryError> {
        let mut calcolator = VertexLayoutsKeyCalcolator { key: 0, use_bytes: 0, key2: 0, use_bytes2: 0  };
        layouts.iter().for_each(|layout| {
            layout.calc_key(&mut calcolator);
        });

        if calcolator.isok() {
            println!(">>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
            println!("{:?}", calcolator);
            println!("{:?}", layouts);
            Ok(KeyVertexLayouts(calcolator.key, calcolator.key2))
        } else {
            Err(EGeometryError::AttributesInfoTooMoreNotSupport)
        }
    }

    pub fn layouts(&self) -> Vec<wgpu::VertexBufferLayout> {
        let mut list = vec![];
        self.layout_list.iter().for_each(|item| {
            list.push(
                wgpu::VertexBufferLayout {
                    array_stride: item.stride,
                    step_mode: item.step_mode,
                    attributes: item.list.as_slice(),
                }
            );
        });

        list
    }
}
impl Asset for VertexBufferLayouts {
    type Key = KeyVertexLayouts;

    fn size(&self) -> usize {
        self.size
    }
}

#[derive(Clone, Debug)]
pub struct ResVertexBufferLayouts(pub Share<VertexBufferLayouts>);
impl From<VertexBufferLayouts> for ResVertexBufferLayouts {
    fn from(value: VertexBufferLayouts) -> Self {
        ResVertexBufferLayouts(Share::new(value))
    }
}

impl Deref for ResVertexBufferLayouts {
    type Target = VertexBufferLayouts;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Asset for ResVertexBufferLayouts {
    type Key = KeyVertexLayouts;

    fn size(&self) -> usize {
        self.0.size
    }
}


#[cfg(test)]
mod vertex_code_test {
    use pi_assets::asset::Asset;

    use crate::vertex_code::TVertexShaderCode;

    use super::{TVertexBufferDesc, VertexAttribute, VertexBufferLayouts};

    #[derive(Debug)]
    pub struct TestVertexBufferDesc {
        pub attributes: Vec<VertexAttribute>,
        pub step_mode: wgpu::VertexStepMode,
    }
    impl TVertexBufferDesc for TestVertexBufferDesc {
        fn attributes(&self) -> &Vec<VertexAttribute> {
            &self.attributes
        }

        fn step_mode(&self) -> wgpu::VertexStepMode {
            self.step_mode
        }
    }

    /// .
    #[test]
    fn test() {
        let meshdes = vec![
            TestVertexBufferDesc {
                attributes: vec![
                    VertexAttribute { kind: super::EVertexDataKind::Position, format: wgpu::VertexFormat::Float32x3 },
                    VertexAttribute { kind: super::EVertexDataKind::Normal, format: wgpu::VertexFormat::Float32x3 },
                    VertexAttribute { kind: super::EVertexDataKind::UV, format: wgpu::VertexFormat::Float32x2 }
                ],
                step_mode: wgpu::VertexStepMode::Vertex,
            },
            TestVertexBufferDesc { 
                attributes: vec![
                    VertexAttribute { kind: super::EVertexDataKind::Color4, format: wgpu::VertexFormat::Float32x4 }
                ],
                step_mode: wgpu::VertexStepMode::Instance,
            }
        ];

        let reslayouts = VertexBufferLayouts::from(&meshdes);
        
        println!("{}", reslayouts.vs_defines_code());
        println!("{}", reslayouts.vs_running_code());
        println!("{:?}", reslayouts.layouts());
        println!("{:?}", reslayouts.size);
    }
}