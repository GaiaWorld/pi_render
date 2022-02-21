//! 渲染图对应的边
//!

use super::node::NodeId;

/// 边
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Edge {
    /// 指定 两个槽 的 关系
    /// 执行顺序 (output_node, output_index) 先于 (input_node, input_index)
    SlotEdge {
        /// 输入节点
        input_node: NodeId,
        /// 输入槽
        input_index: usize,
        /// 输出节点
        output_node: NodeId,
        /// 输出槽
        output_index: usize,
    },
    /// 指定 两个节点 的 关系
    /// 执行顺序 output_node 先于 input_node
    NodeEdge {
        /// 输入节点
        input_node: NodeId,
        /// 输出节点
        output_node: NodeId,
    },
}

impl Edge {
    /// 返回 `input_node` 的 id
    pub fn get_input_node(&self) -> NodeId {
        match self {
            Edge::SlotEdge { input_node, .. } => *input_node,
            Edge::NodeEdge { input_node, .. } => *input_node,
        }
    }

    /// 返回 `output_node` 的 id
    pub fn get_output_node(&self) -> NodeId {
        match self {
            Edge::SlotEdge { output_node, .. } => *output_node,
            Edge::NodeEdge { output_node, .. } => *output_node,
        }
    }
}
