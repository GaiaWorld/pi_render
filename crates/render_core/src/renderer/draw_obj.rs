
use std::ops::Range;

use super::{pipeline::{TRenderPipeline}, vertices::{RenderVertices, RenderIndices}, bind_group::BindGroupUsage};

pub trait TBindGroups: Clone {
    fn bindgroups<'a>(&'a self) -> std::slice::Iter<'a, Option<BindGroupUsage>>;
}
pub trait TVerticess: Clone {
    fn value(&self) -> &[RenderVertices];
}
pub trait TInstaces: Clone {
    fn value(&self) -> &[RenderVertices];
}
pub trait TGeometry: Clone {
    fn vertices<'a>(&'a self) -> &[RenderVertices];
    fn instances<'a>(&'a self) -> &[RenderVertices];
    fn indices(&self) -> Option<&RenderIndices>;
}

pub trait TFixedGeometry: Clone {
    fn vertices<'a>(&'a self) -> &[Option<RenderVertices>];
    fn instances(&self) -> Range<u32>;
    fn indices(&self) -> Option<&RenderIndices>;
}

///
/// * MAX_VERTEX_BUFFER : 可能的最大顶点Buffer数目, 本地电脑 16
#[derive(Debug, Clone)]
pub struct DrawObjGeometry<const MAX_VERTEX_BUFFER: usize> {
    pub vertices: [Option<RenderVertices>;MAX_VERTEX_BUFFER],
    pub instances: Range<u32>,
    pub indices: Option<RenderIndices>,
}
impl<const MAX_VERTEX_BUFFER: usize> TFixedGeometry for DrawObjGeometry<MAX_VERTEX_BUFFER> {
    fn vertices(&self) -> &[Option<RenderVertices>] {
        &self.vertices
    }

    fn instances(&self) -> Range<u32> {
        self.instances.clone()
    }

    fn indices(&self) -> Option<&RenderIndices> {
        self.indices.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct DrawObj<T: TRenderPipeline, B: TBindGroups, G: TGeometry> {
    pub pipeline: T,
    pub bindgroups: B,
    pub geometry: G,
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
