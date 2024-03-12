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
    param::{Assign, InParam, OutParam},
    GraphError
};
use pi_futures::BoxFuture;
use pi_hash::{XHashMap, XHashSet};
use pi_share::{Cell, Share, ThreadSync};
use pi_slotmap::new_key_type;
use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicI32, Ordering},
	borrow::Cow,
};

/// 图节点，管理输入输出
pub trait DependNode<Context>: 'static + ThreadSync {
    /// 输入参数
    type Input: InParam + Default;

    /// 输出参数
    type Output: OutParam + Default + Clone;

    // build, 在所有节点的run之前， 都要执行所有节点的build
	// build， 输出节点运行结果（结果一般都是fbo， build先输出一个没有渲染内容的fbo）
	fn build<'a>(
        &'a mut self,
        context: &'a mut Context,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
		id: NodeId, 
		from: &[NodeId],
		to: &[NodeId],
    ) -> Result<Self::Output, String>;

	// 
	fn reset<'a>(
        &'a mut self,
    );

    /// 执行，每帧会调用一次
    /// 执行 run方法之前，会先取 前置节点 相同类型的输出 填充到 input 来
    ///
    /// input-output 生命周期管理：
    ///     run 执行完毕后，input 会 重置 为 Default
    ///     该节点的 所有后继节点 取完 该节点的Output 作为 输入 之后，该节点的 Output 会重置为 Default
    /// usage 的 用法 见 build 方法
	/// -index: 节点在整个图的topo排序的索引
    fn run<'a>(
        &'a mut self,
		index: usize,
        context: &'a Context,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
		id: NodeId, 
		from: &'static [NodeId],
		to: &'static [NodeId],

    ) -> BoxFuture<'a, Result<(), String>>;
}

new_key_type! {
    /// 节点 ID
    pub struct NodeId;
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
    pub(crate) output_usage_set: Share<Cell<XHashSet<TypeId>>>,
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
            output_usage_set: Share::new(Cell::new(Default::default())),
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
pub(crate) trait InternalNode<Context: ThreadSync + 'static>: OutParam {
    // 当 sub_ng 改变后，需要调用
    fn reset(&mut self);

	fn clear(&mut self);

	// 构建结束时调用（指所有出度节点的build方法都调用完成）
	fn build_end(&mut self);

    // 当 sub_ng 改变后，需要调用
    fn inc_next_refs(&mut self);

    // 添加 前置节点
    fn add_pre_node(&mut self, nodes: (NodeId, NodeState<Context>));

    // 每帧 后继的渲染节点 获取参数时候，需要调用 此函数
    fn dec_curr_run_ref(&self);

	// 每帧 后继的渲染节点 获取参数时候，需要调用 此函数
    fn dec_curr_build_ref(&mut self) -> i32;

    // 构建，当依赖图 构建时候，会调用一次
    // 一般 用于 准备 渲染 资源的 创建
    fn build<'a>(&'a mut self, context: &'a mut Context, id: NodeId, from: &[NodeId], to: &[NodeId]) -> Result<(), GraphError>;

    // 执行依赖图
    fn run<'a>(&'a mut self, index: usize, context: &'a Context, id: NodeId, from: &'static [NodeId], to: &'static [NodeId]) -> BoxFuture<'a, Result<(), GraphError>>;
}


/// 链接 NodeInteral 和 DependNode 的 结构体
pub(crate) struct DependNodeImpl<I, O, R, Context>
where
    Context: ThreadSync + 'static,
    I: InParam + Default,
    O: OutParam + Default,
    R: DependNode<Context, Input = I, Output = O>,
{
    node: R,
    input: I,
    output: O,

    context: std::marker::PhantomData<Context>,

    param_usage: ParamUsage,
    pre_nodes: Vec<(NodeId, NodeState<Context>)>,

    // 该节点 的后继 节点数量
    // 当依赖图改变节点的拓扑关系后，需要调用一次
    total_next_refs: i32,

    // 该节点 当前 后继节点数量
    // 每帧 运行 依赖图 前，让它等于  next_refs
    curr_next_refs: AtomicI32,

	curr_next_build_refs: i32,
}

impl<I, O, R, Context> DependNodeImpl<I, O, R, Context>
where
    Context: ThreadSync + 'static,
    I: InParam + Default,
    O: OutParam + Default,
    R: DependNode<Context, Input = I, Output = O>,
{
    pub(crate) fn new(node: R) -> Self {
        Self {
            context: Default::default(),
            node,
            pre_nodes: Default::default(),
            input: Default::default(),
            output: Default::default(),

            param_usage: Default::default(),

            total_next_refs: 0,
            curr_next_refs: AtomicI32::new(0),
			curr_next_build_refs: 0,
        }
    }
}

impl<I, O, R, Context> OutParam for DependNodeImpl<I, O, R, Context>
where
    Context: ThreadSync + 'static,
    I: InParam + Default,
    O: OutParam + Default,
    R: DependNode<Context, Input = I, Output = O>,
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

impl<I, O, R, Context> InternalNode<Context> for DependNodeImpl<I, O, R, Context>
where
    Context: ThreadSync + 'static,
    I: InParam + Default,
    O: OutParam + Default,
    R: DependNode<Context, Input = I, Output = O>,
{
    fn reset(&mut self) {
        self.input = Default::default();
        self.output = Default::default();

        self.param_usage.reset();
        self.pre_nodes.clear();

        self.total_next_refs = 0;
        self.curr_next_refs = AtomicI32::new(0);
    }

	fn clear(&mut self) {
		self.input = Default::default();
        self.output = Default::default();
	}

	fn build_end(&mut self) {
		self.node.reset();
	}

    fn inc_next_refs(&mut self) {
        self.total_next_refs += 1;
    }

    fn add_pre_node(&mut self, node: (NodeId, NodeState<Context>)) {
        node.1 .0.as_ref().borrow_mut().inc_next_refs();

        {
            let n = node.1 .0.as_ref().borrow();
            // 填写 该节点输入 和 前置节点输出 的信息
            self.input
                .can_fill(&mut self.param_usage.input_map_fill, node.0, n.deref());
        }
		
        self.pre_nodes.push(node);
    }

    fn dec_curr_run_ref(&self) {
        // 注：这里 last_count 是 self.curr_next_refs 减1 前 的结果
        let last_count = self.curr_next_refs.fetch_sub(1, Ordering::SeqCst);
        // assert!(
        //     last_count >= 1,
        //     "DependNode error, last_count = {last_count}"
        // );

        if last_count == 1 {
            // SAFE: 此处强转可变，然后清理self.output是安全的
            // curr_next_refs 为 原子操作，在一次图运行过程中， 保证了此处代码仅运行一次
            unsafe { &mut *(self as *const Self as usize as *mut Self) }.output =
                Default::default();
        }
    }

	fn dec_curr_build_ref(&mut self) -> i32 {
		self.curr_next_build_refs -= 1;
		self.curr_next_build_refs
    }

	fn build<'a>(&'a mut self, context: &'a mut Context, id: NodeId, from: &'a [NodeId], to: &'a [NodeId]) -> Result<(), GraphError> {
		for (pre_id, pre_node) in &self.pre_nodes {
			let p = pre_node.0.as_ref();
			let p1 = p.borrow();
			self.input.fill_from(*pre_id, p1.deref());
		}

		let runner = self.node.build(context, &self.input, &self.param_usage, id, from, to);
		match runner {
			Ok(output) => {
				// 结束前，先 重置 引用数
				self.curr_next_build_refs = self.total_next_refs;

				for (_pre_id, pre_node) in &self.pre_nodes {
					let p = pre_node.0.as_ref();
					let mut p = p.borrow_mut();
					// // 用完了 一个前置，引用计数 减 1
					// build阶段不减1，在run中减一
					let cur_count = p.deref_mut().dec_curr_build_ref();
					if cur_count == 0 {
						// SAFE: 此处强转可变是安全的，因为单线程执行build
						p.deref_mut().build_end();
					}
				}

				// 替换 输出
				if self.total_next_refs != 0 {
					self.output = output;
				}

				Ok(())
			}
			Err(msg) => Err(GraphError::CustomRunError(msg)),
		}
    }

    fn run<'a>(&'a mut self, index: usize, context: &'a Context, id: NodeId, from: &'static [NodeId], to: &'static [NodeId]) -> BoxFuture<'a, Result<(), GraphError>> {
        Box::pin(async move {
            let runner = self.node.run(index, context, &self.input, &self.param_usage, id, from, to);

            match runner.await {
                Ok(_output) => Ok(()),
                Err(msg) => Err(GraphError::CustomRunError(msg)),
            }
        })
    }
}

// 节点 状态
pub(crate) struct NodeState<Context: ThreadSync + 'static>(
    pub Share<Cell<dyn InternalNode<Context>>>,
);

impl<Context: ThreadSync + 'static> Clone for NodeState<Context> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}


impl<Context: ThreadSync + 'static> NodeState<Context> {
    pub(crate) fn new<I, O, R>(node: R) -> Self
    where
        I: InParam + Default,
        O: OutParam + Default + Clone,
        R: DependNode<Context, Input = I, Output = O>,
    {
        let imp = DependNodeImpl::new(node);

        let imp = Share::new(Cell::new(imp));

        Self(imp)
    }
}


/// [`NodeLabel`] 用 名字 或者 [`NodeId`] 来 引用 [`NodeState`]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NodeLabel {
    /// 节点 ID 引用
    NodeId(NodeId),
    /// 节点名 引用
    NodeName(Cow<'static, str>),
}

impl From<&NodeLabel> for NodeLabel {
    fn from(value: &NodeLabel) -> Self {
        value.clone()
    }
}

impl From<NodeId> for NodeLabel {
    fn from(value: NodeId) -> Self {
        NodeLabel::NodeId(value)
    }
}

impl From<String> for NodeLabel {
    fn from(value: String) -> Self {
        NodeLabel::NodeName(value.into())
    }
}
impl From<&'static str> for NodeLabel {
    fn from(value: &'static str) -> Self {
        NodeLabel::NodeName(value.into())
    }
}

impl From<&NodeLabel> for String {
    fn from(value: &NodeLabel) -> Self {
        match value {
            NodeLabel::NodeName(value) => value.to_string(),
			NodeLabel::NodeId(id) => {
                format!("{id:?}")
            }
        }
    }
}


