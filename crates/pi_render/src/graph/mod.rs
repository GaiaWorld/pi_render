use pi_ecs::prelude::World;
use thiserror::Error;

use crate::rhi::RenderDevice;

pub struct RenderGraph;

pub(crate) struct RenderGraphRunner;

#[derive(Error, Debug)]
pub enum RenderGraphRunnerError {
    #[error("node run error")]
    NodeRunError,
    #[error("node output slot not set")]
    EmptyNodeOutputSlot,
    #[error("graph could not be run because slot at index has no value")]
    MissingInput,
    #[error("attempted to use the wrong type for input slot")]
    MismatchedInputSlotType,
}

impl RenderGraphRunner {
    pub fn run(
        graph: &RenderGraph,
        render_device: RenderDevice,
        queue: &wgpu::Queue,
        world: &World,
    ) -> Result<(), RenderGraphRunnerError> {
        Ok(())
    }
}
