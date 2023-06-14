use std::{marker::PhantomData, hash::Hash, fmt::Debug};

use pi_assets::asset::Asset;
use pi_atom::Atom;

use crate::{asset::ASSET_SIZE_FOR_UNKOWN, render_3d::shader::ERenderAlignment};

use super::{attributes::KeyShaderFromAttributes};


pub type KeyShaderMeta = Atom;

pub trait TKeyShaderSetBlock: Debug + Clone + Hash + PartialEq + Eq + 'static {

}

pub trait TShaderBindCode {
    fn vs_define_code(&self, set: u32) -> String;
    fn fs_define_code(&self, set: u32) -> String;
}

pub trait TShaderSetBlock {
    fn fs_define_code(&self) -> String;
    fn vs_define_code(&self) -> String;
    // fn fs_running_code(&self) -> String;
    // fn vs_running_code(&self) -> String;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyShaderSetBlocks<const MAX_SET_COUNT: usize, K: TKeyShaderSetBlock>(pub [Option<K>; MAX_SET_COUNT]);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct KeyShader<const MAX_SET_COUNT: usize, K: TKeyShaderSetBlock> {
    pub key_meta: KeyShaderMeta,
    pub key_attributes: KeyShaderFromAttributes,
    pub key_set_blocks: KeyShaderSetBlocks<MAX_SET_COUNT, K>,
    pub defines: u128,
    pub renderalignment: ERenderAlignment,
}

#[derive(Debug)]
pub struct Shader<const MAX_SET_COUNT: usize, K: TKeyShaderSetBlock> {
    pub vs: wgpu::ShaderModule,
    pub vs_point: &'static str,
    pub fs: wgpu::ShaderModule,
    pub fs_point: &'static str,
    pub p: PhantomData<K>,
}
impl<const MAX_SET_COUNT: usize, K: TKeyShaderSetBlock> Asset for Shader<MAX_SET_COUNT, K> {
    type Key = KeyShader<MAX_SET_COUNT, K>;
    fn size(&self) -> usize {
        ASSET_SIZE_FOR_UNKOWN
    }
}