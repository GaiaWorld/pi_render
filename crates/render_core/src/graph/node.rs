//! 渲染节点
//!
//! 主要概念
//!
//! + [`Node`] 渲染节点，含: input, output, update, run
//! + [`NodeState`] 节点状态
//! + [`NodeId`]
//! + [`NodeLabel`] name 或 id

use super::RenderContext;
use downcast_rs::Downcast;
use futures::{future::BoxFuture, FutureExt};
use pi_share::{ShareRefCell, cell::TrustCell};
use pi_slotmap::new_key_type;
use std::{borrow::Cow, fmt::Debug, sync::Arc};
use thiserror::Error;
use wgpu::CommandEncoder;

new_key_type! {
    /// 渲染节点 ID
    pub struct NodeId;
}

/// 输入槽 ID
pub type SlotId = usize;

/// 渲染图的节点
/// 可以 异步 执行
/// 注：为什么 要用 'static

pub trait NodeOutputType: Clone + Default + Send + Sync + 'static {
    
}

impl<T: Clone + Default + Send + Sync + 'static> NodeOutputType for T {

}

pub trait Node: Downcast + Sync + Send + 'static {
    type Output: NodeOutputType;

    /// 异步 创建 gpu 资源
    /// 该函数 在 所有节点的 run方法 之前
    /// 存放资源的地方
    /// 资源可以来自 渲染图 之外，也可以来自 渲染节点
    fn prepare<'a>(
        &'a self,
        _context: RenderContext,
    ) -> Option<BoxFuture<'a, Result<(), NodeRunError>>> {
        None
    }

    /// 每个节点 执行完 run之后，就会 执行 finish
    /// 由 渲染图 runner 进行 加锁，保证内部的 代码可以串行
    fn finish<'a>(
        &'a self,
        _context: RenderContext,
        _inputs: &'a [Self::Output],
    ) -> BoxFuture<'a, Result<(), NodeRunError>> {
        async { Ok(()) }.boxed()
    }

    /// 异步执行 渲染方法
    /// 一个渲染节点，通常是 开始 RenderPass
    /// 将 渲染指令 录制在 wgpu 中
    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
        _inputs: &'a [Self::Output],
    ) -> BoxFuture<'a, Result<Self::Output, NodeRunError>>;
}


/// 渲染图 运行过程 遇到的 错误
#[derive(Error, Debug, Eq, PartialEq)]
pub enum NodeRunError {
    #[error("encountered node depend error")]
    DependError,
}

/// 节点状态: [`Node`] 内部 表示
/// 注: 节点状态不含值
pub struct NodeState<O: NodeOutputType> {
    /// 名字
    pub name: Option<Cow<'static, str>>,
    
    /// 实现 node 的 类型名
    pub type_name: &'static str,

    /// 节点 本身
    pub node: Arc<dyn Node<Output = O>>,

    /// SlotId 对应 对方节点的 inputs 的槽位
    pub output: Vec<(NodeId, SlotId)>,
}

impl<O: NodeOutputType> Debug for NodeState<O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "({:?})", self.name)
    }
}

impl<O: NodeOutputType> NodeState<O> {
    /// 创建 默认 节点状态
    pub fn new<T>(node: T) -> Self
    where
        T: Node<Output = O>,
    {
        NodeState {
            name: None,
            node: Arc::new(node),
            type_name: std::any::type_name::<T>(),
            
            output: vec![],
        }
    }

    /// 清空 输入输出，供 Runner::build 方法调用
    pub(crate) fn clear_input_output(&mut self) {
        self.output.clear();
    }
}

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
