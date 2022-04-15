use float_ord::FloatOrd;
use pi_ecs::entity::Entity;
use pi_render::{
    components::mesh::RenderPipelineKey,
    phase::{
        BatchedPhaseItem, DrawFunctionId, EntityPhaseItem, PhaseItem, RenderPipelinePhaseItem,
    },
};
use std::ops::Range;

pub struct RenderItem {
    pub sort_key: FloatOrd<f32>,
    pub entity: Entity,
    pub pipeline: RenderPipelineKey,
    pub draw_function: DrawFunctionId,
    pub batch_range: Option<Range<u32>>,
}

impl PhaseItem for RenderItem {
    type SortKey = FloatOrd<f32>;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
}

impl EntityPhaseItem for RenderItem {
    #[inline]
    fn entity(&self) -> Entity {
        self.entity
    }
}

impl RenderPipelinePhaseItem for RenderItem {
    #[inline]
    fn cached_pipeline(&self) -> RenderPipelineKey {
        self.pipeline
    }
}

impl BatchedPhaseItem for RenderItem {
    fn batch_range(&self) -> &Option<Range<u32>> {
        &self.batch_range
    }

    fn batch_range_mut(&mut self) -> &mut Option<Range<u32>> {
        &mut self.batch_range
    }
}
