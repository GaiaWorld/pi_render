//! 依赖图 节点
//!
//! 主要 数据结构：
//!
//!     + trait DependNode 图节点的 对外接口
//!         - 关联类型：Input, Output
//!     + NodeId     节点的id
//!     + NodeLabel  节点标示，可以用 Id 或 String
//!     + ParamUsage 参数的用途
//!
use super::{
    graph::DependGraph,
    param::{Assign, InParam, OutParam},
    GraphError,
};
use pi_async::rt::AsyncRuntime;
use pi_futures::BoxFuture;
use pi_hash::{XHashMap, XHashSet};
use pi_share::{cell::TrustCell, Share, ThreadSync};
use pi_slotmap::new_key_type;
use std::{
    any::TypeId,
    borrow::Cow,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicI32, Ordering},
};

/// 图节点，给 外部 扩展 使用
pub trait DependNode: 'static + ThreadSync {
    /// 输入参数
    type Input: InParam + Default;

    /// 输出参数
    type Output: OutParam + Default + Clone;

    /// 准备: 当依赖图 重新构建 之后，第一次调用run之前，会调用一次
    /// 一般 用于 准备 渲染 资源的 创建
    /// 如果 无 异步资源创建，可以返回 None
    /// usage 判断 该节点的 输入输出的 用途
    ///     + 判断 输入参数 是否 被前置节点 填充；
    ///     + 判断 输出参数 是否 被后继节点 使用；
    fn prepare<'a>(
        &'a mut self,
        _usage: &'a ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        None
    }

    /// 执行，每帧会调用一次
    /// 执行 run方法之前，会先取 前置节点 相同类型的输出 填充到 input 来
    ///
    /// input-output 生命周期管理：
    ///     run 执行完毕后，input 会 重置 为 Default
    ///     该节点的 所有后继节点 取完 该节点的Output 作为 输入 之后，该节点的 Output 会重置为 Default
    /// usage 的 用法 见 build 方法
    fn run<'a>(
        &'a mut self,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>>;

    /// 当 依赖图 拓扑结构改变时，第一次调用run之前，会调用一次
    /// 目前，仅计划 给 框架 子图 使用
    /// 外部使用人员 不用 管它
    fn build<'a>(
        &'a mut self,
        _usage: &'a ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        None
    }
}

new_key_type! {
    /// 节点 ID
    pub struct NodeId;
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

impl From<&NodeLabel> for String {
    fn from(value: &NodeLabel) -> Self {
        match value {
            NodeLabel::Name(value) => value.to_string(),
            NodeLabel::Id(id) => {
                format!("{:?}", id)
            }
        }
    }
}

impl From<NodeId> for NodeLabel {
    fn from(value: NodeId) -> Self {
        NodeLabel::Id(value)
    }
}

/// 用于 参数 该节点 参数的 用途
#[derive(Debug)]
pub struct ParamUsage {
    // key =  输入中 被 前置节点 填充的类型
    // value = 填充 该 类型的 前置节点的 ID
    // 注：对 输入收集器，InParamCollector<T>，typeID = T
    // 注：除了 输入收集器，其他输入类型的 Vec 的 大小 只能为 1
    pub(crate) input_map_fill: XHashMap<TypeId, Vec<NodeId>>,

    // 输出 用的到 的 类型
    pub(crate) output_usage_set: Share<TrustCell<XHashSet<TypeId>>>,
}

impl ParamUsage {
    /// ty 作为输入 的 一部分，是否 被 输出 填充
    pub fn is_input_fill(&self, ty: TypeId) -> bool {
        if let Some(v) = self.input_map_fill.get(&ty) {
            !v.is_empty()
        } else {
            // 这时候，ty 不属于 Input 的 字段
            false
        }
    }

    /// ty 作为输出 的 一部分，是否 被 某个后继节点 使用
    pub fn is_output_usage(&self, ty: TypeId) -> bool {
        self.output_usage_set.as_ref().borrow().contains(&ty)
    }
}

// ====================== crate内 使用的 数据结构

impl Default for ParamUsage {
    fn default() -> Self {
        Self {
            output_usage_set: Share::new(TrustCell::new(Default::default())),
            input_map_fill: Default::default(),
        }
    }
}

impl ParamUsage {
    // 当 图 拓扑结构改变，需要 重置
    pub(crate) fn reset(&mut self) {
        self.input_map_fill.clear();
        self.output_usage_set.as_ref().borrow_mut().clear();
    }
}

// 渲染节点，给 依赖图 内部 使用
pub(crate) trait InternalNode: OutParam {
    // 当 sub_ng 改变后，需要调用
    fn reset(&mut self);

    // 当 sub_ng 改变后，需要调用
    fn inc_next_refs(&mut self);

    // 添加 前置节点
    fn add_pre_node(&mut self, nodes: (NodeId, NodeState));

    // 每帧 后继的渲染节点 获取参数时候，需要调用 此函数
    fn dec_curr_ref(&mut self);

    // 构建，当依赖图 构建时候，会调用一次
    // 一般 用于 准备 渲染 资源的 创建
    fn build<'a>(&'a mut self) -> BoxFuture<'a, Result<(), GraphError>>;

    // 执行依赖图
    fn run<'a>(&'a mut self) -> BoxFuture<'a, Result<(), GraphError>>;
}

/// 链接 NodeInteral 和 DependNode 的 结构体
pub(crate) struct DependNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default,
    R: DependNode<Input = I, Output = O>,
{
    node: R,
    input: I,
    output: O,

    param_usage: ParamUsage,

    pre_nodes: Vec<(NodeId, NodeState)>,

    // 该节点 的后继 节点数量
    // 当依赖图改变节点的拓扑关系后，需要调用一次
    total_next_refs: i32,

    // 该节点 当前 后继节点数量
    // 每帧 运行 依赖图 前，让它等于  next_refs
    curr_next_refs: AtomicI32,
}

impl<I, O, R> DependNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default,
    R: DependNode<Input = I, Output = O>,
{
    pub(crate) fn new(node: R) -> Self {
        Self {
            node,
            pre_nodes: Default::default(),
            input: Default::default(),
            output: Default::default(),

            param_usage: Default::default(),

            total_next_refs: 0,
            curr_next_refs: AtomicI32::new(0),
        }
    }
}

impl<I, O, R> OutParam for DependNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default,
    R: DependNode<Input = I, Output = O>,
{
    fn can_fill(&self, set: &mut Option<&mut XHashSet<TypeId>>, ty: TypeId) -> bool {
        assert!(set.is_none());

        let mut p = self.param_usage.output_usage_set.as_ref().borrow_mut();
        self.output.can_fill(&mut Some(p.deref_mut()), ty)
    }

    fn fill_to(&self, this_id: NodeId, to: &mut dyn Assign, ty: TypeId) -> bool {
        self.output.fill_to(this_id, to, ty)
    }
}

impl<I, O, R> InternalNode for DependNodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default,
    R: DependNode<Input = I, Output = O>,
{
    fn reset(&mut self) {
        self.input = Default::default();
        self.output = Default::default();

        self.param_usage.reset();
        self.pre_nodes.clear();

        self.total_next_refs = 0;
        self.curr_next_refs = AtomicI32::new(0);
    }

    fn inc_next_refs(&mut self) {
        self.total_next_refs += 1;
    }

    fn add_pre_node(&mut self, node: (NodeId, NodeState)) {
        node.1 .0.as_ref().borrow_mut().inc_next_refs();

        {
            let n = node.1 .0.as_ref().borrow();
            // 填写 该节点输入 和 前置节点输出 的信息
            self.input
                .can_fill(&mut self.param_usage.input_map_fill, node.0, n.deref());
        }

        self.pre_nodes.push(node);
    }

    fn dec_curr_ref(&mut self) {
        // 注：这里 last_count 是 self.curr_next_refs 减1 前 的结果
        let last_count = self.curr_next_refs.fetch_sub(1, Ordering::SeqCst);
        assert!(
            last_count >= 1,
            "DependNode error, last_count = {}",
            last_count
        );

        if last_count == 1 {
            self.output = Default::default();
        }
    }

    fn build<'a>(&'a mut self) -> BoxFuture<'a, Result<(), GraphError>> {
        Box::pin(async move {
            match self.node.build(&self.param_usage) {
                Some(f) => f.await.map_err(|e| GraphError::CustomBuildError(e)),
                None => Ok(()),
            }
        })
    }

    fn run<'a>(&'a mut self) -> BoxFuture<'a, Result<(), GraphError>> {
        Box::pin(async move {
            for (pre_id, pre_node) in &self.pre_nodes {
                let p = pre_node.0.as_ref();
                let mut p = p.borrow_mut();
                self.input.fill_from(*pre_id, p.deref_mut());

                // 用完了 一个前置，引用计数 减 1
                p.deref_mut().dec_curr_ref();
            }

            let runner = self.node.run(&self.input, &self.param_usage);

            match runner.await {
                Ok(output) => {
                    // 结束前，先 重置 引用数
                    self.curr_next_refs
                        .store(self.total_next_refs, Ordering::SeqCst);

                    // 运行完，重置 输入
                    self.input = Default::default();

                    // 替换 输出
                    self.output = output;

                    Ok(())
                }
                Err(msg) => Err(GraphError::CustomRunError(msg)),
            }
        })
    }
}

// 节点 状态
#[derive(Clone)]
pub(crate) struct NodeState(pub Share<TrustCell<dyn InternalNode>>);

impl NodeState {
    pub(crate) fn new<I, O, R>(node: R) -> Self
    where
        I: InParam + Default,
        O: OutParam + Default + Clone,
        R: DependNode<Input = I, Output = O>,
    {
        let imp = DependNodeImpl::new(node);

        let imp = Share::new(TrustCell::new(imp));

        Self(imp)
    }
}
