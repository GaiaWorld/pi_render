use super::{
    context::{InputSlotError, OutputSlotError, RenderGraphContext, RunSubGraphError},
    edge::Edge,
    node_slot::{SlotInfo, SlotInfos},
    RenderContext, RenderGraphError,
};
use futures::future::BoxFuture;
use pi_ecs::prelude::World;
use std::{borrow::Cow, fmt::Debug};
use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NodeId(pub u32);

pub trait Node: Send + Sync + 'static {
    fn input(&self) -> Vec<SlotInfo> {
        Vec::new()
    }

    fn output(&self) -> Vec<SlotInfo> {
        Vec::new()
    }

    fn update(&mut self, _world: &mut World) {}

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError>;
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum NodeRunError {
    #[error("encountered an input slot error")]
    InputSlotError(#[from] InputSlotError),
    #[error("encountered an output slot error")]
    OutputSlotError(#[from] OutputSlotError),
    #[error("encountered an error when running a sub-graph")]
    RunSubGraphError(#[from] RunSubGraphError),
}

/// A collection of input and output [`Edges`](Edge) for a [`Node`].
#[derive(Debug)]
pub struct Edges {
    pub id: NodeId,
    pub input_edges: Vec<Edge>,
    pub output_edges: Vec<Edge>,
}

impl Edges {
    /// Adds an edge to the `input_edges` if it does not already exist.
    pub(crate) fn add_input_edge(&mut self, edge: Edge) -> Result<(), RenderGraphError> {
        if self.has_input_edge(&edge) {
            return Err(RenderGraphError::EdgeAlreadyExists(edge));
        }
        self.input_edges.push(edge);
        Ok(())
    }

    /// Adds an edge to the `output_edges` if it does not already exist.
    pub(crate) fn add_output_edge(&mut self, edge: Edge) -> Result<(), RenderGraphError> {
        if self.has_output_edge(&edge) {
            return Err(RenderGraphError::EdgeAlreadyExists(edge));
        }
        self.output_edges.push(edge);
        Ok(())
    }

    /// Checks whether the input edge already exists.
    pub fn has_input_edge(&self, edge: &Edge) -> bool {
        self.input_edges.contains(edge)
    }

    /// Checks whether the output edge already exists.
    pub fn has_output_edge(&self, edge: &Edge) -> bool {
        self.output_edges.contains(edge)
    }

    /// Searches the `input_edges` for a [`Edge::SlotEdge`],
    /// which `input_index` matches the `index`;
    pub fn get_input_slot_edge(&self, index: usize) -> Result<&Edge, RenderGraphError> {
        self.input_edges
            .iter()
            .find(|e| {
                if let Edge::SlotEdge { input_index, .. } = e {
                    *input_index == index
                } else {
                    false
                }
            })
            .ok_or(RenderGraphError::UnconnectedNodeInputSlot {
                input_slot: index,
                node: self.id,
            })
    }

    /// Searches the `output_edges` for a [`Edge::SlotEdge`],
    /// which `output_index` matches the `index`;
    pub fn get_output_slot_edge(&self, index: usize) -> Result<&Edge, RenderGraphError> {
        self.output_edges
            .iter()
            .find(|e| {
                if let Edge::SlotEdge { output_index, .. } = e {
                    *output_index == index
                } else {
                    false
                }
            })
            .ok_or(RenderGraphError::UnconnectedNodeOutputSlot {
                output_slot: index,
                node: self.id,
            })
    }
}

/// The internal representation of a [`Node`], with all data required
/// by the [`RenderGraph`](super::RenderGraph).
///
/// The `input_slots` and `output_slots` are provided by the `node`.
pub struct NodeState {
    pub id: NodeId,
    pub name: Option<Cow<'static, str>>,
    /// The name of the type that implements [`Node`].
    pub type_name: &'static str,
    pub node: Box<dyn Node>,
    pub input_slots: SlotInfos,
    pub output_slots: SlotInfos,
    pub edges: Edges,
}

impl Debug for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?} ({:?})", self.id, self.name)
    }
}

impl NodeState {
    /// Creates an [`NodeState`] without edges, but the `input_slots` and `output_slots`
    /// are provided by the `node`.
    pub fn new<T>(id: NodeId, node: T) -> Self
    where
        T: Node,
    {
        NodeState {
            id,
            name: None,
            input_slots: node.input().into(),
            output_slots: node.output().into(),
            node: Box::new(node),
            type_name: std::any::type_name::<T>(),
            edges: Edges {
                id,
                input_edges: Vec::new(),
                output_edges: Vec::new(),
            },
        }
    }

    /// Retrieves the [`Node`].
    pub fn node<T>(&self) -> Result<&T, RenderGraphError>
    where
        T: Node,
    {
        self.node
            .downcast_ref::<T>()
            .ok_or(RenderGraphError::WrongNodeType)
    }

    /// Retrieves the [`Node`] mutably.
    pub fn node_mut<T>(&mut self) -> Result<&mut T, RenderGraphError>
    where
        T: Node,
    {
        self.node
            .downcast_mut::<T>()
            .ok_or(RenderGraphError::WrongNodeType)
    }

    /// Validates that each input slot corresponds to an input edge.
    pub fn validate_input_slots(&self) -> Result<(), RenderGraphError> {
        for i in 0..self.input_slots.len() {
            self.edges.get_input_slot_edge(i)?;
        }

        Ok(())
    }

    /// Validates that each output slot corresponds to an output edge.
    pub fn validate_output_slots(&self) -> Result<(), RenderGraphError> {
        for i in 0..self.output_slots.len() {
            self.edges.get_output_slot_edge(i)?;
        }

        Ok(())
    }
}

/// A [`NodeLabel`] is used to reference a [`NodeState`] by either its name or [`NodeId`]
/// inside the [`RenderGraph`](super::RenderGraph).
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NodeLabel {
    Id(NodeId),
    Name(Cow<'static, str>),
}

impl From<&NodeLabel> for NodeLabel {
    fn from(value: &NodeLabel) -> Self {
        value.clone()
    }
}

impl From<String> for NodeLabel {
    fn from(value: String) -> Self {
        NodeLabel::Name(value.into())
    }
}

impl From<&'static str> for NodeLabel {
    fn from(value: &'static str) -> Self {
        NodeLabel::Name(value.into())
    }
}

impl From<NodeId> for NodeLabel {
    fn from(value: NodeId) -> Self {
        NodeLabel::Id(value)
    }
}
