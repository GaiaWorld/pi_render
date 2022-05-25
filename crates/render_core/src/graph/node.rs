//! 渲染节点
//!
//! 主要概念
//!
//! + [`Node`] 渲染节点，含: input, output, update, run
//! + [`Edges`] 某个节点的边集，含：入边 和 出边
//! + [`NodeState`] 节点状态，含：Node, name, id, Edges
//! + [`NodeId`]
//! + [`NodeLabel`] name 或 id

use super::{RenderContext, RenderGraphError};
use downcast_rs::{impl_downcast, Downcast};
use futures::{future::BoxFuture, FutureExt};
use pi_ecs::entity::Entity;
use pi_null::Null;
use pi_share::ShareRefCell;
use pi_slotmap::SlotMap;
use wgpu::CommandEncoder;
use std::{borrow::Cow, cell::RefCell, fmt::Debug, sync::Arc};
use thiserror::Error;

/// 渲染图的节点
/// 可以 异步 执行
pub trait Node: Downcast + Send + Sync + 'static {
	type Output: Clone + Default;

    /// 返回 输出 结果
    fn output(&self) -> Self::Output {Self::Output::default()}

    /// 异步 创建 gpu 资源
    /// 该函数 在 所有节点的 run方法 之前
    /// 存放资源的地方
    /// 资源可以来自 渲染图 之外，也可以来自 渲染节点
    fn prepare(
        &self,
        _context: RenderContext,
        _inputs: &[Self::Output],
    ) -> Option<BoxFuture<'static, Result<(), NodeRunError>>> {
        None
    }
	
	fn finish(
        &mut self,
        _context: RenderContext,
        _inputs: &[Self::Output],
    ) -> BoxFuture<'static, Result<(), NodeRunError>> {async {Ok(())}.boxed()}

    /// 异步执行 渲染方法
    /// 一个渲染节点，通常是 开始 RenderPass
    /// 将 渲染指令 录制在 wgpu 中
    fn run(
        &mut self,
        context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
		_inputs: &[Self::Output],
    ) -> BoxFuture<'static, Result<(), NodeRunError>>;
}

// impl_downcast!(Node);

/// 渲染图 运行过程 遇到的 错误
#[derive(Error, Debug, Eq, PartialEq)]
pub enum NodeRunError {
    #[error("encountered an input slot error")]
    InputSlotError,
    #[error("encountered an output slot error")]
    OutputSlotError,
}

/// 节点状态: [`Node`] 内部 表示
/// 注: 节点状态不含值
pub struct NodeState<O> {
    /// 节点 ID
    pub id: NodeId,
    /// 名字
    pub name: Option<Cow<'static, str>>,
    /// 实现 node 的 类型名
    pub type_name: &'static str,
    /// 节点 本身
    pub node: Arc<dyn Node<Output = O>>
}

impl<O> Debug for NodeState<O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?} ({:?})", self.id, self.name)
    }
}

impl<O> NodeState<O> {
    /// 创建 默认 节点状态
    pub fn new<T>(id: NodeId, node: T) -> Self
    where
        T: Node<Output = O>,
    {
        NodeState {
            id,
            name: None,
            node: Arc::new(node),
            type_name: std::any::type_name::<T>(),
        }
    }

    // /// 取节点值
    // pub fn node<T>(&self) -> Result<&T, RenderGraphError>
    // where
    //     T: Node<Output = O>,
    // {
    //     self.node
    //         .downcast_ref::<T>()
    //         .ok_or(RenderGraphError::WrongNodeType)
    // }

    // /// 取节点值
    // pub fn node_mut<T>(&mut self) -> Result<&mut T, RenderGraphError>
    // where
    //     T: Node<Output = O>,
    // {
    //     self.node
    //         .as_any_mut()
    //         .downcast_mut::<T>()
    //         .ok_or(RenderGraphError::WrongNodeType)
    // }
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
