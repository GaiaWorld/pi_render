use super::{
    param::{InParam, OutParam},
    GraphError, RenderContext,
};
use futures::{future::BoxFuture, FutureExt};
use pi_share::{cell::TrustCell, Share, ShareRefCell};
use pi_slotmap::new_key_type;
use std::{
    any::TypeId,
    borrow::Cow,
    sync::atomic::{AtomicI32, Ordering},
};
use wgpu::CommandEncoder;

new_key_type! {
    /// 渲染节点 ID
    pub struct NodeId;
}

/// 渲染节点，给 外部 扩展 使用
pub trait Node: 'static + Send + Sync {
    /// 输入参数
    type Input: InParam + Default;

    /// 输出参数
    type Output: OutParam + Default + Clone;

    /// 执行
    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: Share<TrustCell<CommandEncoder>>,
        input: &Self::Input,
    ) -> BoxFuture<'a, Result<Self::Output, String>>;
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

// ====================== crate内 使用的 数据结构

pub(crate) struct NodeState(pub Share<TrustCell<dyn InternalNode>>);

// 渲染节点，给 渲染图 内部 使用
pub(crate) trait InternalNode: OutParam {
    // 当 sub_ng 改变后，需要调用
    fn reset_next_refs(&mut self);

    // 当 sub_ng 改变后，需要调用
    fn inc_next_refs(&mut self);

    // 重置 输出参数为 Default
    fn reset_output(&mut self);

    // 检查 Self 的 输入参数
    fn is_input_match(&mut self, pre_id: NodeId, pre_node: &dyn InternalNode) -> bool;

    /// 填充 Self 的 输入参数
    fn fill_input(&mut self, pre_id: NodeId, pre_node: &dyn InternalNode);

    /// 录制渲染指令
    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
    ) -> BoxFuture<'a, Result<(), GraphError>>;
}

/// 链接 NodeInteral 和 Node 的 结构体
pub(crate) struct GraphNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Input = I, Output = O>,
{
    node: R,
    input: I,
    output: O,

    // 该节点 的后继 节点数量
    // 当渲染图改变节点的拓扑关系后，需要调用一次
    next_refs: i32,

    // 该节点 当前 后继节点数量
    // 每帧 运行 渲染图 前，让它等于  next_refs
    curr_next_refs: AtomicI32,
}

impl<I, O, R> GraphNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Input = I, Output = O>,
{
    pub(crate) fn new(node: R) -> Self {
        Self {
            node,

            input: Default::default(),
            output: Default::default(),

            next_refs: 0,
            curr_next_refs: AtomicI32::new(0),
        }
    }

    // 每帧 后继的渲染节点 获取参数时候，需要调用 此函数
    fn dec_curr_ref(&mut self) {
        // 注：这里 last_count 是 self.curr_next_refs 减1 前 的结果
        let last_count = self.curr_next_refs.fetch_sub(1, Ordering::SeqCst);
        assert!(
            last_count >= 1,
            "RenderNode error, last_count = {}",
            last_count
        );

        // 输出参数重置为 Default
        if last_count == 1 {
            self.imp.reset_output();
        }
    }
}

impl<I, O, R> OutParam for GraphNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Input = I, Output = O>,
{
    fn is_out_macth(&self, ty: TypeId) -> bool {}

    fn fill_to(self, next_id: NodeId, ty: TypeId) {}
}

impl<I, O, R> InternalNode for GraphNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Input = I, Output = O>,
{
    fn reset_next_refs(&mut self) {
        self.next_refs = 0;
    }

    fn inc_next_refs(&mut self) {
        self.next_refs += 1;
    }

    fn reset_output(&mut self) {
        self.output = Default::default();
        self.curr_next_refs.store(self.next_refs, Ordering::SeqCst);
    }

    fn is_input_match(&mut self, pre_id: NodeId, pre_node: &dyn InternalNode) -> bool {
        self.input.is_in_macth(pre_node, TypeId::of::<Self::I>())
    }

    // 每个节点 每帧 需要调用的 入口
    fn fill_input(&mut self, pre_id: NodeId, pre_node: &dyn InternalNode) {}

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
    ) -> BoxFuture<'a, Result<(), String>> {
        async move {
            let runner = self.node.run(context, commands.clone(), &self.input);
            match runner.await {
                Ok(output) => self.output = output,
                Err(msg) => Err(GraphError::RunNodeError(msg)),
            }
        }
        .boxed()
    }
}
