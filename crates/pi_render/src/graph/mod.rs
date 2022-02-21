pub mod context;
pub mod edge;
pub mod graph;
pub mod node;
pub mod node_slot;
pub mod runner;

use self::edge::Edge;
use self::node::{NodeId, NodeLabel};
use self::node_slot::{SlotLabel, SlotValue};
use crate::rhi::device::RenderDevice;
use hash::XHashMap;
use thiserror::Error;
use wgpu::CommandEncoder;

pub struct RenderContext {
    pub device: RenderDevice,
    pub commands: CommandEncoder,
    pub res_mgr: XHashMap<SlotLabel, SlotValue>,
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum RenderGraphError {
    #[error("node does not exist")]
    InvalidNode(NodeLabel),
    #[error("output node slot does not exist")]
    InvalidOutputNodeSlot(SlotLabel),
    #[error("input node slot does not exist")]
    InvalidInputNodeSlot(SlotLabel),
    #[error("node does not match the given type")]
    WrongNodeType,
    #[error("attempted to connect a node output slot to an incompatible input node slot")]
    MismatchedNodeSlots {
        output_node: NodeId,
        output_slot: usize,
        input_node: NodeId,
        input_slot: usize,
    },
    #[error("attempted to add an edge that already exists")]
    EdgeAlreadyExists(Edge),
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
