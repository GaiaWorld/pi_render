//!
//! 依赖图
//! + 槽位：add_depend 的 slot_name 如果为 ""，意味着整个参数 关联，不关联某个字段
//!
//! 主要数据结构
//!   
//!   + DependGraph
//!

use super::{
    node::{DependNode, InternalNode, NodeId, NodeLabel, NodeState},
    param::{InParam, OutParam},
    GraphError,
};
use log::error;
use pi_async_rt::prelude::AsyncRuntime;
use pi_async_graph::{async_graph, ExecNode as ExecNode1, RunFactory, Runner, Runnble, GetRunnble};
use pi_futures::BoxFuture;
use pi_graph::{DirectedGraph, DirectedGraphNode, NGraph, NGraphBuilder, NGraphNode};
use pi_hash::{XHashMap, XHashSet};
use pi_share::{Share, ShareCell, ThreadSync};
use pi_slotmap::SlotMap;
use std::{borrow::Cow, marker::PhantomData, mem::replace};

type ExecNode<Context> = ExecNode1<NodeId, Context, BuildSyncRun<Context>, BuildSyncRun<Context>>;
/// 依赖图
pub struct DependGraph<Context: ThreadSync + 'static> {
    // ================== 拓扑信息

    // 名字 和 NodeId 映射
    node_names: XHashMap<Cow<'static, str>, NodeId>,


    // // 边 (before, after)
    // edges: XHashMap<NodeId, NodeSlot>,

    // 最终节点，渲染到屏幕的节点
    finish_nodes: XHashSet<NodeId>,

    // 有没有 修改 nodes, edges
    is_topo_dirty: bool,

    // 拓扑图，is_topo_dirty 为 false 则 不会 构建
    // 注：ng 的 边 和 edges 的 (before, after) 是 相反的
    topo_graph: NGraphBuilder<NodeId, ()>,

	edge_map: XHashSet<(NodeId, NodeId)>,

    // 有没有 修改 finish_nodes
    is_finish_dirty: bool,

    // ================== 运行信息

    // // 运行图 中 入度为0 的节点
    // input_node_ids: Vec<NodeId>,



	// // 构建执行方法，
	// // 当切仅当 图 有所变化时候，每个节点会重新运行一次；
	// build_runners: SecondaryMap<NodeId, >,
	// // run执行方法
	// // 录制渲染指令的, 用于 更新 GPU 资源
	// // 每帧执行
	// run_runners: SecondaryMap<NodeId, ExecNode<Context, BuildSyncRun<Context>>>,
	
	// 派发任务所依赖的图结构， 与topo_graph不同的是，topo_graph中包含所有节点， schedule_graph只包含需要执行的节点（这些节点必须流向某个终节点）
	schedule_graph: ScheduleGraph<Context, RunType>,


    // // 运行 节点 build 方法的图，当切仅当 图 有所变化时候，每个节点会重新运行一次；
    // // build_ng 如果为Some，这一帧会执行，紧接着 build_ng = None
    // // build_ng 边 和 edges 的 (before, after) 相同
    // build_ng: Option<
    //     Share<NGraph<NodeId, ExecNode<Context, BuildSyncRun<Context>, BuildSyncRun<Context>>>>,
    // >,

    // // 录制渲染指令的 异步执行图，用于 更新 GPU 资源
    // // 当 渲染图 拓扑改变 或 finish 节点 改变后，会 重新 构建个 新的
    // // run_ng边 和 edges 的 (before, after) 相同
    // run_ng: Option<
    //     Share<NGraph<NodeId, ExecNode<Context, BuildSyncRun<Context>, BuildSyncRun<Context>>>>,
    // >,
}

impl<Context: ThreadSync + 'static> Default for DependGraph<Context> {
    fn default() -> Self {
        Self {
            node_names: XHashMap::default(),

            // edges: XHashMap::default(),

            finish_nodes: XHashSet::default(),

            is_topo_dirty: true,

            is_finish_dirty: true,

            // input_node_ids: vec![],

			topo_graph: NGraphBuilder::new(),
			edge_map: XHashSet::default(),
			schedule_graph: ScheduleGraph {
				graph: Default::default(),
				nodes: Default::default(),
			},
        }
    }
}

/// 渲染图的 拓扑信息 相关 方法
impl<Context: ThreadSync + 'static> DependGraph<Context> {
    #[cfg(not(debug_assertions))]
    pub fn dump_graphviz(&self) -> String {
        "".into()
    }

    /// 将 渲染图 打印成 Graphviz (.dot) 格式
    /// 红色 是 结束 节点
    #[cfg(debug_assertions)]
    pub fn dump_graphviz(&self) -> String {
        use log::warn;

        let s = self.dump_graphviz_impl();

        // + Debug 模式
        //     - windwos 非 wasm32 平台，运行目录 生成 dump_graphviz.dot
        //     - 其他 平台，返回 字符串
        // + Release 模式：返回 空串
        #[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
        {
            use std::io::Write;

            match std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open("dump_graphviz.dot")
            {
                Ok(mut file) => match file.write_all(s.as_bytes()) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("write to dump_graphviz.dot error = {:?}", e);
                    }
                },
                Err(e) => {
                    warn!("open dump_graphviz.dot for write error = {:?}", e);
                }
            }
        }
        s
    }

    /// 将 渲染图 打印成 Graphviz (.dot) 格式
    /// 红色 是 结束 节点
    #[cfg(debug_assertions)]
    fn dump_graphviz_impl(&self) -> String {
        let mut v = vec!["digraph Render {".into()];

        for (id, n) in self.schedule_graph.nodes.iter() {
			let name = &n.name;
            let color = if self.finish_nodes.get(&id).is_some() {
                "red"
            } else {
                "white"
            };

            v.push(format!(
                "\t \"{id:?}\" [\"style\"=\"filled\" \"label\"={name} \"fillcolor\"=\"{color}\"]"
            ));
        }

        v.push("".into());

        for (from, to) in self.edge_map.iter() {
			v.push(format!("\t \"{from:?}\" -> \"{to:?}\""));
        }

        v.push("}".into());

        v.join("\n")
    }

    /// 查 指定节点 的 前驱节点
    pub fn get_prev_ids(&self, id: NodeId) -> Option<&[NodeId]> {
        self.topo_graph.graph().get(id).map(|v| v.from())
    }

    /// 查 指定节点 的 后继节点
    pub fn get_next_ids(&self, id: NodeId) -> Option<&[NodeId]> {
        self.topo_graph.graph().get(id).map(|v| v.to())
    }

    /// 添加 名为 name 的 节点
    pub fn add_node<I, O, R>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        node: R,
    ) -> Result<NodeId, GraphError>
    where
        I: InParam + Default,
        O: OutParam + Default + Clone,
        R: DependNode<Context, Input = I, Output = O>,
    {
        let name = name.into();

        // 如果存在同名节点，返回 Err
        if let Some(id) = self.node_names.get(&name) {
            return Err(GraphError::ExitNode(format!("{id:?}")));
        }

        // 拓扑结构改变
        self.is_topo_dirty = true;

        let node_state = NodeState::<Context>::new(node);
		let run_node = self.create_run_node(node_state.clone())?;
		let build_node = self.create_build_node(node_state.clone())?;
        let node_id = self.schedule_graph.nodes.insert(ScheduleNode {
            build_node,
            run_node,
            name: name.to_string(),
            state: node_state,
            mark: PhantomData,
        });

        self.node_names.insert(name, node_id);
		self.topo_graph.node(node_id, ());
		
        Ok(node_id)
    }

    /// 移除 节点
    pub fn remove_node(&mut self, label: impl Into<NodeLabel>) -> Result<(), GraphError> {
        let label = label.into();

        let id = match self.get_node_id(&label) {
            Ok(v) => v,
            Err(_) => return Err(GraphError::NoneNode(format!("{:?}", label))),
        };

        let node = match self.schedule_graph.remove(id) {
            Some(r) => {
				// 拓扑结构改变
				self.is_topo_dirty = true;
				r
			},
            None => return Err(GraphError::NoneNode(format!("{:?}", label))),
        };
        self.finish_nodes.remove(&id);
        self.node_names.remove(node.name.as_str());
        self.topo_graph.remove_node(id);

        Ok(())
    }

    /// 设置 是否 是 最终节点，默认值：false
    /// 出度为0的 Node 并不是 终点
    /// 只有 设置为 true 的 节点 才是 终点
    pub fn set_finish(
        &mut self,
        label: impl Into<NodeLabel>,
        is_finish: bool,
    ) -> Result<(), GraphError> {
        let label = label.into();

        let node_id = self.get_node_id(&label)?;

        if is_finish {
            if !self.finish_nodes.contains(&node_id) {
                // finish 改变
                self.is_finish_dirty = true;

                self.finish_nodes.insert(node_id);
            }
        } else if self.finish_nodes.contains(&node_id) {
            // finish 改变
            self.is_finish_dirty = true;

            self.finish_nodes.remove(&node_id);
        }

        Ok(())
    }

    /// 添加 Node 间 Slot 的 依赖
    /// 执行顺序 `before_label` 先于 `after_label`
    pub fn add_depend(
        &mut self,
        before_label: impl Into<NodeLabel>,
        after_label: impl Into<NodeLabel>,
    ) -> Result<(), GraphError> {
        let before_label = before_label.into();
        let after_label = after_label.into();

        let before_node = self.get_node_id(&before_label)?;
        let after_node: NodeId = self.get_node_id(&after_label)?;

		if self.edge_map.insert((before_node, after_node)) {
			self.topo_graph.edge(before_node, after_node);
			self.is_topo_dirty = true;
		}
		
        Ok(())
    }

    /// 移除 Node 间 Slot 的 依赖
    /// 执行顺序 `before_label` 先于 `after_label`
    pub fn remove_depend(
        &mut self,
        before_label: impl Into<NodeLabel>,
        after_label: impl Into<NodeLabel>,
    ) -> Result<(), GraphError> {
        let before_label = before_label.into();
        let after_label = after_label.into();

        let before_node = self.get_node_id(&before_label)?;
        let after_node = self.get_node_id(&after_label)?;

		self.topo_graph.remove_edge(before_node, after_node);
		// 拓扑结构改变
		self.is_topo_dirty = true;

        Ok(())
    }
}

/// 渲染图的 执行 相关
impl<Context: ThreadSync + 'static> DependGraph<Context> {
    /// 执行 渲染
    pub async fn run<A: 'static + AsyncRuntime + Send>(
        &mut self,
        rt: &A,
        context: &Context,
    ) -> Result<(), GraphError> {
		let is_topo_dirty = self.is_topo_dirty;

		// 检查图是否改变，如果改变， 需要重构图
		let build_ret: Result<(), GraphError> = self.build();
		self.is_finish_dirty = false;
		self.is_topo_dirty = false;
		build_ret?;

		// 运行所有图节点的build方法
        // 只有topo图改变时需要运行一次
		if is_topo_dirty {
			let g = self.schedule_graph.transmute::<BuildType>();
			match async_graph::<_, _, _, _, ExecNode<Context>, _>(rt.clone(), g, context).await {
				Ok(_) => {}
				Err(e) => {
					let err = GraphError::RunNGraphError(format!("run_ng, {e:?}"));

					error!("{}", err);
					return Err(err);
				}
			}
		}

		// 运行所有图节点的run方法
        match async_graph::<_, _, _, _, ExecNode<Context>, _>(rt.clone(), &self.schedule_graph, context).await {
			Ok(_) => Ok(()),
			Err(e) => {
				let err = GraphError::RunNGraphError(format!("run_ng, {e:?}"));

				error!("{}", err);
				Err(err)
			}
		}
    }

	/// 构建图，不需要 运行时
    fn build(&mut self) -> Result<(), GraphError> {
        self.update_graph()?;

        // 构建 run_ng，返回 构建图
		if self.is_topo_dirty {
			self.update_run_ng()?;
		}
        Ok(())
    }
}

// ================== 以下方法 仅供 crate 使用

impl<Context: ThreadSync + 'static> DependGraph<Context> {
    /// 如果 finishes 节点数量 不等于1，返回 None，否则返回 ID
    // #[inline]
    // pub(crate) fn get_once_finsh_id(&mut self) -> Option<NodeId> {
    //     if self.finish_nodes.len() != 1 {
    //         None
    //     } else {
    //         self.finish_nodes.iter().next().copied()
    //     }
    // }

    // /// 根据当前的 finishes 去取 ng 的 入度为0的节点
    // #[inline]
    // pub(crate) fn get_input_nodes(&mut self) -> &[NodeId] {
    //     self.update_topo();

    //     self.input_node_ids.as_slice()
    // }

    fn get_node_id(&self, label: &NodeLabel) -> Result<NodeId, GraphError> {
        match label {
            NodeLabel::Id(id) => Ok(*id),
            NodeLabel::Name(ref name) => self
                .node_names
                .get(name)
                .cloned()
                .ok_or_else(|| GraphError::NoneNode(label.into())),
        }
    }

    fn update_graph(&mut self) -> Result<bool, GraphError> {
        // 有必要的话，修改 拓扑结构
        // 拓扑 没修改，返回 原图
        if self.is_topo_dirty {
			self.topo_graph = NGraphBuilder::new_with_graph(match replace(&mut self.topo_graph, NGraphBuilder::new()).build(){
				Ok(ng) => ng,
				Err(e) => {
					let msg = format!("ng build failed, e = {e:?}");
					return Err(GraphError::BuildError(msg));
				}
			});
        }

		if self.is_finish_dirty || self.is_topo_dirty {
			// 以终为起，构建需要的 节点
			let finishes: Vec<NodeId> = self.finish_nodes.iter().copied().collect();

			self.schedule_graph.graph = self.topo_graph.graph().gen_graph_from_keys(&finishes);
		}
		Ok(true)
    }

    // // 取 label 对应的 Name
    // fn get_node_name(&self, id: NodeId) -> Result<&str, GraphError> {
    //     self.nodes
    //         .get(id)
    //         .map(|v| v.0.as_str())
    //         .ok_or_else(|| GraphError::NoneNode(format!("id = {id:?}")))
    // }

    // // 取 label 对应的 NodeState
    // fn get_node_state(&self, label: &NodeLabel) -> Result<&NodeState<Context>, GraphError> {
    //     self.get_node_id(label).and_then(|id| {
    //         self.schedule_graph.nodes
    //             .get(id)
    //             .map(|v| &v.state)
    //             .ok_or_else(|| GraphError::NoneNode(label.into()))
    //     })
    // }

    // // 根据finish，生成 子图
    // fn gen_sub(&mut self, finish_nodes: graph: &mut NGraph<NodeId, NodeId>) -> Result<NGraph<NodeId, NodeId>, GraphError> {
    //     // 以终为起，构建需要的 节点
    //     let finishes: Vec<NodeId> = self.finish_nodes.iter().copied().collect();

    //     let sub_ng = graph.gen_graph_from_keys(&finishes);

    //     // self.input_node_ids = sub_ng.from().to_vec();

    //     Ok(sub_ng)
    // }

    // // 取 拓扑图，有必要就重新构建
    // fn change_topo(&mut self) -> Result<(), GraphError> {
    //     // 拓扑 没修改，返回 原图
    //     if !self.is_topo_dirty {
    //         return Ok(());
    //     }
    //     self.is_topo_dirty = false;
	// 	Ok(match replace(&mut self.topo_graph, NGraphBuilder::new()).build(){
	// 		Ok(ng) => (),
    //         Err(e) => {
    //             let msg = format!("ng build failed, e = {e:?}");
    //             return Err(GraphError::BuildError(msg));
    //         }
    //     })

    //     // // 构建成功, ng_builder 就 删掉
    //     // let mut builder = NGraphBuilder::<NodeId, NodeId>::new();
    //     // // 节点 就是 高层添加 的 节点
    //     // for (node_id, _) in &self.nodes {
    //     //     builder = builder.node(node_id, node_id);
    //     // }

    //     // {
    //     //     let mut access_edges = XHashSet::<(NodeId, NodeId)>::default();
    //     //     for (before, slot) in self.edges.iter() {
    //     //         for after in slot.next_nodes.iter() {
    //     //             if !access_edges.contains(&(*before, *after)) {
    //     //                 // 顺序 必须和 依赖图顺序 相反
    //     //                 builder = builder.edge(*after, *before);

    //     //                 access_edges.insert((*before, *after));
    //     //             }
    //     //         }
    //     //     }
    //     // }

    //     // let ng = match builder.build() {
    //     //     Ok(ng) => ng,
    //     //     Err(e) => {
    //     //         let msg = format!("ng build failed, e = {e:?}");
    //     //         return Err(GraphError::BuildError(msg));
    //     //     }
    //     // };

    //     // self.topo_graph = Some(ng);
    //     // Ok(())
    // }

    // 创建真正的 运行图
    // 返回 构建 的 执行图
    fn update_run_ng(&mut self) -> Result<(), GraphError> {

        // 异步图 节点
        for (_, node) in &self.schedule_graph.nodes {
            // 先重置 节点
            node.state.0.as_ref().borrow_mut().reset();
		}

		for (id, node) in &self.schedule_graph.nodes {
			let graph_node = self.topo_graph.graph().get(id).unwrap();
			for from in graph_node.from() {
                let from_node = self.schedule_graph.nodes.get(*from).unwrap();
                node.state.0.as_ref().borrow_mut().add_pre_node((*from, from_node.state.clone()));
            }
        }
		Ok(())
    }

    // 创建 构建 节点
    fn create_build_node(
        &self,
        node_state: NodeState<Context>,
    ) -> Result<ExecNode<Context>, GraphError> {
        let node = node_state.0.clone();

        // 该函数 会在 ng 图上，每帧每节点 执行一次
        let f = BuildSyncRun::new(node);

        Ok(ExecNode1::new_sync(f))
    }

    // 创建 渲染 节点
    fn create_run_node(
        &self,
		node_state: NodeState<Context>,
        // node_id: NodeId,
    ) -> Result<ExecNode<Context>, GraphError> {
        // 该函数 会在 ng 图上，每帧每节点 执行一次
        let f = move |context: &'static Context, id: NodeId, from: &'static [NodeId], to: &'static [NodeId]| -> BoxFuture<'static, std::io::Result<()>> {
            let node_state = node_state.0.clone();
            Box::pin(async move {
                // log::warn!("run graphnode start {:?}", node_id);
                node_state.as_ref().borrow_mut().run(context, id, from, to).await.unwrap();
                // log::warn!("run graphnode end {:?}", node_id);
                Ok(())
            })
        };

        Ok(ExecNode1::new_async(Box::new(f)))
    }
}

struct BuildSyncRun<Context: 'static + ThreadSync> {
    node: Share<ShareCell<dyn InternalNode<Context>>>,
    _c: std::marker::PhantomData<Context>,
}

impl<Context: 'static + ThreadSync> Clone for BuildSyncRun<Context> {
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            _c: PhantomData,
        }
    }
}

impl<Context: 'static + ThreadSync> BuildSyncRun<Context> {
    fn new(node: Share<ShareCell<dyn InternalNode<Context>>>) -> Self {
        Self {
            node,
            _c: PhantomData,
        }
    }
}

impl<Context: 'static + ThreadSync> Runner<NodeId, Context> for BuildSyncRun<Context> {
    fn run(self, context: &'static Context, _id: NodeId, _from: &[NodeId], _to: &[NodeId]) {
        self.node.as_ref().borrow_mut().build(context).unwrap();
    }
}

impl<Context: 'static + ThreadSync> RunFactory<NodeId, Context> for BuildSyncRun<Context> {
    type R = BuildSyncRun<Context>;

    fn create(&self) -> Self::R {
        self.clone()
    }
}

struct BuildType;
struct RunType;
struct ScheduleNode<Context: 'static + ThreadSync, T> {
	build_node: ExecNode<Context>,
	run_node: ExecNode<Context>,
	name: String,
	state: NodeState<Context>,
	mark: PhantomData<T>,
}
// impl<T> ScheduleNode<T> {
// 	pub fn transmute<D>(&self) -> &ScheduleNode<D> {
// 		const _: () = [()][(mem::size_of::<Self>() == mem::size_of::<ScheduleNode<D>>()) as usize];
// 		unsafe { std::mem::transmute(self) }
// 	}
// }

impl<Context: 'static + ThreadSync> Runnble<NodeId, Context> for ScheduleNode<Context, BuildType> {
    type R = BuildSyncRun<Context>;

    fn is_sync(&self) -> Option<bool> {
        self.build_node.is_sync()
    }

    fn get_sync(&self) -> Self::R {
        self.build_node.get_sync()
    }

    fn get_async(&self, context: &'static Context, id: NodeId, from: &'static [NodeId], to: &'static [NodeId]) -> BoxFuture<'static, std::io::Result<()>> {
		self.build_node.get_async(context, id, from, to)
    }

    fn load_ready_count(&self) -> usize {
        self.build_node.load_ready_count()
    }

    fn add_ready_count(&self, count: usize) -> usize {
        self.build_node.add_ready_count(count)
    }

    fn store_ready_count(&self, count: usize) {
        self.build_node.store_ready_count(count)
    }
}

impl<Context: 'static + ThreadSync> Runnble<NodeId, Context> for ScheduleNode<Context, RunType> {
    type R = BuildSyncRun<Context>;

    fn is_sync(&self) -> Option<bool> {
        self.run_node.is_sync()
    }

    fn get_sync(&self) -> Self::R {
        self.run_node.get_sync()
    }

    fn get_async(&self, context: &'static Context, id: NodeId, from: &'static [NodeId], to: &'static [NodeId]) -> BoxFuture<'static, std::io::Result<()>> {
		self.run_node.get_async(context, id, from, to)
    }

    fn load_ready_count(&self) -> usize {
        self.run_node.load_ready_count()
    }

    fn add_ready_count(&self, count: usize) -> usize {
		self.run_node.add_ready_count(count)
    }

    fn store_ready_count(&self, count: usize) {
        self.run_node.store_ready_count(count)
    }
}

struct ScheduleGraph<Context: ThreadSync + 'static, T> {
	graph: NGraph<NodeId, ()>,
	nodes: SlotMap<NodeId, ScheduleNode<Context, T>>,
}

impl<Context: ThreadSync + 'static, T> ScheduleGraph<Context, T>{
	pub fn remove(&mut self, id: NodeId) -> Option<ScheduleNode<Context, T>> {
		self.nodes.remove(id)
	}
}

impl<Context: ThreadSync + 'static, T> ScheduleGraph<Context, T>{
	pub fn transmute<D>(&self) -> &ScheduleGraph<Context, D> {
		// const _: () = [()][(std::mem::size_of::<T>() == std::mem::size_of::<D>()) as usize];
		unsafe { std::mem::transmute(self) }
	}
}

impl<Context: ThreadSync + 'static> GetRunnble<NodeId, Context, ExecNode<Context>> for ScheduleGraph<Context, BuildType>{
	fn get_runnble(&self, id: NodeId) -> Option<&ExecNode<Context>> {
		self.nodes.get(id).map(|r| {&r.build_node})
	}
}

impl<Context: ThreadSync + 'static> GetRunnble<NodeId, Context, ExecNode<Context>> for ScheduleGraph<Context, RunType>{
	fn get_runnble(&self, id: NodeId) -> Option<&ExecNode<Context>> {
		self.nodes.get(id).map(|r| {&r.run_node})
	}
}


impl<Context: ThreadSync + 'static, T> DirectedGraph<NodeId, ()> for ScheduleGraph<Context, T> {
    type Node = NGraphNode<NodeId, ()>;

    fn get(&self, key: NodeId) -> Option<&Self::Node> {
        self.graph.get(key)
    }

    fn get_mut(&mut self, key: NodeId) -> Option<&mut Self::Node> {
        self.graph.get_mut(key)
    }

    fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    fn from_len(&self) -> usize {
        self.graph.from_len()
    }

    fn to_len(&self) -> usize {
        self.graph.to_len()
    }

    fn from(&self) -> &[NodeId] {
        self.graph.from()
    }

    fn to(&self) -> &[NodeId] {
        self.graph.to()
    }

    fn topological_sort(&self) -> &[NodeId] {
        self.graph.topological_sort()
    }
}





