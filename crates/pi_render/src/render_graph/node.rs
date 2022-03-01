//! 渲染节点
//!
//! 主要概念
//!
//! + [`Node`] 渲染节点，含: input, output, update, run
//! + [`Edges`] 某个节点的边集，含：入边 和 出边
//! + [`NodeState`] 节点状态，含：Node, name, id, Edges, input_slots, output_slots
//! + [`NodeId`]
//! + [`NodeLabel`] name 或 id

use super::{
    node_slot::{SlotId, SlotInfo, SlotInfos, SlotLabel, SlotValue},
    RenderContext, RenderGraphError,
};
use downcast_rs::{impl_downcast, Downcast};
use futures::future::BoxFuture;
use hash::XHashMap;
use pi_ecs::prelude::World;
use std::{borrow::Cow, cell::RefCell, fmt::Debug, sync::Arc};
use thiserror::Error;

/// 实参
#[derive(Clone, Debug)]
pub struct RealValue(Arc<RefCell<Option<SlotValue>>>);

impl Default for RealValue {

    fn default() -> Self {
        RealValue(Arc::new(RefCell::new(None)))
    }
}

unsafe impl Sync for RealValue {}

unsafe impl Send for RealValue {}

/// 渲染图的节点
/// 可以 异步 执行
pub trait Node: Downcast + Send + Sync + 'static {
    /// 返回 是否 渲染 的 终点，一般是 渲染到屏幕的节点
    /// 注：如果某个节点 改变了 这个属性，需要 重新构建 渲染图
    fn is_finish(&self) -> bool {
        false
    }

    /// 返回 输入 槽位 信息
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    /// 返回 输出 槽位 信息
    fn output(&self) -> Vec<SlotInfo> {
        vec![]
    }

    /// 异步 创建 gpu 资源
    /// 该函数 在 所有节点的 run方法 之前
    /// 存放资源的地方
    /// 资源可以来自 渲染图 之外，也可以来自 渲染节点
    fn prepare(
        &self,
        context: RenderContext,
        inputs: &[Option<RealValue>],
        outputs: &[Option<RealValue>],
    ) -> Option<BoxFuture<'static, Result<(), NodeRunError>>>;

    /// 异步执行 渲染方法
    /// 一个渲染节点，通常是 开始 RenderPass
    /// 将 渲染指令 录制在 wgpu 中
    fn run(
        &self,
        context: RenderContext,
        inputs: &[Option<RealValue>],
        outputs: &[Option<RealValue>],
    ) -> BoxFuture<'static, Result<(), NodeRunError>>;
}

impl_downcast!(Node);

/// 渲染图 运行过程 遇到的 错误
#[derive(Error, Debug, Eq, PartialEq)]
pub enum NodeRunError {
    #[error("encountered an input slot error")]
    InputSlotError,
    #[error("encountered an output slot error")]
    OutputSlotError,
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
    pub node: Arc<dyn Node>,
    /// 输入槽位
    pub input_slots: SlotInfos,
    /// 输出槽位
    pub output_slots: SlotInfos,
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
            node: Arc::new(node),
            type_name: std::any::type_name::<T>(),
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

    /// 取 Slot ID
    pub fn input_slot_id(&self, label: impl Into<SlotLabel>) -> Option<usize> {
        self.input_slots.get_slot_index(label)
    }

    /// 取 Slot ID
    pub fn output_slot_id(&self, label: impl Into<SlotLabel>) -> Option<usize> {
        self.output_slots.get_slot_index(label)
    }
}

/// 渲染节点 ID
pub type NodeId = usize;

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
