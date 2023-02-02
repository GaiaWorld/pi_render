use std::{num::NonZeroU64, hash::Hash, sync::Arc};

use pi_atom::Atom;
use render_core::rhi::{dyn_uniform_buffer::{AsBind}, bind_group::BindGroup, asset::TextureRes, texture::Sampler, device::RenderDevice};
use render_resource::uniform_buffer::RenderDynUniformBuffer;

use crate::{set_bind::ShaderSetBind, skin_code::{ESkinCode, EBoneCount}, shader_bind::{ShaderBindSceneAboutCamera, ShaderBindSceneAboutTime, ShaderBindSceneAboutFog, ShaderBindModelAboutMatrix, ShaderBindEffectValue, ShaderBindModelAboutSkin}, shader::ShaderEffectMeta, unifrom_code::TUnifromShaderProperty, buildin_var::ShaderVarUniform};


pub trait ShaderSet {
    fn define_code(&self) -> String;
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShaderSetSceneAbout {
    set: u32,
    time: bool,
    fog: bool,
    brdf: bool,
    env: bool,
    bind_camera: u32,
    bind_time: u32,
    bind_fog: u32,
    bind_brdf_texture: u32,
    bind_brdf_sampler: u32,
    bind_env_param: u32,
    bind_env_texture: u32,
    bind_env_sampler: u32,
}
impl ShaderSetSceneAbout {
    pub fn new(
        set: u32,
        scene_time: bool,
        scene_fog: bool,
        scene_brdf: bool,
        scene_env: bool,
    ) -> Self {
        let mut bind = 0;

        let bind_camera = bind; bind += 1;
        let bind_time = if scene_time { 
            let result = bind; bind += 1;
            result
        } else {
            u32::MAX
        };

        let bind_fog = if scene_fog {
            let result = bind; bind += 1;
            result
        } else {
            u32::MAX
        };

        let (bind_brdf_texture, bind_brdf_sampler) = if scene_brdf {
            let result = bind; bind += 2;
            (result, result + 1)
        } else {
            (u32::MAX, u32::MAX)
        };

        let (bind_env_param, bind_env_texture, bind_env_sampler) = if scene_brdf {
            let result = bind; bind += 3; (result, result + 1, result + 2)
        } else {
            (u32::MAX, u32::MAX, u32::MAX)
        };

        Self {
            set,
            time: scene_time,
            fog: scene_fog,
            brdf: scene_brdf,
            env: scene_env,
            bind_camera,
            bind_time,
            bind_fog,
            bind_brdf_texture,
            bind_brdf_sampler,
            bind_env_param,
            bind_env_texture,
            bind_env_sampler,
        }
    }
    pub fn brdf(&self) -> bool {
        self.brdf 
    }
    pub fn env(&self) -> bool {
        self.env 
    }
    pub fn bind_camera(&self) -> u32 {
        self.bind_camera 
    }
    pub fn bind_time(&self) -> u32 {
        self.bind_time 
    }
    pub fn bind_fog(&self) -> u32 {
        self.bind_fog 
    }
    pub fn bind_brdf_texture(&self) -> u32 {
        self.bind_brdf_texture 
    }
    pub fn bind_brdf_sampler(&self) -> u32 {
        self.bind_brdf_sampler 
    }
    pub fn bind_env_param(&self) -> u32 {
        self.bind_env_param 
    }
    pub fn bind_env_texture(&self) -> u32 {
        self.bind_env_texture 
    }
    pub fn bind_env_sampler(&self) -> u32 {
        self.bind_env_sampler 
    }
    pub fn define_code(&self) -> String {
        let mut result = String::from("");

        result += ShaderSetBind::code_set_bind_head(self.set, self.bind_camera).as_str();
        result += " Camera {\r\n";
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::VIEW_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::PROJECT_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::VIEW_PROJECT_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::CAMERA_POSITION).as_str();
        result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::CAMERA_DIRECTION).as_str();
        result += "};\r\n";

        if self.time {
            result += ShaderSetBind::code_set_bind_head(self.set, self.bind_time).as_str();
            result += " Time {\r\n";
            result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::TIME).as_str();
            result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::DELTA_TIME).as_str();
            result += "};\r\n";
        }
        
        if self.fog {
            result += ShaderSetBind::code_set_bind_head(self.set, self.bind_fog).as_str();
            result += " Fog {\r\n";
            result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::FOG_INFO).as_str();
            result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::FOG_PARAM).as_str();
            result += "};\r\n";
        }

        result
    }

    pub fn vs_running_code(&self) -> String {
         String::from("")
    }

    pub fn fs_running_code(&self) -> String {
         String::from("")
    }

    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut result = vec![
            ShaderBindSceneAboutCamera::layout_entry(self.bind_camera)
        ];

        if self.time {
            result.push(ShaderBindSceneAboutTime::layout_entry(self.bind_time))
        }

        if self.fog {
            result.push(ShaderBindSceneAboutFog::layout_entry(self.bind_fog))
        }

        result
    }

    pub fn bind_group_entries<'a>(
        &'a self,
        dynbuffer: &'a RenderDynUniformBuffer,
        brdf_tex: Option<&'a TextureRes>,
        brdf_sampler: Option<&'a Sampler>,
        env_tex: Option<&'a TextureRes>,
        env_sampler: Option<&'a Sampler>,
    ) -> Vec<wgpu::BindGroupEntry> {
        let mut entries: Vec<wgpu::BindGroupEntry> = vec![
            wgpu::BindGroupEntry {
                binding: self.bind_camera,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: dynbuffer.buffer().unwrap(),
                    offset: 0,
                    size: NonZeroU64::new(ShaderBindSceneAboutCamera::TOTAL_SIZE as wgpu::BufferAddress),
                }),
            }
        ];

        if self.time {
            entries.push(wgpu::BindGroupEntry {
                binding: self.bind_time,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: dynbuffer.buffer().unwrap(),
                    offset: 0,
                    size: NonZeroU64::new(ShaderBindSceneAboutTime::TOTAL_SIZE),
                }),
            });
        }
        
        if self.fog {
            entries.push(wgpu::BindGroupEntry {
                binding: self.bind_fog,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: dynbuffer.buffer().unwrap(),
                    offset: 0,
                    size: NonZeroU64::new(ShaderBindSceneAboutFog::TOTAL_SIZE),
                }),
            });
        }

        entries
    }

    pub fn label(&self) -> &'static str {
        "SceneAbout"
    }

    pub fn bind_offset(&self, dynbuffer: &mut render_resource::uniform_buffer::RenderDynUniformBuffer) -> ShaderSetSceneAboutBindOffset {

        let time = if self.time {
            Some(ShaderBindSceneAboutTime::new(self.bind_time, dynbuffer))
        } else {
            None
        };

        let fog = if self.fog {
            Some(ShaderBindSceneAboutFog::new(self.bind_fog, dynbuffer))
        } else {
            None
        };

        ShaderSetSceneAboutBindOffset {
            camera: ShaderBindSceneAboutCamera::new(self.bind_camera, dynbuffer),
            time,
            fog,
        }
    }
}

#[derive(Debug)]
pub struct ShaderSetSceneAboutBindOffset {
    pub(crate) camera: ShaderBindSceneAboutCamera,
    pub(crate) time: Option<ShaderBindSceneAboutTime>,
    pub(crate) fog: Option<ShaderBindSceneAboutFog>,
}
impl ShaderSetSceneAboutBindOffset {
    pub fn get(&self) -> Vec<u32> {
        let mut result: Vec<u32> = vec![
            self.camera.offset()
        ];

        if let Some(time) = &self.time {
            result.push(time.offset())
        }

        if let Some(fog) = &self.fog {
            result.push(fog.offset())
        }

        result
    }

    pub fn camera(&self) -> &ShaderBindSceneAboutCamera {
        &self.camera
    }

    pub fn time(&self) -> Option<&ShaderBindSceneAboutTime> {
        self.time.as_ref()
    }

    pub fn fog(&self) -> Option<&ShaderBindSceneAboutFog> {
        self.fog.as_ref()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShaderSetModelAbout {
    set: u32,
    skin: ESkinCode,
    bind_matrix: u32,
    bind_bone_info: u32,
    bind_bone_texture: u32,
    bind_bone_sampler: u32,
}
impl ShaderSetModelAbout {
    pub fn new(set: u32, skin: ESkinCode) -> Self {
        let mut bind = 0;

        let bind_matrix = bind;
        bind += 1;

        let (bind_bone_size, bind_bone_texture, bind_bone_sampler) = match skin {
            ESkinCode::None => {
                (u32::MAX, u32::MAX, u32::MAX)
            },
            ESkinCode::UBO(_, _) => {
                let result = bind; bind += 1;
                (result, u32::MAX, u32::MAX)
            },
            ESkinCode::RowTexture(_) => {
                let result = bind; bind += 3;
                (result, result + 1, result + 2)
            },
            ESkinCode::FramesTexture(_) => {
                let result = bind; bind += 3;
                (result, result + 1, result + 2)
            },
        };

        Self { 
            set,
            skin,
            bind_matrix,
            bind_bone_info: bind_bone_size,
            bind_bone_texture,
            bind_bone_sampler
        }
    }
    pub fn set(&self) -> u32 {
        self.set
    }
    pub fn bind_matrix(&self) -> u32 {
        self.bind_matrix
    }
    pub fn bind_bone_size(&self) -> u32 {
        self.bind_bone_info
    }
    pub fn bind_bone_texture(&self) -> u32 {
        self.bind_bone_texture
    }
    pub fn bind_bone_sampler(&self) -> u32 {
        self.bind_bone_sampler
    }
    pub fn define_code(&self) -> String {
        let mut result = String::from("");

        result += ShaderSetBind::code_set_bind_head(self.set, self.bind_matrix).as_str();
        result += " Model {\r\n";
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::_WORLD_MATRIX).as_str();
        result += ShaderSetBind::code_uniform("mat4", ShaderVarUniform::_WORLD_MATRIX_INV).as_str();
        result += "};\r\n";

        match self.skin {
            ESkinCode::None => {
                
            },
            ESkinCode::UBO(_, bone) => {
                result += ShaderSetBind::code_set_bind_head(self.set, self.bind_bone_info).as_str();
                result += " Bone {\r\n";
                result += ShaderSetBind::code_uniform_array("mat4", ShaderVarUniform::BONE_MATRICES, bone.count()).as_str();
                result += "};\r\n";

                result += self.skin.define_code().as_str();
            },
            _ => {

                result += ShaderSetBind::code_set_bind_head(self.set, self.bind_bone_info).as_str();
                result += " Bone {\r\n";
                result += ShaderSetBind::code_uniform("vec4", ShaderVarUniform::BONE_TEX_SIZE).as_str();
                result += "};\r\n";
                
                result += ShaderSetBind::code_set_bind_texture2d(self.set, self.bind_bone_texture, ShaderVarUniform::BONE_TEX).as_str();
    
                result += ShaderSetBind::code_set_bind_sampler(self.set, self.bind_bone_sampler, ShaderVarUniform::BONE_TEX).as_str();

                result += self.skin.define_code().as_str();
            },
        }

        result
    }
    
    pub fn running_code(&self) -> String {
        self.skin.running_code()
   }

    pub fn layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        let mut result = vec![
            ShaderBindModelAboutMatrix::layout_entry(self.bind_matrix)
        ];

        match &self.skin {
            ESkinCode::None => {},
            ESkinCode::UBO(_, bone) => {
                ShaderBindModelAboutSkin::layout_entry_ubo(&mut result, self.bind_bone_info, bone);
            },
            ESkinCode::RowTexture(_) => {
                ShaderBindModelAboutSkin::layout_entry_tex(&mut result, self.bind_bone_info, self.bind_bone_texture, self.bind_bone_sampler);
            },
            ESkinCode::FramesTexture(_) => {
                ShaderBindModelAboutSkin::layout_entry_tex(&mut result, self.bind_bone_info, self.bind_bone_texture, self.bind_bone_sampler);
            }
        }

        result
    }

    pub fn bind_group_entries<'a>(
        &'a self,
        dynbuffer: &'a RenderDynUniformBuffer,
        textures: Option<&'a wgpu::TextureView>,
        samplers: Option<&'a Sampler>,
    ) -> Vec<wgpu::BindGroupEntry> {
        let mut entries: Vec<wgpu::BindGroupEntry> = vec![
            wgpu::BindGroupEntry {
                binding: self.bind_matrix,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: dynbuffer.buffer().unwrap(),
                    offset: 0,
                    size: NonZeroU64::new(ShaderBindModelAboutMatrix::TOTAL_SIZE as wgpu::BufferAddress),
                }),
            }
        ];


        match &self.skin {
            ESkinCode::None => {},
            ESkinCode::UBO(_, bone) => {
                entries.push(
                    wgpu::BindGroupEntry {
                        binding: self.bind_bone_info,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: dynbuffer.buffer().unwrap(),
                            offset: 0,
                            size: NonZeroU64::new(bone.use_bytes() as wgpu::BufferAddress),
                        }),
                    }
                );
            },
            _ => {
                entries.push(
                    wgpu::BindGroupEntry {
                        binding: self.bind_bone_info,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: dynbuffer.buffer().unwrap(),
                            offset: 0,
                            size: NonZeroU64::new(ShaderBindModelAboutSkin::TOTAL_SIZE),
                        }),
                    }
                );

                entries.push(
                    wgpu::BindGroupEntry {
                        binding: self.bind_bone_texture,
                        resource: wgpu::BindingResource::TextureView(textures.unwrap()),
                    }
                );
                
                entries.push(
                    wgpu::BindGroupEntry {
                        binding: self.bind_bone_sampler,
                        resource: wgpu::BindingResource::Sampler(samplers.unwrap()),
                    }
                );
            },
        }

        entries
    }

    pub fn label(&self) -> &'static str {
        "ModelAbout"
    }

    pub fn bind_offset(&self, dynbuffer: &mut render_resource::uniform_buffer::RenderDynUniformBuffer, skin: Option<Arc<ShaderBindModelAboutSkin>>) -> ShaderSetModelAboutBindOffset {
        ShaderSetModelAboutBindOffset {
            matrix: ShaderBindModelAboutMatrix::new(self.bind_matrix, dynbuffer),
            skin,
        }
    }
}

#[derive(Debug)]
pub struct ShaderSetModelAboutBindOffset {
    pub(crate) matrix: ShaderBindModelAboutMatrix,
    pub(crate) skin: Option<Arc<ShaderBindModelAboutSkin>>,
}
impl ShaderSetModelAboutBindOffset {
    pub fn get(&self) -> Vec<u32> {
        let mut result: Vec<u32> = vec![
            self.matrix.offset()
        ];

        if let Some(skin) = &self.skin {
            skin.offset(&mut result);
        }

        result
    }

    pub fn matrix(&self) -> &ShaderBindModelAboutMatrix {
        &self.matrix
    }
    
    pub fn skin(&self) -> Option<Arc<ShaderBindModelAboutSkin>> {
        self.skin.clone()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ShaderSetEffectAbout {
    name: Atom,
    set: u32,
    val_size: usize,
    tex_count: u32,
}
impl ShaderSetEffectAbout {
    pub fn new(
        name: Atom,
        set: u32,
        val_size: usize,
        tex_count: u32,
    ) -> Self {
        Self {
            name,
            set,
            val_size,
            tex_count,
        }
    }

    pub fn set(&self) -> u32 {
        self.set
    }

    pub fn tex_count(&self) -> u32 {
        self.tex_count
    }

    pub fn tex_start_bind(&self) -> u32 {
        if self.val_size == 0 { 0 } else { 1 }
    }

    pub fn bind_group_entries<'a>(
        &'a self,
        dynbuffer: &'a RenderDynUniformBuffer,
        textures: &[&'a TextureRes],
        samplers: &[&'a Sampler],
    ) -> Vec<wgpu::BindGroupEntry> {
        let mut entries: Vec<wgpu::BindGroupEntry> = vec![];

        if self.val_size > 0 {
            entries.push(
                wgpu::BindGroupEntry {
                    binding: ShaderBindEffectValue::BIND,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: dynbuffer.buffer().unwrap(),
                        offset: 0,
                        size: NonZeroU64::new(self.val_size as wgpu::BufferAddress),
                    }),
                }
            );
        }

        for i in 0..self.tex_count {
            entries.push(
                wgpu::BindGroupEntry {
                    binding: i as u32 * 2 + 0 + self.tex_start_bind(),
                    resource: wgpu::BindingResource::TextureView(&textures.get(i as usize).unwrap().texture_view),
                }
            );
            
            entries.push(
                wgpu::BindGroupEntry {
                    binding: i as u32 * 2 + 1 + self.tex_start_bind(),
                    resource: wgpu::BindingResource::Sampler(samplers.get(i as usize).unwrap()),
                }
            );
        }

        entries
    }

    pub fn label(&self) -> &'static str {
        "EffectAbout"
    }
}
