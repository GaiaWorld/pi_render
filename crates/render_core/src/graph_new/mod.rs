pub mod node;
pub mod param;

use crate::rhi::{device::RenderDevice, RenderQueue};
use pi_ecs::prelude::World;
use thiserror::Error;

pub use param::*;
pub use node::{NodeId, NodeLabel};

#[derive(Clone)]
pub struct RenderContext {
    // ECS 的 World，用于 查询 渲染数据
    pub world: World,
    // 队列，用于 创建 和 提交 CommandEncoder
    pub queue: RenderQueue,
    // 渲染 设备，用于 创建资源
    pub device: RenderDevice,
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum RenderGraphError {
    #[error("NGGraph is null")]
    NoneNGraph,
    #[error("node does not exist")]
    InvalidNode(NodeLabel),
    #[error("node does not match the given type")]
    WrongNodeType,
    #[error("attempted to connect a node output slot to an incompatible input node slot")]
    MismatchedNodeSlots {
        output_node: NodeId,
        output_slot: usize,
        input_node: NodeId,
        input_slot: usize,
    },
    #[error("node has an unconnected input slot")]
    UnconnectedNodeInputSlot { node: NodeId, input_slot: usize },
    #[error("node has an unconnected output slot")]
    UnconnectedNodeOutputSlot { node: NodeId, output_slot: usize },
    #[error("node input slot already occupied")]
    NodeInputSlotAlreadyOccupied {
        node: NodeId,
        input_slot: usize,
        occupied_by_node: NodeId,
    },

	#[error("Input and output types do not match")]
	MismatchedParam,
}
