
use super::{pipeline::{TRenderPipeline}, vertices::{RenderVertices, RenderIndices}, bind_group::BindGroupUsage};

pub trait TBindGroups: Clone {
    fn bindgroups(&self) -> &[BindGroupUsage];
}
pub trait TVerticess: Clone {
    fn value(&self) -> &[RenderVertices];
}
pub trait TInstaces: Clone {
    fn value(&self) -> &[RenderVertices];
}
pub trait TGeometry: Clone {
    fn vertices(&self) -> &[RenderVertices];
    fn instances(&self) -> &[RenderVertices];
    fn indices(&self) -> Option<&RenderIndices>;
}

pub struct DrawObjGeometry {
    pub vertices: Vec<RenderVertices>,
    pub instances: Vec<RenderVertices>,
    pub indices: Option<RenderIndices>,
}
impl TGeometry for DrawObjGeometry {
    fn vertices(&self) -> &[RenderVertices] {
        &self.vertices
    }

    fn instances(&self) -> &[RenderVertices] {
        &self.instances
    }

    fn indices(&self) -> Option<&RenderIndices> {
        self.indices.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct DrawObj<T: TRenderPipeline, B: TBindGroups, G: TGeometry> {
    pub pipeline: T,
    pub bindgroups: B,
    pub geo: G,
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
