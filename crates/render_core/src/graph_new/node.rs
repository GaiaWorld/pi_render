use super::{
    param::{InParam, OutParam, Assign},
    GraphError, RenderContext,
};
use futures::{future::BoxFuture, FutureExt};
use pi_share::{cell::TrustCell, Share, ShareRefCell};
use pi_slotmap::new_key_type;
use std::{
    any::TypeId,
    borrow::Cow,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
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
        commands: ShareRefCell<CommandEncoder>,
        input: &'a Self::Input,
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

// 渲染节点，给 渲染图 内部 使用
pub(crate) trait InternalNode: OutParam {
    // 当 sub_ng 改变后，需要调用
    fn inc_next_refs(&mut self);

    // 添加 前置节点
    fn add_pre_node(&mut self, nodes: (NodeId, NodeState));

    // 每帧 后继的渲染节点 获取参数时候，需要调用 此函数
    fn dec_curr_ref(&mut self);

    /// 录制渲染指令
    fn run<'a>(
        &'a mut self,
        context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
    ) -> BoxFuture<'a, Result<(), GraphError>>;
}

/// 链接 NodeInteral 和 Node 的 结构体
pub(crate) struct GraphNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default,
    R: Node<Input = I, Output = O>,
{
    node: R,
    input: I,
    output: O,

    pre_nodes: Vec<(NodeId, NodeState)>,

    // 该节点 的后继 节点数量
    // 当渲染图改变节点的拓扑关系后，需要调用一次
    total_next_refs: i32,

    // 该节点 当前 后继节点数量
    // 每帧 运行 渲染图 前，让它等于  next_refs
    curr_next_refs: AtomicI32,
}

impl<I, O, R> GraphNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default,
    R: Node<Input = I, Output = O>,
{
    pub(crate) fn new(node: R) -> Self {
        Self {
            node,
            pre_nodes: Default::default(),
            input: Default::default(),
            output: Default::default(),

            total_next_refs: 0,
            curr_next_refs: AtomicI32::new(0),
        }
    }
}

impl<I, O, R> OutParam for GraphNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default,
    R: Node<Input = I, Output = O>,
{
    fn fill_to(&self, this_id: NodeId, to: &mut dyn Assign, ty: TypeId) -> bool {
        self.output.fill_to(this_id, to, ty)
    }
}

impl<I, O, R> InternalNode for GraphNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default,
    R: Node<Input = I, Output = O>,
{
    fn inc_next_refs(&mut self) {
        self.total_next_refs += 1;
    }

    fn add_pre_node(&mut self, node: (NodeId, NodeState)) {
        node.1 .0.as_ref().borrow_mut().inc_next_refs();

        self.pre_nodes.push(node);
    }

    fn dec_curr_ref(&mut self) {
        // 注：这里 last_count 是 self.curr_next_refs 减1 前 的结果
        let last_count = self.curr_next_refs.fetch_sub(1, Ordering::SeqCst);
        assert!(
            last_count >= 1,
            "RenderNode error, last_count = {}",
            last_count
        );

        if last_count == 1 {
            self.output = Default::default();
        }
    }

    fn run<'a>(
        &'a mut self,
        context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
    ) -> BoxFuture<'a, Result<(), GraphError>> {
        async move {
            for (pre_id, pre_node) in &self.pre_nodes {
                let p = pre_node.0.as_ref();
                let mut p = p.borrow_mut();
                println!("pre_node id = {:?}", pre_id);
                self.input.fill_from(*pre_id, p.deref_mut());

                // 用完了 一个前置，引用计数 减 1
                p.deref_mut().dec_curr_ref();
            }

            let runner = self.node.run(context, commands.clone(), &self.input);
            match runner.await {
                Ok(output) => {
                    // 结束前，先 重置 引用数
                    self.curr_next_refs
                        .store(self.total_next_refs, Ordering::SeqCst);

                    self.output = output;
                    Ok(())
                }
                Err(msg) => Err(GraphError::RunNodeError(msg)),
            }
        }
        .boxed()
    }
}

#[derive(Clone)]
pub(crate) struct NodeState(pub Share<TrustCell<dyn InternalNode>>);

impl NodeState {
    pub(crate) fn new<I, O, R>(node: R) -> Self
    where
        I: InParam + Default,
        O: OutParam + Default + Clone,
        R: Node<Input = I, Output = O>,
    {
        let imp = GraphNodeImpl::new(node);

        let imp = Arc::new(TrustCell::new(imp));

        Self(imp)
    }
}
