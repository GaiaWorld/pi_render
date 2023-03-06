
use std::ops::Range;

use pi_assets::asset::Handle;
use pi_map::vecmap::VecMap;
use wgpu::RenderPass;

use crate::rhi::{draw_obj::DrawGroup, dyn_uniform_buffer::BufferGroup, asset::RenderRes, bind_group::BindGroup, pipeline::RenderPipeline, buffer::Buffer};

use super::{vertices::{RenderVertices, RenderIndices}, bind_group::BindGroupUsage};

pub trait TBindGroups: Clone {
    fn bindgroups<'a>(&'a self) -> std::slice::Iter<'a, Option<BindGroupUsage>>;
}

#[derive(Debug)]
pub enum DrawBindGroup {
    Offset(BufferGroup), // 某个buffer的部分 (在全局中的索引， buffer偏移量) 具有动态偏移
    Independ(Handle<RenderRes<BindGroup>>), // 无动态偏移
    GroupUsage(BindGroupUsage),
}
impl DrawBindGroup {
    pub fn set<'w, 'a>(&'a self, rpass: &'w mut RenderPass<'a>, i: u32) {
        match self {
            Self::Offset(index) => {
                let group = index.get_group();
                rpass.set_bind_group(i as u32, group.bind_group, group.offsets);
            }
            Self::Independ(group) => {
                rpass.set_bind_group(i as u32, group, &[]);
            },
            Self::GroupUsage(group) => {
                rpass.set_bind_group(group.set, group.bind_group(), &group.offsets());
            },
        };
    }
}

#[derive(Debug, Default)]
pub struct DrawBindGroups(VecMap<DrawBindGroup>);
impl DrawBindGroups {
    #[inline]
    pub fn get_group(&self, group_id: u32) -> Option<&DrawBindGroup> {
        self.0.get(group_id as usize)
    }

    pub fn insert_group(&mut self, group_id: u32, value: DrawBindGroup) {
        self.0.insert(group_id as usize, value);
    }

    #[inline]
    pub fn groups(&self) -> &VecMap<DrawBindGroup> {
        &self.0
    }

    pub fn set<'w, 'a>(&'a self, rpass: &'w mut RenderPass<'a>) {
        let mut i = 0;
        for r in self.0.iter() {
            if let Some(group) = r {
                group.set(rpass, i as u32);
            }
            i += 1;
        }
    }
}

#[derive(Debug)]
pub struct DrawObj {
    pub pipeline: Option<Handle<RenderRes<RenderPipeline>>>,
    pub bindgroups: DrawBindGroups,
    ///
    /// * MAX_VERTEX_BUFFER : 可能的最大顶点Buffer数目, 本地电脑 16
    pub vertices: VecMap<RenderVertices>,
    pub instances: Range<u32>,
    pub indices: Option<RenderIndices>,
}


#[derive(Debug, Default)]
pub(crate) struct TempDrawInfoRecord {
    list: Vec<Option<RenderVertices>>,
    indices: Option<RenderIndices>,
}
impl TempDrawInfoRecord {
    pub(crate) fn record_vertex_and_check_diff_with_last(
        &mut self,
        vertex: &RenderVertices,
    ) -> bool {
        if let Some(save) = self.get(vertex.slot as usize) {
            if save == vertex {
                return false;
            } else {
                self.list[vertex.slot as usize] = Some(vertex.clone());
                return true;
            }
        } else {
            self.list[vertex.slot as usize] = Some(vertex.clone());
            return true;
        }
    }
    pub(crate) fn record_indices_and_check_diff_with_last(
        &mut self,
        indices: &RenderIndices,
    ) -> bool {
        let result = match &self.indices {
            Some(old) => {
                old != indices
            },
            None => {
                true
            },
        };

        self.indices = Some(indices.clone());
        
        result
    }
    fn get(&mut self, slot: usize) -> Option<&RenderVertices> {
        let oldlen = self.list.len();
        let mut addcount = 0;
        while oldlen + addcount <= slot {
            self.list.push(None);
            addcount += 1;
        }

        self.list.get(slot).unwrap().as_ref()
    }
}
