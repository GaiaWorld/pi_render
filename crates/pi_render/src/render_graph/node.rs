//! 渲染节点
//!
//! 主要概念
//!
//! + [`Node`] 渲染节点，含: input, output, update, run
//! + [`Edges`] 某个节点的边集，含：入边 和 出边
//! + [`NodeState`] 节点状态，含：Node, name, id, Edges, input_slots, output_slots
//! + [`NodeId`]
//! + [`NodeLabel`] name 或 id

use crate::rhi::device::RenderDevice;

use super::{
    edge::Edge,
    node_slot::{SlotInfo, SlotInfos, SlotValue},
    RenderContext, RenderGraphError,
};
use futures::future::BoxFuture;
use hash::XHashMap;
use pi_ecs::prelude::World;
use wgpu::CommandEncoder;
use std::{borrow::Cow, fmt::Debug};
use thiserror::Error;

/// 渲染图的节点
/// 可以 异步 执行
pub trait Node: Send + Sync + 'static {

    /// 返回 是否 渲染 的 终点，一般是 渲染到屏幕的节点
    /// 注：如果某个节点 改变了 这个属性，需要 重新构建 渲染图
    fn is_finish(&self) -> bool;

    /// 返回 输入 槽位 信息
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    /// 返回 输出 槽位 信息
    fn output(&self) -> Vec<SlotInfo> {
        vec![]
    }

    /// 异步执行 渲染方法
    /// 一个渲染节点，通常是 开始 RenderPass
    /// 将 渲染指令 录制在 wgpu 中
    fn run(
        &self,
        context: RenderContext,
        inputs: Vec<SlotInfo>,
        outputs: Vec<Option<SlotInfo>>,
    ) -> BoxFuture<'static, Result<(), NodeRunError>>;
}

/// 渲染图 运行过程 遇到的 错误
#[derive(Error, Debug, Eq, PartialEq)]
pub enum NodeRunError {
    #[error("encountered an input slot error")]
    InputSlotError,
    #[error("encountered an output slot error")]
    OutputSlotError,
}

/// 渲染节点 对应的 边集合
/// 包括 入边 和 出边
#[derive(Debug)]
pub struct Edges {
    pub id: NodeId,
    /// 入边
    pub input_edges: Vec<Edge>,
    /// 出边
    pub output_edges: Vec<Edge>,
}

impl Edges {
    /// 如果对应边 不存在，添加 入边
    pub(crate) fn add_input_edge(&mut self, edge: Edge) -> Result<(), RenderGraphError> {
        if self.has_input_edge(&edge) {
            return Err(RenderGraphError::EdgeAlreadyExists(edge));
        }
        self.input_edges.push(edge);
        Ok(())
    }

    /// 如果对应边 不存在，添加 出边
    pub(crate) fn add_output_edge(&mut self, edge: Edge) -> Result<(), RenderGraphError> {
        if self.has_output_edge(&edge) {
            return Err(RenderGraphError::EdgeAlreadyExists(edge));
        }
        self.output_edges.push(edge);
        Ok(())
    }

    /// 检查有没有对应的 入边
    pub fn has_input_edge(&self, edge: &Edge) -> bool {
        self.input_edges.contains(edge)
    }

    /// 检查有没有对应的 出边
    pub fn has_output_edge(&self, edge: &Edge) -> bool {
        self.output_edges.contains(edge)
    }

    /// 查找 `input_index` 是 index值 的 input_edge
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

    /// 查找 `output_index` 是 index值 的 output_edge
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

/// 节点状态: [`Node`] 内部 表示
///
/// `input_slots`, `output_slots` 都是 由 [`Node`] 提供
/// 注: 节点状态不含值
pub struct NodeState {
    /// 节点 ID
    pub id: NodeId,
    /// 名字
    pub name: Option<Cow<'static, str>>,
    /// 实现 node 的 类型名
    pub type_name: &'static str,
    /// 节点 本身
    pub node: Box<dyn Node>,
    /// 输入槽位
    pub input_slots: SlotInfos,
    /// 输出槽位
    pub output_slots: SlotInfos,
    /// 边
    pub edges: Edges,
}

impl Debug for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?} ({:?})", self.id, self.name)
    }
}

impl NodeState {
    /// 创建 默认 节点状态
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

    /// 取节点值
    pub fn node<T>(&self) -> Result<&T, RenderGraphError>
    where
        T: Node,
    {
        self.node
            .downcast_ref::<T>()
            .ok_or(RenderGraphError::WrongNodeType)
    }

    /// 取节点值
    pub fn node_mut<T>(&mut self) -> Result<&mut T, RenderGraphError>
    where
        T: Node,
    {
        self.node
            .downcast_mut::<T>()
            .ok_or(RenderGraphError::WrongNodeType)
    }

    /// 检查 输入槽的 合法性
    /// 主要是 检查 self 的 输入槽，有没有 对应的 入边
    pub fn validate_input_slots(&self) -> Result<(), RenderGraphError> {
        for i in 0..self.input_slots.len() {
            self.edges.get_input_slot_edge(i)?;
        }

        Ok(())
    }

    /// 检查 输出槽的 合法性
    /// 主要是 检查 self 的 输出槽，有没有 对应的 出边
    pub fn validate_output_slots(&self) -> Result<(), RenderGraphError> {
        for i in 0..self.output_slots.len() {
            self.edges.get_output_slot_edge(i)?;
        }

        Ok(())
    }
}

/// 渲染节点 ID
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NodeId(pub u32);

/// [`NodeLabel`] 用 名字 或者 [`NodeId`] 来 引用 [`NodeState`]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NodeLabel {
    /// 节点 ID 引用
    Id(NodeId),
    /// 节点名 引用
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
