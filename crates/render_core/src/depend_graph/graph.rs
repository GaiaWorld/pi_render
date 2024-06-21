//!
//! 依赖图
//! + 槽位：add_depend 的 slot_name 如果为 ""，意味着整个参数 关联，不关联某个字段
//!
//! 主要数据结构
//!   
//!   + DependGraph
//!

use super::{
    node::{DependNode, NodeId, NodeLabel, NodeState, ParamUsage}, param::{GraphParamError, InParam, OutParam}, sub_graph_data::RootGraph, GraphError
};
use pi_async_rt::prelude::AsyncRuntime;
use pi_futures::BoxFuture;
use pi_null::Null;
use super::graph_data::NGraph;
use pi_hash::{XHashMap, XHashSet};
use pi_share::ThreadSync;
use pi_slotmap::SlotMap;
use std::{borrow::Cow, mem::transmute};

/// 依赖图
pub struct DependGraph<Context: ThreadSync + 'static> {
	
    // ================== 拓扑信息

    // 名字 和 NodeId 映射
    node_names: XHashMap<Cow<'static, str>, NodeId>,
	// 所有节点
	nodes: SlotMap<NodeId, ScheduleNode<Context>>,
	// 最终节点，渲染到屏幕的节点
	finish_nodes: XHashSet<NodeId>,

	schedule_graph: NGraph<NodeId, ()>, // 派发图， 已经将节点与图， 图与图的连接关系转化为节点与节点的连接关系， 并且，只包含最终到达finish_nodes的子图
	topo_graph: RootGraph<NodeId, ()>, // topo图， 包含节点与子图的链接关系
	is_topo_dirty: bool, // 哪些图的拓扑结构更改了，会放在该列表中
	is_finish_dirty: bool,
	can_run_ndoe_count: usize,
}


impl<Context: ThreadSync + 'static> Default for DependGraph<Context> {
    fn default() -> Self {
        Self {
			schedule_graph: NGraph::new(),
			topo_graph: RootGraph::default(),
            node_names: XHashMap::default(),
			nodes: SlotMap::default(),
			// edge_map: XHashSet::default(),
			finish_nodes: XHashSet::default(),

            // edges: XHashMap::default(),
            // input_node_ids: vec![],

			
			// main_graph: NodeId(main_graph),
			// graphs,
			is_topo_dirty: false,
			is_finish_dirty: false,
            // topo_dirty: Vec::new(),
			can_run_ndoe_count: 0,
        }
    }
	
}

/// 渲染图的 拓扑信息 相关 方法
impl<Context: ThreadSync + 'static> DependGraph<Context> {
	// #[cfg(not(debug_assertions))]
    // pub fn dump_graphviz(&self) -> String {
    //     "".into()
    // }

    /// 将 渲染图 打印成 Graphviz (.dot) 格式
    /// 红色 是 结束 节点
    // #[cfg(debug_assertions)]
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
    // #[cfg(debug_assertions)]
    fn dump_graphviz_impl(&self) -> String {
        use pi_slotmap::Key;

        let mut v = vec!["digraph Render {".into()];

        for id in self.schedule_graph.nodes.keys() {
			let node = &self.nodes[id];
			let name = &node.name;
            let color = if self.finish_nodes.get(&id).is_some() {
                "red"
            } else {
                "white"
            };
			let id1 = id.data();

            v.push(format!(
                "\t \"{id:?}\" [\"style\"=\"filled\" \"label\"={name}_{id1:?} \"fillcolor\"=\"{color}\"]"
            ));
        }

        v.push("".into());

        for (id, n) in self.schedule_graph.nodes.iter() {
			for from in n.from() {
				v.push(format!("\t \"{from:?}\" -> \"{id:?}\""));
			}
        }

        v.push("}".into());

        v.join("\n")
    }

	/// 查 指定节点 的 前驱节点
	pub fn get_prev_ids(&self, id: NodeId) -> Option<&[NodeId]> {
        self.schedule_graph.get(id).map(|v| v.from())
    }

    /// 查 指定节点 的 后继节点
    pub fn get_next_ids(&self, id: NodeId) -> Option<&[NodeId]> {
        self.schedule_graph.get(id).map(|v| v.to())
    }


    /// 添加 名为 name 的 节点
    pub fn add_node<'a, I, O, R>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        node: R,
        parent_graph_id: NodeId,
		is_run: bool,
    ) -> Result<NodeId, GraphError>
    where
        I: InParam + Default,
        O: OutParam + Default + Clone,
        R: DependNode<Context, Input = I, Output = O>,
    {
        self.add(name, node, parent_graph_id, is_run, false)
    }

    /// 添加 名为 name 的 子图
    pub fn add_sub_graph(
        &mut self,
        name: impl Into<Cow<'static, str>>,
    ) -> Result<NodeId, GraphError>
    {
        self.add(name, InternalNodeEmptyImpl, NodeId::null(), false, true)
    }

	/// 设置子图的父, 只能在该图与其他节点创建连接关系之前设置， 否则设置不成功
	pub fn set_sub_graph_parent(&mut self, k: NodeId, parent_graph_id: NodeId) {
		self.topo_graph.set_sub_graph_parent(k, parent_graph_id);
	}

    /// 添加 名为 name 的 节点
    fn add<'a, I, O, R>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        node: R,
        parent_graph_id: NodeId,
		is_run: bool,
        is_sub_graph: bool,
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

        // // 拓扑结构改变
        // self.is_topo_dirty = true;

        let node_state = NodeState::<Context>::new(node);
		let run_node = self.create_run_node(node_state.clone())?;
		let build_node: Box<dyn BuildFuncTrait<Context>> = self.create_build_node(node_state.clone())?;
        let node_id = self.nodes.insert(ScheduleNode {
            build_node,
            run_node,
            name: name.to_string(),
            state: node_state,
			is_run,
			// run_way: RunWay::Schedule,
        });
        if is_sub_graph {
            self.topo_graph.add_sub_graph(node_id, ());
        } else {
            self.topo_graph.add_node(node_id, (), parent_graph_id);
        }
        

        self.node_names.insert(name, node_id);
        self.is_topo_dirty = true;
		
        Ok(node_id)
    }

	// pub fn set_run_way(&mut self, node_id: NodeId, run_way: RunWay) {
	// 	if let Some(node) = self.nodes.get_mut(node_id) {
	// 		node.run_way = run_way;
	// 	}
	// }


    /// 移除 节点或子图
    pub fn remove(&mut self, label: impl Into<NodeLabel>) -> Result<NodeId, GraphError> {
        let label = label.into();

        let id = match self.get_id(&label) {
            Ok(v) => v,
            Err(_) => return Err(GraphError::NoneNode(format!("{:?}", label))),
        };

        self.topo_graph.remove_node(id);
        if let Some(n) = self.nodes.remove(id) {
            self.node_names.remove(n.name.as_str());
			self.is_topo_dirty = true;
			self.finish_nodes.remove(&id);
        }
        
        Ok(id)
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

        let before_node = self.get_id(&before_label)?;
        let after_node = self.get_id(&after_label)?;
        self.topo_graph.add_edge(before_node, after_node);
        self.is_topo_dirty = true;
        Ok(())
    }

	/// 取到入度节点
	pub fn before_nodes(
        &self,
        label: impl Into<NodeLabel>,
    ) -> Result<&[NodeId], GraphError> {
        let label = label.into();
        let node = self.get_id(&label)?;
		match self.topo_graph.before_nodes(node) {
			Some(r) => Ok(r),
			None => Err(GraphError::NoneNode("".to_string())),
		}
    }

	/// 取到出度节点
	pub fn after_nodes(
        &self,
        label: impl Into<NodeLabel>,
    ) -> Result<&[NodeId], GraphError> {
        let label = label.into();
        let node = self.get_id(&label)?;
		match self.topo_graph.after_nodes(node) {
			Some(r) => Ok(r),
			None => Err(GraphError::NoneNode("".to_string())),
		}
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

        let before_node = self.get_id(&before_label)?;
        let after_node = self.get_id(&after_label)?;
        self.topo_graph.remove_edge(before_node, after_node);
        self.is_topo_dirty = true;
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

        let node_id = self.get_id(&label)?;
		if let (Some(_), None) = (self.nodes.get_mut(node_id), self.topo_graph.get_graph(node_id)) {
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
		} else {
			Err(GraphError::NoneNode(format!("{:?}", label)))
		}
    }
}



/// 渲染图的 执行 相关
impl<Context: ThreadSync + 'static> DependGraph<Context> {
    /// 执行 渲染
    pub async fn run<A: 'static + AsyncRuntime + Send>(
        &mut self,
        rt: &A,
        context: &mut Context,
    ) -> Result<(), GraphError> {
		// 运行所有图节点的run方法
		let topological_sort = &self.schedule_graph.topological;
		let mut map = rt.map_reduce(self.can_run_count());
		let context: &Context = context;
		let mut index = 0;
		for node_id in topological_sort.iter() {
			let node = match self.nodes.get(*node_id) {
				Some(r) => r,
				None => panic!("error============={:?}", *node_id),
			};
			if !node.is_run {
				continue;
			}
			let graph_node = self.schedule_graph.get(*node_id).unwrap();
			// 这里用transmute绕过声明周期， 是安全的，因为在context、self释放之前，map中的任务已完成（外部等待）
			map.map(rt.clone(), (*node.run_node)(index, unsafe {transmute(context)}, *node_id, unsafe {transmute(graph_node.from())} ,  unsafe { transmute(graph_node.to())})).unwrap();
			index += 1;
		}
		map.reduce(false).await.unwrap();

		// 重置输入输出参数
		for node_id in topological_sort.iter() {
			let node = &self.nodes[*node_id];
			node.state.0.borrow_mut().clear();
		}
		Ok(())
    }

	/// 构建
    pub fn build(
        &mut self,
        context: &mut Context,
    ) -> Result<(), GraphError> {
		// let is_topo_dirty = self.is_topo_dirty;
		// 检查图是否改变，如果改变， 需要重构图
		let build_ret: Result<(), GraphError> = self.update();
		build_ret?;

		// 运行所有图节点的build方法
		let topological_sort = &self.schedule_graph.topological;
		for node_id in topological_sort.iter() {
			let node = &self.nodes[*node_id];
			let graph_node = self.schedule_graph.get(*node_id).unwrap();
			(*node.build_node)(context, *node_id, &graph_node.from(), &graph_node.to()).unwrap();
		}
		Ok(())
    }

	/// 更新图
    pub fn update(&mut self) -> Result<(), GraphError> {
        self.update_graph()?;

        // 构建 run_ng，返回 构建图
		if self.is_topo_dirty {
			if let Err(GraphError::ParamFillRepeat(f1, f2, t)) = self.update_run_ng() {
                log::error!("param fill with repeat, from: {0:?} {1:?}, to: {2:?}", (f1, &self.nodes[f1].name), (f2, self.nodes.get(f2).map(|r| {&r.name})), (t, &self.nodes[t].name));
                return Err(GraphError::ParamFillRepeat(f1, f2, t));
            }
		}

		// 计算可运行节点的数量
		let mut count = 0;
		for i in self.schedule_graph.topological.iter() {
			if self.nodes[*i].is_run {
				count += 1;
			}
		}
		self.can_run_ndoe_count = count;

		self.is_finish_dirty = false;
		self.is_topo_dirty = false;
        Ok(())
    }

	/// 派发图
	pub fn schedule_graph(&self) -> &NGraph<NodeId, ()> {
		&self.schedule_graph
	}

	/// 节点数量
	pub fn node_count(&self) -> usize {
		self.schedule_graph.topological.len()
	}

	/// 可运行的节点的数量
	pub fn can_run_count(&self) -> usize {
		self.can_run_ndoe_count
	}

    
}

// ================== 以下方法 仅供 crate 使用

impl<Context: ThreadSync + 'static> DependGraph<Context> {
	fn get_id(&self, label: &NodeLabel) -> Result<NodeId, GraphError> {
        match label {
            NodeLabel::NodeId(id) => Ok(*id),
            NodeLabel::NodeName(ref name) => self
                .node_names
                .get(name)
                .cloned()
                .ok_or_else(|| GraphError::NoneNode(label.into())),
        }
    }

    fn update_graph(&mut self) -> Result<bool, GraphError> {
        // 有必要的话，修改 拓扑结构
        if self.is_topo_dirty {
            // 根据节点连接关系，更新拓扑图（用户将节点与节点、节点与子图连接在一起， 需要修改为为节点之间的链接关系）
            self.topo_graph.build().unwrap();
        }

		if self.is_finish_dirty || self.is_topo_dirty {
            // 根据最终节点， 重新生成执行图
			self.schedule_graph = self.topo_graph.gen_graph_from_keys(self.finish_nodes.iter());
		}
		Ok(true)
    }

    // 创建真正的 运行图
    // 返回 构建 的 执行图
    fn update_run_ng(&mut self) -> Result<(), GraphError> {

        // 异步图 节点
        for (_, node) in self.nodes.iter_mut() {
            // 先重置 节点
            node.state.0.as_ref().borrow_mut().reset();
		}

		for id in self.schedule_graph.topological.iter() {
			let graph_node = match self.schedule_graph.get(*id) {
                Some(r) => r,
                None => continue,
            };
			for from in graph_node.from() {
                let from_node = self.nodes.get(*from).unwrap();
                if let Err(r) = self.nodes[*id].state.0.as_ref().borrow_mut().add_pre_node((*from, from_node.state.clone())) {
                    if let GraphParamError::ParamFillRepeat = r {
                        let from = from.clone();
                        // 参数重复, 找到与之冲突的节点， 报告明确的错误
                        self.nodes[*id].state.0.as_ref().borrow_mut().reset();
                        if let Err(GraphParamError::ParamFillRepeat) = self.nodes[*id].state.0.as_ref().borrow_mut().add_pre_node((from, from_node.state.clone())) {
                            // 当前节点本身的输出参数类型重复
                            return Err(GraphError::ParamFillRepeat(from, NodeId::null(), *id));
                        }
                        for from1 in graph_node.from() {
                            if *from1 != from {
                                if let Err(GraphParamError::ParamFillRepeat) = self.nodes[*id].state.0.as_ref().borrow_mut().add_pre_node((from, from_node.state.clone())) {
                                    // 当前节点本身的输出参数类型重复
                                    return Err(GraphError::ParamFillRepeat(from, *from1, *id));
                                }
                            }
                        }
                    }
                }
            }
        }
		Ok(())
    }

    // 创建 构建 节点
    fn create_build_node(
        &self,
        node_state: NodeState<Context>,
    ) -> Result<BuildFunc<Context>, GraphError> {
		let f = move |context: &mut Context, id: NodeId, from: &[NodeId], to: &[NodeId]| -> std::io::Result<()> {
			Ok(node_state.0.as_ref().borrow_mut().build(context, id, from, to).unwrap())
        };
		Ok(Box::new(f))
    }

    // 创建 渲染 节点
    fn create_run_node(
        &self,
		node_state: NodeState<Context>,
        // node_id: NodeId,
    ) -> Result<RunFunc<Context>, GraphError> {
        // 该函数 会在 ng 图上，每帧每节点 执行一次
        let f = move |index: usize, context: &'static Context, id: NodeId, from: &'static [NodeId], to: &'static [NodeId]| -> BoxFuture<'static, std::io::Result<()>> {
            let node_state = node_state.0.clone();
            Box::pin(async move {
                // log::warn!("run graphnode start {:?}", node_id);
                node_state.as_ref().borrow_mut().run(index, context, id, from, to).await.unwrap();
                // log::warn!("run graphnode end {:?}", node_id);
                Ok(())
            })
        };
		Ok(Box::new(f))
    }
}

pub trait BuildFuncTrait<C: ThreadSync + 'static>: Fn(&mut C, NodeId, &[NodeId], &[NodeId]) -> std::io::Result<()> + ThreadSync + 'static {}
impl<Context: ThreadSync + 'static, T: Fn(&mut Context, NodeId, &[NodeId], &[NodeId]) -> std::io::Result<()> + ThreadSync + 'static> BuildFuncTrait<Context> for T {}

pub trait RunFuncTrait<C: ThreadSync + 'static>: Fn(usize, &'static C, NodeId, &'static [NodeId], &'static [NodeId]) -> BoxFuture<'static, std::io::Result<()>> + ThreadSync + 'static{}
impl<Context: ThreadSync + 'static, T: Fn(usize, &'static Context, NodeId, &'static [NodeId], &'static [NodeId]) -> BoxFuture<'static, std::io::Result<()>> + ThreadSync + 'static> RunFuncTrait<Context> for T {}

type BuildFunc<Context> = Box<dyn BuildFuncTrait<Context>>;

type RunFunc<Context> = Box<dyn RunFuncTrait<Context>>;


pub struct InternalNodeEmptyImpl;

impl<Context: ThreadSync + 'static> DependNode<Context> for InternalNodeEmptyImpl {
    type Input = ();

    type Output =();

    fn build<'a>(
        &'a mut self,
        _context: &'a mut Context,
        _input: &'a Self::Input,
        _usage: &'a ParamUsage,
		_id: NodeId, 
		_from: &[NodeId],
		_to: &[NodeId],
    ) -> Result<Self::Output, String> {
        Ok(())
    }

	fn reset<'a>(
        &'a mut self,
    ) {
        
    }

    fn run<'a>(
        &'a mut self,
		_index: usize,
        _context: &'a Context,
        _input: &'a Self::Input,
        _usage: &'a ParamUsage,
		_id: NodeId, 
		_from: &'static [NodeId],
		_to: &'static [NodeId],

    ) -> BoxFuture<'a, Result<(), String>> {
        // async {Ok(())}.
		todo!()
    }
}

struct ScheduleNode<Context: 'static + ThreadSync> {
	build_node: BuildFunc<Context>, // build方法， 如果是图，build为 empty_build
	run_node: RunFunc<Context>,// run方法， 如果是图，run为 empty_run
	name: String, // 节点名字
	state: NodeState<Context>, // 节点状态
	is_run: bool,
	// run_way: RunWay, // 运行方式， 默认为RunWay::Schedule
}


// #[derive(Debug)]
// pub enum RunWay {
// 	Schedule, // 运行方式为图运行时派发
// 	Require(AtomicBool), // 运行方式为外部主动请求运行, AtomicBool表示当前是否正在运行
// }










