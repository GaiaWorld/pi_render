use std::hash::Hash;
use render_core::rhi::{bind_group::BindGroup, dyn_uniform_buffer::BindOffset};
use render_data_container::{TVertexDataKindKey, TMaterialBlockKindKey, TextureID, TexturePool, GeometryBufferPool, TGeometryBufferID, EVertexDataFormat, calc_uniform_size};
use render_geometry::{geometry::{VertexBufferMeta}};
use render_data_container::{Matrix, Vector2, Vector4, Matrix2, Color4};

use crate::{
    error::EMaterialError,
    // texture::MaterialTextureSampler,
    binding::{BindingData, BindingDesc},
    // uniform_info::calc_uniform_size
};

/// 数值 Uniform 数据类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EUniformDataFormat {
    Float,
    Float2,
    Float4,
    Color4,
    Mat2,
    Mat4,
}
impl EUniformDataFormat {
    /// 各类数据类型对应的 字节数
    pub fn match_uniform_size(&self) -> wgpu::BufferAddress {
        match self {
            EUniformDataFormat::Float => 4,
            EUniformDataFormat::Float2 => 8,
            EUniformDataFormat::Float4 => 16,
            EUniformDataFormat::Color4 => 16,
            EUniformDataFormat::Mat2 => 16,
            EUniformDataFormat::Mat4 => 64,
        }
    }
    /// 各类数据类型对应的 对齐字节数
    pub fn fill_size(&self) -> wgpu::BufferAddress {
        match self {
            EUniformDataFormat::Float => 4,
            EUniformDataFormat::Float2 => 8,
            EUniformDataFormat::Float4 => 16,
            EUniformDataFormat::Color4 => 16,
            EUniformDataFormat::Mat2 => 16,
            EUniformDataFormat::Mat4 => 16,
        }
    }
}

pub type UniformKindFloat   = f32;
pub type UniformKindFloat2  = Vector2;
pub type UniformKindFloat4  = Vector4;
pub type UniformKindColor4  = Color4;
pub type UniformKindMat2    = Matrix2;
pub type UniformKindMat4    = Matrix;
// pub type UniformKindTexture2D = Option<(f32, f32, f32, f32)>;

/// 材质上的 Uniform 数据描述
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniformDesc<MBKK: TMaterialBlockKindKey> {
    /// 应用为哪个材质属性
    pub kind: MBKK,
    /// 数据所属 bind 信息
    pub bind: usize,
    /// 数据类型
    pub format: EUniformDataFormat,
    pub visibility: wgpu::ShaderStages,
    /// 在 bind 内的字节偏移
    pub byte_offset_in_bind: usize,
}
impl<MBKK: TMaterialBlockKindKey> UniformDesc<MBKK> {
    /// 根据 uniform 数据类型统计 Uniform 整体占用的数据块尺寸
    pub fn calc_buffer_size(descs: &Vec<UniformDesc<MBKK>>) -> wgpu::BufferAddress {
        let mut result = 0;
        let mut last_size = 0;
        for desc in descs.iter() {
            let fill_size = desc.format.fill_size();
            if last_size == 0 || last_size == fill_size {
                last_size = fill_size;
            } else if last_size < fill_size {
                last_size = fill_size;
                result += (fill_size - last_size) as wgpu::BufferAddress;
            } else {
                last_size -= fill_size;
            }
            result += desc.format.match_uniform_size() as wgpu::BufferAddress;
        }

        if last_size > 0 {
            result += last_size as wgpu::BufferAddress;
        }

        result
    }
}

///
/// 材质上的 纹理 描述
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MaterialTextureDesc<MBKK: TMaterialBlockKindKey> {
    /// 应用为哪个材质属性
    pub kind: MBKK,
    /// 纹理数据的 bind 信息
    pub bind: u32,
    /// 纹理采样数据的 bind 信息
    pub bind_sampler: u32,
}

///
/// Uniform 数据结构 - 用于逻辑操作
pub enum UnifromData {
    Float(UniformKindFloat),
    Float2(UniformKindFloat2),
    Float4(UniformKindFloat4),
    Color4(UniformKindColor4),
    Mat2(UniformKindMat2),
    Mat4(UniformKindMat4),
}

impl UnifromData {
    pub fn to_data(
        &self,
        data: &mut Vec<u8>,
        offset: usize,
        format: &EUniformDataFormat,
    ) -> Result<(), EMaterialError> {
        match self {
            UnifromData::Float(v) => {
                match format {
                    EUniformDataFormat::Float => {
                        let v = vec![*v];
                        let bytes = bytemuck::cast_slice(&v);
                        let mut index = 0;
                        for v in bytes.iter()  {
                            data[offset + index] = *v;
                            index += 1;
                        }
                        Ok(())
                    },
                    _ => Err(EMaterialError::UniformDataNotMatch),
                }
            },
            UnifromData::Float2(v) => {
                match format {
                    EUniformDataFormat::Float2 => {
                        let bytes = bytemuck::cast_slice(v.as_slice());
                        let mut index = 0;
                        for v in bytes.iter()  {
                            data[offset + index] = *v;
                            index += 1;
                        }
                        Ok(())
                    },
                    _ => Err(EMaterialError::UniformDataNotMatch),
                }
            },
            UnifromData::Float4(v) => {
                match format {
                    EUniformDataFormat::Float4 => {
                        let bytes = bytemuck::cast_slice(v.as_slice());
                        let mut index = 0;
                        for v in bytes.iter()  {
                            data[offset + index] = *v;
                            index += 1;
                        }
                        Ok(())
                    },
                    _ => Err(EMaterialError::UniformDataNotMatch),
                }
            },
            UnifromData::Color4(v) => {
                match format {
                    EUniformDataFormat::Color4 => {
                        let bytes = bytemuck::cast_slice(v.as_slice());
                        let mut index = 0;
                        for v in bytes.iter()  {
                            data[offset + index] = *v;
                            index += 1;
                        }
                        Ok(())
                    },
                    _ => Err(EMaterialError::UniformDataNotMatch),
                }
            },
            UnifromData::Mat2(v) => {
                match format {
                    EUniformDataFormat::Mat2 => {
                        let bytes = bytemuck::cast_slice(v.as_slice());
                        let mut index = 0;
                        for v in bytes.iter()  {
                            data[offset + index] = *v;
                            index += 1;
                        }
                        Ok(())
                    },
                    _ => Err(EMaterialError::UniformDataNotMatch),
                }
            },
            UnifromData::Mat4(v) => {
                match format {
                    EUniformDataFormat::Mat4 => {
                        let bytes = bytemuck::cast_slice(v.as_slice());
                        let mut index = 0;
                        for v in bytes.iter()  {
                            data[offset + index] = *v;
                            index += 1;
                        }
                        Ok(())
                    },
                    _ => Err(EMaterialError::UniformDataNotMatch),
                }
            },
        }
    }
    /// 获取 Float 数据
    pub fn to_float(&self, data: &mut UniformKindFloat) -> Result<(), EMaterialError> {
        match self {
            UnifromData::Float(v) => {
                *data = *v;
                Ok(())
            },
            _ => Err(EMaterialError::UniformDataNotMatch),
        }
    }
    /// 获取 Float2 数据
    pub fn to_float2(&self, data: &mut UniformKindFloat2) -> Result<(), EMaterialError> {
        match self {
            UnifromData::Float2(v) => {
                data.copy_from(v);
                Ok(())
            },
            _ => Err(EMaterialError::UniformDataNotMatch),
        }
    }
    /// 获取 Float4 数据
    pub fn to_float4(&self, data: &mut UniformKindFloat4) -> Result<(), EMaterialError> {
        match self {
            UnifromData::Float4(v) => {
                data.copy_from(v);
                Ok(())
            },
            _ => Err(EMaterialError::UniformDataNotMatch),
        }
    }
    /// 获取 Color4 数据
    pub fn to_color4(&self, data: &mut UniformKindColor4) -> Result<(), EMaterialError> {
        match self {
            UnifromData::Color4(v) => {
                data.copy_from(v);
                Ok(())
            },
            _ => Err(EMaterialError::UniformDataNotMatch),
        }
    }
    /// 获取 Mat2 数据
    pub fn to_mat2(&self, data: &mut UniformKindMat2) -> Result<(), EMaterialError> {
        match self {
            UnifromData::Mat2(v) => {
                data.copy_from(v);
                Ok(())
            },
            _ => Err(EMaterialError::UniformDataNotMatch),
        }
    }
    /// 获取 Mat4 数据
    pub fn to_mat4(&self, data: &mut UniformKindMat4) -> Result<(), EMaterialError> {
        match self {
            UnifromData::Mat4(v) => {
                data.copy_from(v);
                Ok(())
            },
            _ => Err(EMaterialError::UniformDataNotMatch),
        }
    }
}

// ///
// /// 材质
// /// * 抽象 Uniform 数据操作
// ///   * 在 JS 原本操作为
// ///     * 接口封装层 setUniform数据类型(属性名称, Value)
// ///   * 此处封装为 set_uniform(属性描述, Value)
// ///     * 属性描述 对于一个 Shader 而言是唯一的
// /// * 数值 Uniform 共用一个 set
// /// * 纹理 Uniform 共用一个 set
// pub struct Material<VDK: TVertexDataKindKey, MBKK: TMaterialBlockKindKey, TID: TextureID> {
//     /** Uniforms */
//     uniform_bind_group: Option<render_core::rhi::bind_group::BindGroup>,
//     uniform_bind_about: Vec<Option<UniformBindGroupAbout>>,
//     uniform_bind_dirty: Vec<bool>,
//     uniform_buffer: Option<render_core::rhi::dyn_uniform_buffer::DynUniformBuffer>,
//     uniform_descs: Vec<MaterialUniformDesc<MBKK>>,
//     /// 每个Uniform在各自数据类型的存储数值中的存储位置
//     uniform_type_save_index: Vec<usize>,
//     float_pool: Vec<UniformKindFloat>,
//     float2_pool: Vec<UniformKindFloat2>,
//     float4_pool: Vec<UniformKindFloat4>,
//     color4_pool: Vec<UniformKindColor4>,
//     mat2_pool: Vec<UniformKindMat2>,
//     mat4_pool: Vec<UniformKindMat4>,
//     /** Textures */
//     texture_bind_group: Option<render_core::rhi::bind_group::BindGroup>,
//     texture_keys: Vec<Option<TID>>,
//     texture_samplers: Vec<MaterialTextureSampler>,
//     texture_descs: Vec<MaterialTextureDesc<MBKK>>,
//     texture_dirty: bool,
//     /** Attributes */
//     attribute_descs: Vec<GeometryBufferDesc<VDK>>,
//     attribute_slot_desc: Vec<MBKK>,
// }

// impl<VBK: TVertexDataKindKey, MBKK: TMaterialBlockKindKey, TID: TextureID> Default for Material<VBK, MBKK, TID> {
//     fn default() -> Self {
//         Self {
//             uniform_bind_group: None,
//             uniform_bind_about: vec![],
//             uniform_bind_dirty: vec![],
//             uniform_buffer: None,
//             uniform_descs: vec![],
//             float_pool: vec![],
//             float2_pool: vec![],
//             float4_pool: vec![],
//             color4_pool: vec![],
//             mat2_pool: vec![],
//             mat4_pool: vec![],
//             texture_bind_group: None,
//             texture_keys: vec![],
//             texture_samplers: vec![],
//             texture_descs: vec![],
//             texture_dirty: false,
//             uniform_type_save_index: vec![],
//             attribute_descs: vec![],
//             attribute_slot_desc: vec![],
//         }
//     }
// }

// impl<VBK: TVertexDataKindKey, MBKK: TMaterialBlockKindKey, TID: TextureID> Material<VBK, MBKK, TID> {
//     pub fn init(
//         &mut self,
//         device: &wgpu::Device,
//         attributes: Vec<GeometryBufferDesc<VBK>>,
//         uniform_usage: wgpu::BufferUsages,
//         uniform_descs: Vec<MaterialUniformDesc<MBKK>>,
//         textures: Vec<MaterialTextureDesc<MBKK>>,
//         uniform_bind_group_layout: &wgpu::BindGroupLayout,
//     ) {
//         self.uniform_bind_about.clear();
//         self.uniform_bind_dirty.clear();
//         self.uniform_descs.clear();
//         self.float_pool.clear();
//         self.float2_pool.clear();
//         self.float4_pool.clear();
//         self.color4_pool.clear();
//         self.mat2_pool.clear();
//         self.mat4_pool.clear();
//         self.texture_keys.clear();
//         self.texture_descs.clear();
//         self.uniform_type_save_index.clear();
//         self.attribute_descs.clear();
//         self.attribute_slot_desc.clear();

//         // 计算 数值 Uniform 需要的数据空间
//         let used_size = MaterialUniformDesc::calc_buffer_size(&uniform_descs) as wgpu::BufferAddress;
//         if used_size > 0 {
//             let buffer_size = calc_uniform_size(device, used_size);
//             // 创建 数值 Unifrom 的 buffer
//             self.uniform_buffer = Some(
//                 device.create_buffer(&wgpu::BufferDescriptor {
//                     label: None,
//                     size: buffer_size,
//                     usage: uniform_usage,
//                     mapped_at_creation: false,
//                 })
//             );
//         }
//         // 初始化 数值 Uniform 信息记录
//         for desc in uniform_descs.iter() {
//             self.add_unifrom_desc(*desc);
//         }

//         // 初始化纹理信息记录
//         for _ in textures.iter() {
//             self.texture_keys.push(None);
//             self.texture_samplers.push(MaterialTextureSampler::default());
//         }
//         self.texture_bind_group = None;
//         self.texture_descs = textures;
//         self.attribute_descs = attributes;

//         let mut uniform_binds = vec![];
//         uniform_binds.push(
//             wgpu::BindGroupEntry {
//                 binding: 0,
//                 resource: wgpu::BindingResource::Buffer (
//                     wgpu::BufferBinding {
//                         buffer: self.uniform_buffer.as_ref().unwrap(),
//                         offset: 0,
//                         size: wgpu::BufferSize::new(MaterialUniformDesc::calc_buffer_size(&uniform_descs)),
//                     }
//                 ),
//             }
//         );
//         self.uniform_bind_group = Some(
//             device.create_bind_group(
//                 &wgpu::BindGroupDescriptor {
//                     label: None,
//                     layout: &uniform_bind_group_layout,
//                     entries: uniform_binds.as_slice()
//                 }
//             )
//         );
//     }

//     fn init_bind_group() {

//     }

//     fn add_unifrom_desc(&mut self, desc: MaterialUniformDesc<MBKK>) {
//         match desc.format {
//             EUniformDataFormat::Float => {
//                 let index = self.float_pool.len();
//                 self.float_pool.push(0.);
//                 self.uniform_descs.push(desc);
//                 self.uniform_type_save_index.push(index);
//             },
//             EUniformDataFormat::Float2 => {
//                 let index = self.float2_pool.len();
//                 self.float2_pool.push(Vector2::new(0., 0.));
//                 self.uniform_descs.push(desc);
//                 self.uniform_type_save_index.push(index);
//             },
//             EUniformDataFormat::Float4 => {
//                 let index = self.float4_pool.len();
//                 self.float4_pool.push(Vector4::new(0., 0., 1., 1.));
//                 self.uniform_descs.push(desc);
//                 self.uniform_type_save_index.push(index);
//             },
//             EUniformDataFormat::Color4 => {
//                 let index = self.float4_pool.len();
//                 self.float4_pool.push(Vector4::new(0., 0., 1., 1.));
//                 self.uniform_descs.push(desc);
//                 self.uniform_type_save_index.push(index);
//             },
//             EUniformDataFormat::Mat2 => {
//                 let index = self.float4_pool.len();
//                 self.float4_pool.push(Vector4::new(0., 0., 1., 1.));
//                 self.uniform_descs.push(desc);
//                 self.uniform_type_save_index.push(index);
//             },
//             EUniformDataFormat::Mat4 => {
//                 let index = self.mat4_pool.len();
//                 self.mat4_pool.push(Matrix::identity());
//                 self.uniform_descs.push(desc);
//                 self.uniform_type_save_index.push(index);
//             },
//         }
//     }

//     pub fn set_uniform(&mut self, desc: MaterialUniformDesc<MBKK>, data: UnifromData) -> Result<(), EMaterialError> {
//         match self.uniform_descs.binary_search(&desc) {
//             Ok(index) => {
//                 match self.uniform_type_save_index.get(index) {
//                     Some(index) => {
//                         match desc.format {
//                             EUniformDataFormat::Float => {
//                                 let value = self.float_pool.get_mut(*index).unwrap();
//                                 self.uniform_bind_dirty = true;
//                                 data.to_float(value)
//                             },
//                             EUniformDataFormat::Float2 => {
//                                 let value = self.float2_pool.get_mut(*index).unwrap();
//                                 self.uniform_bind_dirty = true;
//                                 data.to_float2(value)
//                             },
//                             EUniformDataFormat::Float4 => {
//                                 let value = self.float4_pool.get_mut(*index).unwrap();
//                                 self.uniform_bind_dirty = true;
//                                 // println!("################### Float4");
//                                 data.to_float4(value)
//                             },
//                             EUniformDataFormat::Color4 => {
//                                 let value = self.color4_pool.get_mut(*index).unwrap();
//                                 self.uniform_bind_dirty = true;
//                                 data.to_color4(value)
//                             },
//                             EUniformDataFormat::Mat2 => {
//                                 let value = self.mat2_pool.get_mut(*index).unwrap();
//                                 self.uniform_bind_dirty = true;
//                                 data.to_mat2(value)
//                             },
//                             EUniformDataFormat::Mat4 => {
//                                 // println!("################### Mat4");
//                                 let value = self.mat4_pool.get_mut(*index).unwrap();
//                                 self.uniform_bind_dirty = true;
//                                 data.to_mat4(value)
//                             },
//                         }
//                     },
//                     None => Err(EMaterialError::NotSupportUniformDesc),
//                 }
//             },
//             Err(_) => {
//                 Err(EMaterialError::NotSupportUniformDesc)
//             },
//         }
//     }
    
//     pub fn set_texture(&mut self, kind: MaterialTextureDesc<MBKK>, sampler: &MaterialTextureSampler, key: TID) {
//         match self.texture_descs.binary_search(&kind) {
//             Ok(index) => {
//                 let new_sampler = sampler.clone();
//                 let mut bind_dirty = false;
//                 let old_key = self.texture_keys.get(index).unwrap();
//                 let old_sampler = self.texture_samplers.get(index).unwrap();
//                 let old_bind = self.texture_bind_group.as_ref();

//                 bind_dirty = bind_dirty || (old_key.is_none() || old_key.unwrap() != key);
//                 bind_dirty = bind_dirty || (!old_sampler.is_same(sampler));
//                 bind_dirty = bind_dirty || old_bind.is_none();

//                 self.texture_keys[index] = Some(key);
//                 self.texture_samplers[index] = new_sampler;

//                 self.texture_dirty = bind_dirty;
//             },
//             Err(_) => {
                
//             },
//         }
//     }
    
//     pub fn update_uniform<TP: TexturePool<TID>>(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, texture_layout: Option<&wgpu::BindGroupLayout>, textures: &TP) {
//         if self.texture_dirty {
//             match texture_layout {
//                 Some(texture_layout) => {
//                     let mut entries = vec![];
//                     let mut i = 0;
//                     let keys = &self.texture_keys;
//                     let mut samplers = vec![];
//                     self.texture_samplers.iter().for_each(|v| { samplers.push(v.to_sampler_resource(device, None)) });
//                     for desc in self.texture_descs.iter() {
//                         let key = keys.get(i).unwrap();
        
//                         match key {
//                             Some(key) => {
//                                 match textures.get(*key) {
//                                     Some(textureview) => {
//                                         entries.push(
//                                             wgpu::BindGroupEntry {
//                                                 binding: desc.bind_sampler,
//                                                 resource: wgpu::BindingResource::Sampler(samplers.get(i).unwrap()),
//                                             }
//                                         );
//                                         entries.push(
//                                             wgpu::BindGroupEntry {
//                                                 binding: desc.bind,
//                                                 resource: wgpu::BindingResource::TextureView (
//                                                     textureview
//                                                 ),
//                                             }
//                                         );
//                                     },
//                                     None => {},
//                                 }
//                             },
//                             None => {},
//                         }
        
//                         i += 1;
//                     }
//                     self.texture_bind_group = Some(
//                         device.create_bind_group(
//                             &wgpu::BindGroupDescriptor {
//                                 label: None,
//                                 layout: texture_layout,
//                                 entries: entries.as_slice(),
//                             }
//                         )
//                     );
//                 },
//                 None => {},
//             }
//         }
//         if self.uniform_bind_dirty {
//             match self.uniform_buffer.as_ref() {
//                 Some(buffer) => {
//                     let mut datas = vec![];
//                     let mut i = 0;
//                     let index_list = &self.uniform_type_save_index;
//                     for desc in self.uniform_descs.iter() {
//                         let index = index_list.get(i).unwrap();
//                         match desc.format {
//                             EUniformDataFormat::Float => {
//                                 let data = self.float_pool.get(*index).unwrap();
//                                 datas.push(*data);
//                             },
//                             EUniformDataFormat::Float2 => {
//                                 let data = self.float2_pool.get(*index).unwrap();
//                                 data.as_slice().iter().for_each(|v| { datas.push(*v); });
//                             },
//                             EUniformDataFormat::Float4 => {
//                                 let data = self.float4_pool.get(*index).unwrap();
//                                 data.as_slice().iter().for_each(|v| { datas.push(*v); });
//                             },
//                             EUniformDataFormat::Color4 => {
//                                 let data = self.float4_pool.get(*index).unwrap();
//                                 data.as_slice().iter().for_each(|v| { datas.push(*v); });
//                             },
//                             EUniformDataFormat::Mat2 => {
//                                 let data = self.float4_pool.get(*index).unwrap();
//                                 data.as_slice().iter().for_each(|v| { datas.push(*v); });
//                             },
//                             EUniformDataFormat::Mat4 => {
//                                 let data = self.mat4_pool.get(*index).unwrap();
//                                 data.as_slice().iter().for_each(|v| { datas.push(*v); });
//                             },
//                         }
//                         i       += 1;
//                     }
                    
//                     // println!("!!!!!!!!!!!!! {:?}", datas);
//                     queue.write_buffer(buffer, 0, bytemuck::cast_slice(&datas));
//                     self.uniform_bind_dirty = false;
//                 },
//                 None => todo!(),
//             }
//         }
//     }
    
//     pub fn bind_groups<'a>(&'a self, renderpass: &mut wgpu::RenderPass<'a>) {
//         match self.uniform_bind_group.as_ref() {
//             Some(bind_group) => {
//                 renderpass.set_bind_group(0, bind_group, &[]);
//             },
//             None => todo!(),
//         }

//         match self.texture_bind_group.as_ref() {
//             Some(bind_group) => {
//                 renderpass.set_bind_group(1, bind_group, &[]);
//             },
//             None => {},
//         }
//     }

//     pub fn draw<'a, GBID: TGeometryBufferID, GBP: GeometryBufferPool<GBID>>(&'a self, renderpass: &mut wgpu::RenderPass<'a>, geometry: &Geometry<VBK, GBID>, geo_buffer_pool: &'a GBP) -> Result<(), EMaterialError> {
//         self.bind_groups(renderpass);

//         self.attribute_descs.iter().for_each(|desc| {
//             let data = geometry.get_vertices(desc);
//             match desc.format {
//                 EVertexDataFormat::U8 => match data {
//                     Some(id) => {
//                         match geo_buffer_pool.get_buffer(&id) {
//                             Some(buffer) => {
//                                 renderpass.set_vertex_buffer(desc.slot, buffer.slice(..));
//                             },
//                             None => {},
//                         }
//                     },
//                     None => {},
//                 },
//                 EVertexDataFormat::U16 => match data {
//                     Some(id) => {
//                         match geo_buffer_pool.get_buffer(&id) {
//                             Some(buffer) => {
//                                 renderpass.set_vertex_buffer(desc.slot, buffer.slice(..));
//                             },
//                             None => {},
//                         }
//                     },
//                     None => {},
//                 },
//                 EVertexDataFormat::U32 => match data {
//                     Some(id) => {
//                         match geo_buffer_pool.get_buffer(&id) {
//                             Some(buffer) => {
//                                 renderpass.set_vertex_buffer(desc.slot, buffer.slice(..));
//                             },
//                             None => {},
//                         }
//                     },
//                     None => {},
//                 },
//                 EVertexDataFormat::F32 => match data {
//                     Some(id) => {
//                         match geo_buffer_pool.get_buffer(&id) {
//                             Some(buffer) => {
//                                 renderpass.set_vertex_buffer(desc.slot, buffer.slice(..));
//                             },
//                             None => {},
//                         }
//                     },
//                     None => {},
//                 },
//                 EVertexDataFormat::F64 => match data {
//                     Some(id) => {
//                         match geo_buffer_pool.get_buffer(&id) {
//                             Some(buffer) => {
//                                 renderpass.set_vertex_buffer(desc.slot, buffer.slice(..));
//                             },
//                             None => {},
//                         }
//                     },
//                     None => {},
//                 },
//             }
//         });

//         let instance_count = geometry.get_instanced_number(geo_buffer_pool);

//         match geometry.get_indices() {
//             Some(indices_buffer_id) => match geo_buffer_pool.get_buffer(&indices_buffer_id) {
//                 Some(buffer) => {
//                     renderpass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
//                     renderpass.draw_indexed(0..geo_buffer_pool.get_size(&indices_buffer_id) as u32, 0, 0..instance_count as u32);
//                     Ok(())
//                 },
//                 None => {
//                     Err(EMaterialError::NotFoundIndicesBuffer)
//                 },
//             },
//             None => {
//                 match geometry.get_vertices_number(geo_buffer_pool) {
//                     Some(count) => {
//                         renderpass.draw(0..count as u32, 0..instance_count as u32);
//                         Ok(())
//                     },
//                     None => {
//                         Ok(())
//                     },
//                 }
//             },
//         }

//     }
    
//     pub fn attributes(&self) -> &Vec<GeometryBufferDesc<VBK>> {
//         &self.attribute_descs
//     }
//     fn analy_uniform_descs(&mut self, device: &wgpu::Device, descs: &Vec<MaterialUniformDesc<MBKK>>, uniform_usage: wgpu::BufferUsages) {
//         let mut bind_indexs = vec![];
//         descs.iter().for_each(|desc| {
//             if bind_indexs.contains(&desc.bind) == false {
//                 match bind_indexs.binary_search(&desc.bind) {
//                     Ok(index) => {},
//                     Err(index) => bind_indexs.insert(index, desc.bind),
//                 }
//             }
//         });

//         let mut offset = 0 as wgpu::BufferAddress;
//         let mut buffer_size = 0 as wgpu::BufferAddress;
//         let mut counter = 0 as usize;
//         bind_indexs.iter().for_each(|bind| {
//             let about = UniformBindGroupAbout::new(device, descs, *bind, offset);
//             if counter < *bind {
//                 for _ in counter..*bind {
//                     self.uniform_bind_about.push(None);
//                 }
//             }
//             self.uniform_bind_about.push(Some(about));

//             buffer_size += about.buffer_size;
//             offset      = buffer_size;
//             counter     = *bind;
//         });

//         self.uniform_buffer = Some(device.create_buffer(
//             &wgpu::BufferDescriptor {
//                 label: None,
//                 size: buffer_size,
//                 usage: uniform_usage,
//                 mapped_at_creation: false,
//             }
//         ));

//         let mut bind_entries = vec![];
//         let mut bind_layout_entries = vec![];
//         self.uniform_bind_about.iter().for_each(|about| {
//             match about {
//                 Some(about) => {
//                     bind_entries.push(wgpu::BindGroupEntry {
//                         binding: about.bind as u32,
//                         resource: wgpu::BindingResource::Buffer (
//                             wgpu::BufferBinding {
//                                 buffer: self.uniform_buffer.as_ref().unwrap(),
//                                 offset: about.buffer_offset,
//                                 size: wgpu::BufferSize::new(about.data_size),
//                             }
//                         ),
//                     });
//                     bind_layout_entries.push(
//                         wgpu::BindGroupLayoutEntry {
//                             binding: about.bind as u32,
//                             visibility: about.visibility,
//                             ty: wgpu::BindingType::Buffer {
//                                 ty: wgpu::BufferBindingType::Uniform,
//                                 has_dynamic_offset: false,
//                                 // min_binding_size: wgpu::BufferSize::new(uniform_size)
//                                 min_binding_size: None,
//                             },
//                             count: None,
//                         },
//                     )
//                 },
//                 None => {},
//             }
//         });

//         let uniform_layout = device.create_bind_group_layout(
//             &wgpu::BindGroupLayoutDescriptor {
//                 label: None,
//                 entries: bind_layout_entries.as_slice()
//             }
//         );

//         self.uniform_bind_group = Some(
//             device.create_bind_group(
//                 &wgpu::BindGroupDescriptor {
//                     label: None,
//                     layout: &uniform_layout,
//                     entries: bind_entries.as_slice()
//                 }
//             )
//         );

//     }
// }

pub struct UniformBindGroupAbout {
    pub buffer_offset: wgpu::BufferAddress,
    pub buffer_size: wgpu::BufferAddress,
    pub data_size: wgpu::BufferAddress,
    pub bind: usize,
    pub visibility: wgpu::ShaderStages,
}

impl UniformBindGroupAbout {
    pub fn new<MBKK: TMaterialBlockKindKey>(device: &wgpu::Device, descs: &Vec<UniformDesc<MBKK>>, bind: usize, offset: wgpu::BufferAddress) -> Self {

        let mut visibility = wgpu::ShaderStages::NONE;
        let mut uniform_descs = vec![];
        descs.iter().for_each(|desc| {
            if desc.bind == bind {
                uniform_descs.push(desc.clone());
                visibility = visibility | desc.visibility;
            }
        });

        let data_size = UniformDesc::calc_buffer_size(&uniform_descs) as wgpu::BufferAddress;
        let buffer_size = calc_uniform_size(device, data_size);

        Self {
            buffer_offset: offset,
            buffer_size,
            data_size,
            bind,
            visibility,
        }
    }
}

pub struct LightingEnable;

pub struct CastShadow;

pub struct ReceiveShadow;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, std::hash::Hash)]
struct UniformBindKindIndex<MBKK: TMaterialBlockKindKey> {
    kind: MBKK,
    index: usize,
}

pub struct Material<MBKK: TMaterialBlockKindKey> {
    bind_datas: Vec<BindingData<MBKK>>,
    bind_data_save: Vec<BindOffset>,
    bind_memory_blocks: Vec<BindGroup>,
    bind_groups: Vec<BindGroup>,
    bind_indexs: Vec<usize>,
    uniform_bind_indexs: Vec<usize>,
    uniform_kinds: Vec<MBKK>,
}

impl<MBKK> Default for Material<MBKK>
    where
        MBKK: TMaterialBlockKindKey
{
    fn default() -> Self {
        Self {
            bind_datas: vec![],
            bind_data_save: vec![],
            bind_groups: vec![],
            bind_indexs: vec![],
            uniform_bind_indexs: vec![],
            uniform_kinds: vec![],
            bind_memory_blocks: vec![],
        }
    }
}

impl<MBKK> Material<MBKK>
    where
        MBKK: TMaterialBlockKindKey
{
    pub fn init(
        &mut self,
        bind_descs: &Vec<BindingDesc<MBKK>>,

    ) {
        let mut uniforms = vec![];
        bind_descs.iter().for_each(|binding| {
            let index = self.bind_indexs.len();
            self.bind_indexs.push(binding.id);

            let mut bind_data = BindingData::default();
            // bind_data.init(binding, );
            self.bind_datas.push(bind_data);

            binding.uniforms.iter().for_each(|uniform| {
                uniforms.push(UniformBindKindIndex { kind: uniform.kind.clone(), index: index  });
            });
        });

        uniforms.sort();
        uniforms.iter().for_each(|uniform| {
            self.uniform_bind_indexs.push(uniform.index);
            self.uniform_kinds.push(uniform.kind.clone());
        });
    }

    pub fn modify(
        &mut self,
        kind: MBKK,
        data: UnifromData,
    ) -> Result<(), EMaterialError> {
        match self.uniform_kinds.binary_search(&kind) {
            Ok(index) => {
                match self.uniform_bind_indexs.get(index) {
                    Some(index) => match self.bind_datas.get_mut(*index) {
                        Some(bind_data) => Ok(()), //bind_data.modify(kind, data),
                        None => Err(EMaterialError::NotSupportUniformDesc),
                    },
                    None => Err(EMaterialError::NotSupportUniformDesc),
                }
            },
            Err(_) => Err(EMaterialError::NotSupportUniformDesc),
        }
    }

    pub fn update(
        &self,

    ) {

    }
}