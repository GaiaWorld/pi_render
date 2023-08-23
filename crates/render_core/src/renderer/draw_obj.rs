
use std::{ops::Range, sync::Arc};

use pi_assets::asset::Handle;
use pi_map::smallvecmap::SmallVecMap;
use wgpu::RenderPass;

use crate::rhi::{dyn_uniform_buffer::BufferGroup, asset::RenderRes, bind_group::BindGroup, pipeline::RenderPipeline, shader::{Uniform, BindLayout}};

use super::{vertices::{RenderVertices, RenderIndices}, bind_group::BindGroupUsage};

pub trait TBindGroups: Clone {
    fn bindgroups<'a>(&'a self) -> std::slice::Iter<'a, Option<BindGroupUsage>>;
}

#[derive(Debug)]
pub enum DrawBindGroup {
    Offset(BufferGroup), // 某个buffer的部分 (在全局中的索引， buffer偏移量) 具有动态偏移
    Independ(Handle<RenderRes<BindGroup>>), // 无动态偏移
    GroupUsage(BindGroupUsage),
    Arc(Arc<BindGroup>),
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
            Self::Arc(group) => {
                rpass.set_bind_group(i, group, &[]);
            },
        };
    }

	pub fn set_uniform<T: Uniform>(&mut self, value: &T) {
        let _ = match self {
            DrawBindGroup::Offset(group) => group.set_uniform(value),
            DrawBindGroup::Independ(_group) => todo!(),
			DrawBindGroup::GroupUsage(_group) => todo!(),
			DrawBindGroup::Arc(_group) => todo!(),
        };
    }
}

impl PartialEq for DrawBindGroup {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DrawBindGroup::Offset(a), DrawBindGroup::Offset(b)) => {
                a == b
            },
            (DrawBindGroup::Independ(a), DrawBindGroup::Independ(b)) => {
                a.key() == b.key()
            },
            (DrawBindGroup::GroupUsage(a), DrawBindGroup::GroupUsage(b)) => {
                a.key_bind_group == b.key_bind_group && a.offsets() == b.offsets()
            },
            (DrawBindGroup::Arc(a), DrawBindGroup::Arc(b)) => {
                a.id() == b.id()
            },
            _ => {
                false
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct DrawBindGroups(SmallVecMap<DrawBindGroup, 4>);
impl DrawBindGroups {
    #[inline]
    pub fn get_group(&self, group_id: u32) -> Option<&DrawBindGroup> {
        self.0.get(group_id)
    }

    pub fn insert_group<T: Into<DrawBindGroup>>(&mut self, group_id: u32, value: T) {
        self.0.insert(group_id, value.into());
    }

    #[inline]
    pub fn groups(&self) -> &SmallVecMap<DrawBindGroup, 4> {
        &self.0
    }

	pub fn set_uniform<T: Uniform>(&mut self, value: &T) {
		self.0[T::Binding::set() as usize].set_uniform(value);
    }

    pub fn set<'w, 'a>(&'a self, rpass: &'w mut RenderPass<'a>) {
        for (group, i) in self.0.iter() {
            group.set(rpass, *i);
        }
    }
}

#[derive(Debug)]
pub struct DrawObj {
    pub pipeline: Option<Handle<RenderRes<RenderPipeline>>>,
    pub bindgroups: DrawBindGroups,
    ///
    /// * MAX_VERTEX_BUFFER : 可能的最大顶点Buffer数目, 本地电脑 16
    pub vertices: SmallVecMap<RenderVertices, 3>,
    pub instances: Range<u32>,
    pub vertex: Range<u32>,
    pub indices: Option<RenderIndices>,
}

impl Default for DrawObj {
    fn default() -> Self {
        Self { pipeline: Default::default(), bindgroups: Default::default(), vertices: Default::default(), instances: 0..1, vertex: 0..0, indices: Default::default() }
    }
}

impl DrawObj {
	pub fn insert_vertices(&mut self, vertices: RenderVertices) {
		self.vertices.insert(vertices.slot, vertices);
	}

	pub fn draw<'w, 'a>(&'a self, renderpass: &'w mut RenderPass<'a>) {
		if let Some(pipeline) = &self.pipeline {
			renderpass.set_pipeline(pipeline);
			self.bindgroups.set(renderpass);

			// let mut vertex_range = 0..0;
			let mut v_iter = self.vertices.iter();
			let mut r = v_iter.next();
			// while let Some(item) = r {
			// 	if let Some(item) = item {
			// 		vertex_range = item.value_range();
			// 		break;
			// 	}
			// 	r = v_iter.next();
			// }

			while let Some((item, _)) = r {
				renderpass.set_vertex_buffer(item.slot, item.slice());
				r = v_iter.next();
			}

			let instance_range = self.instances.clone();

			match &self.indices {
				Some(indices) => {
					renderpass.set_index_buffer(indices.slice(), indices.format);
					renderpass.draw_indexed(indices.value_range(), 0 as i32, instance_range);
				},
				None => {
					if !self.vertex.is_empty() {
						renderpass.draw(self.vertex.clone(), instance_range);
					}
				},
			}
		}
	}
}


#[derive(Debug, Default)]
pub(crate) struct TempDrawInfoRecord<'a> {
    vertices: Vec<Option<RenderVertices>>,
    indices: Option<RenderIndices>,
    bindgroups: [Option<&'a DrawBindGroup>;4],
}
impl<'a> TempDrawInfoRecord<'a> {
    pub(crate) fn record_vertex_and_check_diff_with_last(
        &mut self,
        vertex: &RenderVertices,
    ) -> bool {
        if let Some(save) = self.get(vertex.slot as usize) {
            if save == vertex {
                return false;
            } else {
                self.vertices[vertex.slot as usize] = Some(vertex.clone());
                return true;
            }
        } else {
            self.vertices[vertex.slot as usize] = Some(vertex.clone());
            return true;
        }
    }
    pub(crate) fn record_bindgroup_and_check_diff_with_last(
        &mut self,
        slot: usize,
        item: Option<&'a DrawBindGroup>,
    ) -> bool {
        if 4 <= slot {
            return true;
        }
        if let Some(save) = self.bindgroups.get(slot) {
            if *save == item {
                return false;
            } else {
                self.bindgroups[slot] = item;
                return true;
            }
        } else {
            self.bindgroups[slot] = item;
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
        let oldlen = self.vertices.len();
        let mut addcount = 0;
        while oldlen + addcount <= slot {
            self.vertices.push(None);
            addcount += 1;
        }

        self.vertices.get(slot).unwrap().as_ref()
    }
}
