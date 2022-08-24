use std::{borrow::Cow, any::TypeId};

use futures::{future::BoxFuture, FutureExt};
use pi_share::{Share, Cell, ShareRefCell};
use pi_slotmap::new_key_type;
use thiserror::Error;
use wgpu::CommandEncoder;

use super::{RenderContext, param::{FillTarget, FillSrc}, RenderGraphError, Param};

new_key_type! {
    /// 渲染节点 ID
    pub struct NodeId;
}

/// 渲染节点
pub trait Node: Send + Sync + 'static {
	type Input: Param + Default;
	type Output: Param + Default;

	/// 异步执行 渲染方法
    /// 一个渲染节点，通常是 开始 RenderPass
    /// 将 渲染指令 录制在 wgpu 中
	fn run(
		&self, 
		context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
        input: &Self::Input,
	) -> Self::Output;

	/// 异步 创建 gpu 资源
    /// 该函数 在 所有节点的 run方法 之前
    /// 存放资源的地方
    /// 资源可以来自 渲染图 之外，也可以来自 渲染节点
	fn prepare<'a>(
        &'a self,
        context: RenderContext,
    ) -> Option<BoxFuture<'a, Result<(), NodeRunError>>> {
        None
    }

    /// 每个节点 执行完 run之后，就会 执行 finish
    /// 由 渲染图 runner 进行 加锁，保证内部的 代码可以串行
    fn finish<'a>(
        &'a self,
        context: RenderContext,
        input: &'a Self::Input,
    ) -> BoxFuture<'a, Result<(), NodeRunError>> {
        async { Ok(()) }.boxed()
    }
}

/// 渲染图 运行过程 遇到的 错误
#[derive(Error, Debug, Eq, PartialEq)]
pub enum NodeRunError {
    #[error("encountered node depend error")]
    DependError,
}

pub trait GraphNode: FillTarget {
	/// 检查参数匹配
	fn check_param_match(
		&self, 
		next_node: &Share<Cell<dyn GraphNode>>,
	) -> Result<(), RenderGraphError>;

	/// 异步执行 渲染方法
    /// 一个渲染节点，通常是 开始 RenderPass
    /// 将 渲染指令 录制在 wgpu 中
	fn run(
		&self, 
		next_nodes: &mut Vec<Share<Cell<dyn GraphNode>>>,
		context: RenderContext,
        commands: ShareRefCell<CommandEncoder>
	);

	/// 异步 创建 gpu 资源
    /// 该函数 在 所有节点的 run方法 之前
    /// 存放资源的地方
    /// 资源可以来自 渲染图 之外，也可以来自 渲染节点
	fn prepare<'a>(
        &'a self,
        context: RenderContext,
    ) -> Option<BoxFuture<'a, Result<(), NodeRunError>>>;

    /// 每个节点 执行完 run之后，就会 执行 finish
    /// 由 渲染图 runner 进行 加锁，保证内部的 代码可以串行
    fn finish<'a>(
        &'a self,
        context: RenderContext,
    ) -> BoxFuture<'a, Result<(), NodeRunError>>;

}

pub struct GraphNodeImpl<I: Param + Default, O: Param + Default + Clone, R: Node<Input=I, Output=O>> {
	input: I,
	node: R,
	id: NodeId,
}

impl<I: Param + Default + Clone, O: Param + Default + Clone, R: Node<Input=I, Output=O> > FillTarget for GraphNodeImpl<I, O, R> {
    unsafe fn fill(&mut self, src_id: NodeId, src_ptr: usize, ty: TypeId) {
        self.input.fill(src_id, src_ptr, ty);
    }

    fn check_macth(&self, ty: TypeId) -> Result<(), RenderGraphError> {
        self.input.check_macth(ty)
    }
}


impl<I: Param + Default + Clone, O: Param + Default + Clone, R: Node<Input=I, Output=O> > GraphNode for GraphNodeImpl<I, O, R> {
	#[inline]
	fn check_param_match(
		&self, 
		next_node: &Share<Cell<dyn GraphNode>>,
	) -> Result<(), RenderGraphError> {
		O::check_macths(&*next_node.borrow())
	}

	#[inline]
	fn run(&self, next_nodes: &mut Vec<Share<Cell<dyn GraphNode>>>, context: RenderContext, mut commands: ShareRefCell<CommandEncoder>) {
		let r = self.node.run(context, commands, &self.input);
		for next in next_nodes.iter() {
			r.clone().fill_to(self.id,&mut *next.borrow_mut())
		}
	}

	#[inline]
	fn prepare<'a>(
        &'a self,
        context: RenderContext,
    ) -> Option<BoxFuture<'a, Result<(), NodeRunError>>> {
		self.node.prepare(context)
	}

	
	fn finish<'a>(
        &'a self,
        context: RenderContext,
    ) -> BoxFuture<'a, Result<(), NodeRunError>>{
		self.node.finish(context, &self.input)
	}
}

// 	input.fill()
// }

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