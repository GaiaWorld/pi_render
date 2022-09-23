//! 通用图 节点
//!
//! 主要 数据结构：
//!
//!     + trait DependNode 图节点的 对外接口
//!         - 关联类型：Input, Output
//!     + NodeId     节点的id
//!     + NodeLabel  节点标示，可以用 Id 或 String
//!     + ParamUsage 参数的用途
//!
use crate::graph::node::Node;

use super::{
    graph::DependGraph,
    param::{Assign, InParam, OutParam},
    GraphError,
};
use futures::{future::BoxFuture, FutureExt};
use pi_async::rt::AsyncRuntime;
use pi_graph::NGraph;
use pi_hash::{XHashMap, XHashSet};
use pi_share::{cell::TrustCell, Share};
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

/// 图节点，给 外部 扩展 使用
pub trait DependNode: 'static + Send + Sync {
    /// 输入参数
    type Input: InParam + Default;

    /// 输出参数
    type Output: OutParam + Default + Clone;

    /// 构建，当渲染图 构建时候，会调用一次
    /// 一般 用于 准备 渲染 资源的 创建
    /// 如果 无 异步资源创建，可以返回 None
    /// usage 判断 该节点的 输入输出的 用途
    ///     + 判断 输入参数 是否 被前置节点 填充；
    ///     + 判断 输出参数 是否 被后继节点 使用；
    fn build<'a>(&'a self, usage: &'a ParamUsage) -> Option<BoxFuture<'a, Result<(), String>>> {
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
        &'a self,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>>;
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
            v.len() > 0
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

/// 子图：图本身作为一个节点
/// graph成员 只能有一个起始节点，其类型为 Input
/// graph成员 只能有一个finish节点，其类型为 Output
pub struct SubGraph<A, GI, GO>(Arc<TrustCell<SubGraphImpl<A, GI, GO>>>)
where
    A: 'static + AsyncRuntime + Send,
    GI: InParam + OutParam + Default + Clone,
    GO: InParam + OutParam + Default + Clone;

struct SubGraphImpl<A, GI, GO>
where
    A: 'static + AsyncRuntime + Send,
    GI: InParam + OutParam + Default + Clone,
    GO: InParam + OutParam + Default + Clone,
{
    rt: A,
    graph: Option<DependGraph>,

    input: InputNode<GI>,
    output: OutputNode<GO>,

    input_id: NodeId,
    output_id: NodeId,
}

impl<A, GI, GO> SubGraph<A, GI, GO>
where
    A: 'static + AsyncRuntime + Send,
    GI: InParam + OutParam + Default + Clone,
    GO: InParam + OutParam + Default + Clone,
{
    // 输入输入输出
    fn inject_input_outuput(
        g: &mut DependGraph,
        input: InputNode<GI>,
        output: OutputNode<GO>,
    ) -> Result<(NodeId, NodeId), GraphError> {
        let finish_id = match g.get_once_finsh_id() {
            Some(id) => id,
            None => return Err(GraphError::SubGraphOutputError),
        };

        let from_id = g.get_input_nodes();
        let from_id = if from_id.len() != 1 {
            // 子图 输入节点不得多于一个
            return Err(GraphError::SubGraphOutputError);
        } else if let Some(id) = from_id.iter().next() {
            *id
        } else {
            // 子图 输入节点不得为 0
            return Err(GraphError::SubGraphOutputError);
        };

        let input_id = g.add_node("_$pi_m_sub_input$_", input)?;
        let output_id = g.add_node("_$pi_m_sub_output$_", output)?;

        // input -> g.入度为0的节点
        g.add_depend(input_id, "", from_id, "").unwrap();

        // g.finish --> output
        g.add_depend(finish_id, "", output_id, "").unwrap();

        Ok((input_id, output_id))
    }

    /// 创建子图
    pub fn new(rt: A, mut graph: Option<DependGraph>) -> Result<Self, GraphError> {
        let input = InputNode::default();
        let output = OutputNode::default();

        let (input_id, output_id) = if let Some(ref mut g) = graph {
            Self::inject_input_outuput(g, input.clone(), output.clone())?
        } else {
            (NodeId::default(), NodeId::default())
        };

        Ok(Self(Arc::new(TrustCell::new(SubGraphImpl {
            rt,
            graph,

            input,
            output,

            input_id,
            output_id,
        }))))
    }

    /// 更换 异步运行时
    pub fn set_async_runtime(&self, rt: A) {
        self.0.as_ref().borrow_mut().rt = rt;
    }

    /// 更换 运行的子图
    pub fn set_graph(&self, mut g: DependGraph) -> Result<(), GraphError> {
        let mut r = self.0.as_ref().borrow_mut();

        let (i, o) = Self::inject_input_outuput(&mut g, r.input.clone(), r.output.clone())?;

        r.input_id = i;
        r.output_id = o;

        r.graph = Some(g);

        Ok(())
    }

    /// 取目前的子图，以便对子图 做 拓扑修改
    pub fn get_graph(&self) -> Option<DependGraph> {
        let mut r = self.0.as_ref().borrow_mut();

        let input_id = r.input_id;
        let output_id = r.output_id;
        if let Some(ref mut g) = r.graph {
            g.remove_node(input_id).unwrap();
            g.remove_node(output_id).unwrap();
        }

        r.input_id = Default::default();
        r.output_id = Default::default();

        r.graph.take()
    }
}

impl<A, GI, GO> DependNode for SubGraph<A, GI, GO>
where
    A: 'static + AsyncRuntime + Send,
    GI: InParam + OutParam + Default + Clone,
    GO: InParam + OutParam + Default + Clone,
{
    type Input = GI;
    type Output = GO;

    fn build<'a>(&'a self, _usage: &'a ParamUsage) -> Option<BoxFuture<'a, Result<(), String>>> {
        {
            let r = self.0.as_ref().borrow_mut();
            r.graph.as_ref()?;
        }

        Some(
            async move {
                let mut r = self.0.as_ref().borrow_mut();
                let rt = r.rt.clone();
                let g = r.graph.as_mut().unwrap();

                g.build(&rt).await.map_err(|e| e.to_string())
            }
            .boxed(),
        )
    }

    fn run<'a>(
        &'a self,
        input: &'a Self::Input,
        _usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        async move {
            let mut r = self.0.as_ref().borrow_mut();

            // 将 input 扔到 self.input
            *r.input.0.as_ref().borrow_mut() = input.clone();

            let rt = r.rt.clone();
            let output = r.output.clone();

            match r.graph {
                Some(ref mut g) => {
                    match g.run(&rt).await {
                        Ok(_) => {
                            // 将 Output的值拿出来用
                            let output = output.0.as_ref().borrow().clone();
                            Ok(output)
                        }
                        Err(e) => {
                            let msg = format!("sub_graph run_ng, {:?}", e);
                            log::error!("{}", msg);
                            return Err(msg);
                        }
                    }
                }
                None => return Err("sub_graph: no sub_graph".to_string()),
            }
        }
        .boxed()
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

// 渲染节点，给 渲染图 内部 使用
pub(crate) trait InternalNode: OutParam {
    // 当 sub_ng 改变后，需要调用
    fn reset(&mut self);

    // 当 sub_ng 改变后，需要调用
    fn inc_next_refs(&mut self);

    // 添加 前置节点
    fn add_pre_node(&mut self, nodes: (NodeId, NodeState));

    // 每帧 后继的渲染节点 获取参数时候，需要调用 此函数
    fn dec_curr_ref(&mut self);

    // 构建，当渲染图 构建时候，会调用一次
    // 一般 用于 准备 渲染 资源的 创建
    fn build<'a>(&'a self) -> BoxFuture<'a, Result<(), GraphError>>;

    // 执行渲染图
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
    // 当渲染图改变节点的拓扑关系后，需要调用一次
    total_next_refs: i32,

    // 该节点 当前 后继节点数量
    // 每帧 运行 渲染图 前，让它等于  next_refs
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

    fn build<'a>(&'a self) -> BoxFuture<'a, Result<(), GraphError>> {
        async move {
            match self.node.build(&self.param_usage) {
                Some(f) => f.await.map_err(|e| GraphError::CustomBuildError(e)),
                None => Ok(()),
            }
        }
        .boxed()
    }

    fn run<'a>(&'a mut self) -> BoxFuture<'a, Result<(), GraphError>> {
        async move {
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
        }
        .boxed()
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

        let imp = Arc::new(TrustCell::new(imp));

        Self(imp)
    }
}

// ============================ 下面的 结构体 仅供 DependGraph 使用

// 输入节点
#[derive(Clone)]
struct InputNode<I: InParam + OutParam + Default + Clone>(Arc<TrustCell<I>>);

impl<I> InputNode<I>
where
    I: InParam + OutParam + Default + Clone,
{
    fn set_input(&self, data: &I) {
        *self.0.as_ref().borrow_mut() = data.clone();
    }
}

impl<I> Default for InputNode<I>
where
    I: InParam + OutParam + Default + Clone,
{
    fn default() -> Self {
        Self(Arc::new(TrustCell::new(I::default())))
    }
}

impl<I> DependNode for InputNode<I>
where
    I: InParam + OutParam + Default + Clone,
{
    type Input = ();
    type Output = I;

    fn build<'a>(
        &'a self,
        usage: &'a super::node::ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        None
    }

    fn run<'a>(
        &'a self,
        input: &'a Self::Input,
        usage: &'a super::node::ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        let input = self.0.as_ref().borrow().clone();
        async move { Ok(input) }.boxed()
    }
}

// 输出节点
#[derive(Clone)]
struct OutputNode<O: InParam + OutParam + Default + Clone>(Arc<TrustCell<O>>);

impl<O> OutputNode<O>
where
    O: InParam + OutParam + Default + Clone,
{
    fn get_output(&self) -> O {
        let mut p = self.0.as_ref().borrow_mut();
        let r = p.clone();
        *p = Default::default();
        r
    }
}

impl<O> Default for OutputNode<O>
where
    O: InParam + OutParam + Default + Clone,
{
    fn default() -> Self {
        Self(Arc::new(TrustCell::new(O::default())))
    }
}

impl<O> DependNode for OutputNode<O>
where
    O: InParam + OutParam + Default + Clone,
{
    type Input = O;
    type Output = ();

    fn build<'a>(
        &'a self,
        usage: &'a super::node::ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        None
    }

    fn run<'a>(
        &'a self,
        input: &'a Self::Input,
        usage: &'a super::node::ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        // 将 Input 保存起来
        *self.0.as_ref().borrow_mut() = input.clone();

        async move { Ok(()) }.boxed()
    }
}
