use std::{mem::{size_of, replace}, ops::{Deref, Range}, fmt::Debug, hash::Hash};

use pi_assets::asset::Asset;
use pi_share::Share;
use render_core::rhi::pipeline::VertexBufferLayout;
use render_data_container::{TVertexDataKindKey, TVertexBufferID, vertex_layout_key::{KeyVertexLayouts, KeyVertexLayout}, KeyVertexBuffer};
use render_shader::{attributes::{EVertexDataKind, ShaderAttribute}, shader::KeyShaderFromAttributes};

use crate::{ error::EGeometryError, vertex_attribute::{VertexAttribute, TAsWgpuVertexAtribute}, vertex_buffer_desc::VertexBufferDesc, vertex_format::TVertexFormatByteSize};


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ResVertexBufferLayout {
    pub kinds: Vec<EVertexDataKind>,
    pub list: Vec<wgpu::VertexAttribute>,
    pub stride: wgpu::BufferAddress,
    pub step_mode: wgpu::VertexStepMode,
}
impl ResVertexBufferLayout {
    pub fn size(&self) -> usize {
        self.kinds.len() * size_of::<EVertexDataKind>()
        + self.list.len() * size_of::<wgpu::VertexAttribute>()
        + size_of::<wgpu::BufferAddress>()
        + size_of::<wgpu::VertexStepMode>()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VertexBufferLayouts {
    layout_list: Vec<ResVertexBufferLayout>,
    pub size: usize,
}
impl From<&Vec<VertexBufferDesc>> for VertexBufferLayouts {
    fn from(value: &Vec<VertexBufferDesc>) -> Self {
        let mut layouts = vec![];

        // 按 EVertexDataKind 排序确定 shader_location
        let mut temp_kinds = vec![];
        value.iter().for_each(|buffer_desc| {
            buffer_desc.attributes().iter().for_each(|attribute| {
                match temp_kinds.binary_search(&attribute.kind) {
                    Ok(_) => {
                        // 重复的顶点属性
                        log::error!("[{:?}] Can only be set once", attribute.kind);
                    },
                    Err(index) => {
                        temp_kinds.insert(index, attribute.kind);
                    },
                }
            });
        });

        let mut datasize = 0;
        value.iter().for_each(|buffer_desc| {
            let mut temp_attributes = ResVertexBufferLayout { list: vec![], kinds: vec![], stride: 0, step_mode: buffer_desc.step_mode() };

            buffer_desc.attributes().iter().for_each(|attribute| {
                match temp_kinds.binary_search(&attribute.kind) {
                    Ok(shader_location) => {
                        let temp = attribute.as_attribute(temp_attributes.stride, shader_location as u32);

                        temp_attributes.kinds.push(attribute.kind);
                        temp_attributes.list.push(temp);
                        temp_attributes.stride += attribute.format.use_bytes();
                    },
                    Err(_) => todo!(),
                }
            });

            datasize += temp_attributes.size();
            layouts.push(temp_attributes);
        });

        Self { layout_list: layouts, size: datasize }
    }
}
impl VertexBufferLayouts {
    pub fn as_key_shader_from_attributes(&self) -> KeyShaderFromAttributes {
        let mut result = KeyShaderFromAttributes(vec![]);

        self.layout_list.iter().for_each(|layout| {
            let len = layout.list.len();

            for i in 0..len {
                result.0.push(
                    ShaderAttribute {
                        kind: layout.kinds.get(i).unwrap().clone(),
                        location: layout.list.get(i).unwrap().shader_location,
                    }
                );
            }
        });

        result.0.sort();

        result
    }
    pub fn as_key_pipeline_from_vertex_layout(&self) -> Vec<ResVertexBufferLayout> {
        self.layout_list.clone()
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
    type Key = VertexBufferLayouts;

    fn size(&self) -> usize {
        self.size
    }
}


#[cfg(test)]
mod vertex_code_test {
    use std::hash::Hash;

    use render_data_container::KeyVertexBuffer;
    use render_shader::shader::TShaderBlockCode;

    use crate::vertex_buffer_desc::{VertexBufferDesc, EInstanceKind};

    use super::{VertexAttribute, VertexBufferLayouts};

    /// .
    #[test]
    fn test() {
        let meshdes = vec![
            VertexBufferDesc {
                key: KeyVertexBuffer::from("a1"),
                range: None,
                kind: EInstanceKind::None,
                attrs: vec![
                    VertexAttribute { kind: super::EVertexDataKind::Position, format: wgpu::VertexFormat::Float32x3 },
                    VertexAttribute { kind: super::EVertexDataKind::Normal, format: wgpu::VertexFormat::Float32x3 },
                    VertexAttribute { kind: super::EVertexDataKind::UV, format: wgpu::VertexFormat::Float32x2 }
                ],
                step_mode: wgpu::VertexStepMode::Vertex,
            },
            VertexBufferDesc {
                key: KeyVertexBuffer::from("a0"),
                range: None,
                kind: EInstanceKind::None,
                attrs: vec![
                    VertexAttribute { kind: super::EVertexDataKind::Color4, format: wgpu::VertexFormat::Float32x4 }
                ],
                step_mode: wgpu::VertexStepMode::Instance,
            }
        ];

        let reslayouts = VertexBufferLayouts::from(&meshdes);
        let keyshader_attribute = reslayouts.as_key_shader_from_attributes();
        
        println!("{}", keyshader_attribute.vs_define_code());
        println!("{}", keyshader_attribute.vs_running_code());
        println!("{:?}", reslayouts.layouts());
        println!("{:?}", reslayouts.size);
    }
}