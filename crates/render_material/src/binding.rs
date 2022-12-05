use pi_share::Share;
use render_data_container::{TMaterialBlockKindKey, Vector2, Vector4, Matrix};

use crate::{material::{UniformDesc, UniformKindFloat, UniformKindFloat2, UniformKindFloat4, UniformKindColor4, UniformKindMat2, UniformKindMat4, EUniformDataFormat, UnifromData}, error::EMaterialError};

pub struct BindingDesc<MBKK: TMaterialBlockKindKey> {
    pub uniforms: Vec<UniformDesc<MBKK>>,
    pub size: u32,
    pub id: usize,
}

pub struct BindingData<MBKK: TMaterialBlockKindKey> {
    uniform_kinds: Vec<MBKK>,
    uniform_formats: Vec<EUniformDataFormat>,
    /// 每个 Uniform 在该 bind 中字节偏移
    uniform_offset: Vec<usize>,
    /// 整个 bind 在 存储时的字节偏移
    bind_offset: usize,
    dirty: bool,
}

impl<MBKK: TMaterialBlockKindKey> Default for BindingData<MBKK> {
    fn default() -> Self {
        Self {
            uniform_kinds: vec![],
            uniform_formats: vec![],
            uniform_offset: vec![],
            dirty: true,
            bind_offset: 0,
        }
    }
}

impl<MBKK> BindingData<MBKK>
    where
        MBKK: TMaterialBlockKindKey,
{
    ///
    /// uniform_descs 创建时应当进行排序
    pub fn init(
        &mut self,
        bind: &BindingDesc<MBKK>,
        bind_offset: usize,
    ) {
        bind.uniforms.iter().for_each(|desc| {
            self.uniform_kinds.push(desc.kind.clone());
            self.uniform_formats.push(desc.format);
            self.uniform_offset.push(desc.byte_offset_in_bind);
        });
        self.bind_offset = bind_offset;
    }

    pub fn modify(
        &mut self,
        kind: MBKK,
        data: UnifromData,
        data_buffer: &mut Vec<u8>,
    ) -> Result<(), EMaterialError> {
        match self.uniform_kinds.binary_search(&kind) {
            Ok(index) => {
                match self.uniform_formats.get(index) {
                    Some(format) => {
                        match self.uniform_offset.get(index) {
                            Some(offset) => {
                                self.dirty = true;
                                data.to_data(data_buffer, *offset, format)
                            },
                            None => Err(EMaterialError::NotSupportUniformDesc),
                        }
                    },
                    None => {
                        Err(EMaterialError::NotSupportUniformDesc)
                    },
                }
            },
            Err(_) => {
                Err(EMaterialError::NotSupportUniformDesc)
            },
        }
    }

    pub fn dirty(
        &self,
    ) -> bool {
        self.dirty
    }
}