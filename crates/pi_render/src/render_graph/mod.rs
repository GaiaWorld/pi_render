pub mod graph;
pub mod node;
pub mod node_slot;
pub mod runner;

use self::node::{NodeId, NodeLabel};
use self::node_slot::SlotLabel;
use crate::rhi::{device::RenderDevice, RenderQueue};
use pi_ecs::prelude::World;
use thiserror::Error;
use wgpu::CommandEncoder;

pub struct RenderContext {
    // ECS 的 World，用于 查询 渲染数据
    pub(crate) world: World,
    // Queue
    pub(crate) queue: RenderQueue,
    // 渲染 设备，用于 创建资源
    pub(crate) device: RenderDevice,
    // GPU 渲染 指令队列
    pub(crate) commands: Option<CommandEncoder>,
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum RenderGraphError {
    #[error("node does not exist")]
    InvalidNode(NodeLabel),
    #[error("output node slot does not exist")]
    InvalidOutputNodeSlot(NodeLabel, SlotLabel),
    #[error("input node slot does not exist")]
    InvalidInputNodeSlot(NodeLabel, SlotLabel),
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
}
