use super::node::NodeId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Edge {
    /// 
    SlotEdge {
        input_node: NodeId,
        input_index: usize,
        output_node: NodeId,
        output_index: usize,
    },
    /// An edge describing to ordering of both nodes (`output_node` before `input_node`).
    NodeEdge {
        input_node: NodeId,
        output_node: NodeId,
    },
}

impl Edge {
    /// Returns the id of the `input_node`.
    pub fn get_input_node(&self) -> NodeId {
        match self {
            Edge::SlotEdge { input_node, .. } => *input_node,
            Edge::NodeEdge { input_node, .. } => *input_node,
        }
    }

    /// Returns the id of the `output_node`.
    pub fn get_output_node(&self) -> NodeId {
        match self {
            Edge::SlotEdge { output_node, .. } => *output_node,
            Edge::NodeEdge { output_node, .. } => *output_node,
        }
    }
}