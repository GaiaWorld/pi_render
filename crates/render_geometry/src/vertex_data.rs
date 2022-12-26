use std::{mem::size_of, ops::{Deref, Range}};

use pi_assets::asset::Asset;
use pi_share::Share;
use render_core::rhi::pipeline::VertexBufferLayout;
use render_data_container::{TVertexDataKindKey, TVertexBufferID, vertex_layout_key::{KeyVertexLayouts, KeyVertexLayout}, KeyVertexBuffer};

use crate::{vertex_code::{TVertexShaderCode, TVertexFormatCode}, error::EGeometryError};

pub const BYTES_VERTEX_DATA_KIND: u8 = 4;
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
/// 预留为支持 32 种顶点数据
#[derive(Debug, Clone, Copy)]
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
        }
    }
}
impl EVertexDataKind {
    pub fn vs_code(&self) -> &str {
        match self {
            EVertexDataKind::Position               => "A_POSITION",
            EVertexDataKind::Position2D             => "A_POSITION_2D",
            EVertexDataKind::Color4                 => "A_COLOR4",
            EVertexDataKind::UV                     => "A_UV",
            EVertexDataKind::Normal                 => "A_NORMAL",
            EVertexDataKind::Tangent                => "A_TANGENT",
            EVertexDataKind::MatricesIndices        => "A_JOINT_INC",
            EVertexDataKind::MatricesWeights        => "A_JOINT_WEG",
            EVertexDataKind::MatricesIndicesExtra   => "A_JOINT_INC_EX",
            EVertexDataKind::MatricesWeightsExtra   => "A_JOINT_WEG_EX",
            EVertexDataKind::UV2                    => "A_UV2",
            EVertexDataKind::UV3                    => "A_UV3",
            EVertexDataKind::UV4                    => "A_UV4",
            EVertexDataKind::UV5                    => "A_UV5",
            EVertexDataKind::UV6                    => "A_UV6",
            EVertexDataKind::CustomVec4A            => "A_CustomV4A",
            EVertexDataKind::CustomVec4B            => "A_CustomV4B",
            EVertexDataKind::CustomVec3A            => "A_CustomV3A",
            EVertexDataKind::CustomVec3B            => "A_CustomV3B",
            EVertexDataKind::CustomVec2A            => "A_CustomV2A",
            EVertexDataKind::CustomVec2B            => "A_CustomV2B",
            EVertexDataKind::InsWorldRow1           => "A_INS_World1",
            EVertexDataKind::InsWorldRow2           => "A_INS_World2",
            EVertexDataKind::InsWorldRow3           => "A_INS_World3",
            EVertexDataKind::InsWorldRow4           => "A_INS_World4",
            EVertexDataKind::InsColor               => "A_INS_Color",
            EVertexDataKind::InsTillOffset1         => "A_INS_TileOff1",
            EVertexDataKind::InsTillOffset2         => "A_INS_TillOff2",
            EVertexDataKind::InsCustomVec4A         => "A_INS_Vec4A",
            EVertexDataKind::InsCustomVec4B         => "A_INS_Vec4B",
            EVertexDataKind::InsCustomUVec4A        => "A_INS_UVec4A",
            EVertexDataKind::InsCustomIVec4B        => "A_INS_IVec4B",
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
        calcolator.calc(
            self.format.as_vertex_layouts_key(),
            self.format.bytes_for_vertex_layouts_key()
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

pub trait TVertexBufferDesc {
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

#[derive(Debug, Clone)]
pub struct VertexBufferDesc {
    pub bufferkey: KeyVertexBuffer,
    /// byte数据范围
    pub range: Option<Range<wgpu::BufferAddress>>,
    pub attributes: Vec<VertexAttribute>,
    pub step_mode: wgpu::VertexStepMode,
}
impl TVertexBufferDesc for VertexBufferDesc {
    fn attributes(&self) -> &Vec<VertexAttribute> {
        &self.attributes
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
            result += attribute.format.vs_code().as_str();
            result += " ";
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
                    VertexAttribute { kind: super::EVertexDataKind::Normal, format: wgpu::VertexFormat::Float32x3 }
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