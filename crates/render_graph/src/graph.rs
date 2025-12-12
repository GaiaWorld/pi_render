//!
//! 依赖图
//! + 槽位：add_depend 的 slot_name 如果为 ""，意味着整个参数 关联，不关联某个字段
//!
//! 主要数据结构
//!   
//!   + DependGraph
//!

use super::{
    node::{DependNode, NodeId, NodeLabel, NodeState}, sub_graph_data::RootGraph, GraphError
};
use pi_async_rt::prelude::AsyncRuntime;
use pi_futures::BoxFuture;
use pi_null::Null;
use super::graph_data::NGraph;
use pi_hash::{XHashMap, XHashSet};
use pi_share::ThreadSync;
use pi_slotmap::{Key, SlotMap};
use std::{borrow::Cow, mem::transmute};

/// 依赖图
pub struct DependGraph<Context: ThreadSync + 'static, DataId: Key + ThreadSync> {
	
    // ================== 拓扑信息

    // 名字 和 NodeId 映射
    node_names: XHashMap<Cow<'static, str>, NodeId>,
	// 所有节点
	nodes: SlotMap<NodeId, ScheduleNode<Context, DataId>>,
	// 最终节点，渲染到屏幕的节点
	finish_nodes: XHashSet<NodeId>,

	schedule_graph: NGraph<NodeId, ()>, // 派发图， 已经将节点与图， 图与图的连接关系转化为节点与节点的连接关系， 并且，只包含最终到达finish_nodes的子图
	topo_graph: RootGraph<NodeId, ()>, // topo图， 包含节点与子图的链接关系
	is_topo_dirty: bool, // 哪些图的拓扑结构更改了，会放在该列表中
	is_finish_dirty: bool,
    is_enable_dirty: bool,
	can_run_node: Vec<NodeId>,
    build_nodes: Vec<NodeId>,
    need_init_nodes: Vec<NodeId>,

    main_graph_id: NodeId, // 主图id
}


impl<Context: ThreadSync + 'static, DataId: Key + ThreadSync> Default for DependGraph<Context, DataId> {
    fn default() -> Self {
        let mut r = Self {
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
            is_enable_dirty: false,
            // topo_dirty: Vec::new(),
			can_run_node: Vec::new(),
            build_nodes: Vec::new(),
            need_init_nodes: Vec::new(),
            main_graph_id: Default::default(),
        };
        r.main_graph_id = r.add_sub_graph("main_graph").unwrap();
        r
    }
	
}

/// 渲染图的 拓扑信息 相关 方法
impl<Context: ThreadSync + 'static, DataId: Key + ThreadSync> DependGraph<Context, DataId> {
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

    /// toop图
    pub fn dump_toop_graphviz(&self) -> String {

        use pi_slotmap::Key;

        let mut v = vec!["digraph Render {".into()];

        for (id, n) in self.topo_graph.nodes.iter() {
			let node = &self.nodes[id];
			let name = &node.name;
            let color = if self.finish_nodes.get(&id).is_some() {
                "red"
            } else {
                "white"
            };
			let id1 = id.data();
            let enable = if n.is_enable {1}else {0};
            let transfer = if n.is_transfer{1}else{0};
            let build = if node.is_build {1} else {0};
            let run = if node.is_run{1} else {0};
            let entity = node.data_id;

            

            if let Some(_r) = self.topo_graph.sub_graphs.get(id) {
                v.push(format!(
                    "\t \"{id:?}_out\" [\"style\"=\"filled\" \"label\"=\"{name}_{id1:?}_enable_{enable:?}\" \"fillcolor\"=\"green\"]"
                ));
                v.push(format!(
                    "\t \"{id:?}_in\" [\"style\"=\"filled\" \"label\"=\"{name}_{id1:?}_enable_{enable:?}\" \"fillcolor\"=\"green\"]"
                ));
            } else {
                v.push(format!(
                    "\t \"{id:?}\" [\"style\"=\"filled\" \"label\"=\"{name}_{id1:?}_e{enable:?}_t{transfer:?}_b{build:?}_r{run:?}_d{entity:?}\" \"fillcolor\"=\"{color}\"]"
                ));
            }
        }

        v.push("".into());

        for (id, _n) in self.topo_graph.nodes.iter() {
            if let Some(r) = self.topo_graph.sub_graphs.get(id) {
                for input in r.from.iter() {
                    v.push(format!("\t \"{id:?}_in\" -> \"{input:?}\""));
                }
                for out in r.to.iter() {
                    v.push(format!("\t \"{out:?}\" -> \"{id:?}_out\""));
                }
            }
        }

        for (from, to) in self.topo_graph.edges.iter() {
            let from = match self.topo_graph.sub_graphs.get(*from) {
                Some(_r) => format!("{from:?}_out"),
                None => format!("{from:?}"),
            };
            let to = match self.topo_graph.sub_graphs.get(*to) {
                Some(_r) => format!("{to:?}_in"),
                None => format!("{to:?}"),
            };
            v.push(format!("\t \"{from}\" -> \"{to}\""));
        }

        v.push("}".into());

        let s = v.join("\n");

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
                        log::warn!("write to dump_toop_graphviz.dot error = {:?}", e);
                    }
                },
                Err(e) => {
                    log::warn!("open dump_toop_graphviz.dot for write error = {:?}", e);
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
                "\t \"{id:?}\" [\"style\"=\"filled\" \"label\"=\"{name}_{id1:?}\" \"fillcolor\"=\"{color}\"]"
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
    pub fn add_node<'a, R>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        node: R, 
        mut parent_graph_id: NodeId,
		is_run: bool,
    ) -> Result<NodeId, GraphError>
    where
        R: DependNode<Context, DataId>,
    {
        if parent_graph_id.is_null() {
            parent_graph_id = self.main_graph_id;
        }
        self.add(name, node, parent_graph_id, is_run, false)
    }   

    // 设置是否为传输节点
    pub fn set_is_transfer(&mut self, id: NodeId, is_transfer: bool) {
        if self.topo_graph.set_is_transfer(id, is_transfer) {
            self.is_finish_dirty = true; // 设置is_finish_dirty脏
        }
    }

    ///主图id
    pub fn main_graph_id(&self) -> NodeId {
        self.main_graph_id
    }

    /// 添加 名为 name 的 子图
    pub fn add_sub_graph(
        &mut self,
        name: impl Into<Cow<'static, str>>,
    ) -> Result<NodeId, GraphError>
    {
        self.add(name, InternalNodeEmptyImpl, Null::null(), false, true)
    }

	/// 设置子图的父, 只能在该图与其他节点创建连接关系之前设置， 否则设置不成功
	pub fn set_sub_graph_parent(&mut self, k: NodeId, parent_graph_id: NodeId) {
		self.topo_graph.set_sub_graph_parent(k, parent_graph_id);
	}

    /// 设置dataid
    pub fn set_data_id(&mut self, node_id: NodeId, data_id: DataId) {
        if let Some(r) = self.nodes.get_mut(node_id) {
            r.data_id = data_id;
        }
    }

    /// 获取dataid
    pub fn get_data_id(&self, node_id: NodeId) -> DataId {
        if let Some(r) = self.nodes.get(node_id) {
            r.data_id
        } else {
            DataId::null()
        }
    }

    /// 添加 名为 name 的 节点
    fn add<'a, R>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        node: R,
        parent_graph_id: NodeId,
		is_run: bool,
        is_sub_graph: bool,
    ) -> Result<NodeId, GraphError>
    where
        R: DependNode<Context, DataId>,
    {
        let name = name.into();

        // 如果存在同名节点，返回 Err
        if let Some(id) = self.node_names.get(&name) {
            return Err(GraphError::ExitNode(format!("{id:?}")));
        }

        // // 拓扑结构改变
        // self.is_topo_dirty = true;

        let node_state = NodeState::<Context, DataId>::new(node);
		let run_node = self.create_run_node(node_state.clone())?;
		let build_node: Box<dyn BuildFuncTrait<Context, DataId>> = self.create_build_node(node_state.clone())?;
        let node_id = self.nodes.insert(ScheduleNode {
            build_node,
            run_node,
            name: name.to_string(),
            state: node_state,
			is_run,
            is_build: true,
            from_data_id: Default::default(),
            to_data_id: Default::default(),
            data_id: Null::null(),

            curr_next_build_refs: 0,
            total_next_build_refs: 0,
			// run_way: RunWay::Schedule,
        });
        if is_sub_graph {
            self.topo_graph.add_sub_graph(node_id, ());
        } else {
            self.topo_graph.add_node(node_id, (), parent_graph_id);
        }
        self.need_init_nodes.push(node_id);
        

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
            self.need_init_nodes.retain(|&nid| nid != id); // 添加这一行
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

    pub fn set_is_build(
        &mut self,
        label: impl Into<NodeLabel>,
        is_is_build: bool,
    ) -> Result<(), GraphError> {
        let label = label.into();
        let node_id = self.get_id(&label)?;

        if let Some(node) = self.nodes.get_mut(node_id) {
            if node.is_build != is_is_build {
                node.is_build = is_is_build;
                self.is_enable_dirty = true;
            }
        }
        Ok(())
    }

    // pub fn set_enable(
    //     &mut self,
    //     label: impl Into<NodeLabel>,
    //     is_enable: bool,
    // ) -> Result<(), GraphError> {
    //     let label = label.into();
    //     let node_id = self.get_id(&label)?;

    //     if let Some(node) = self.nodes.get_mut(node_id) {
    //         if node.is_build != is_enable {
    //             node.is_build = is_enable;
    //             self.is_enable_dirty = true;
    //         }

    //         if node.is_run != is_enable {
    //             node.is_run = is_enable;
    //             self.is_enable_dirty = true;
    //         }
            
    //     }
    //     Ok(())
    // }
    // 设置是否激活节点, 未激活的节点， 和它的递归前置节点都不会链接到最终的执行图中
    pub fn set_enable(&mut self, label: impl Into<NodeLabel>, is_enable: bool) -> Result<(), GraphError> {
        let label = label.into();
        let node_id = self.get_id(&label)?;
        if self.topo_graph.set_enable(node_id, is_enable) {
            self.is_topo_dirty = true; // 设置is_topo_dirty脏
            // log::warn!("set_enable: {:?}", (node_id, is_enable));
        }
        Ok(())
    }

    pub fn set_is_run(
        &mut self,
        label: impl Into<NodeLabel>,
        is_run: bool,
    ) -> Result<(), GraphError> {
        let label = label.into();
        let node_id = self.get_id(&label)?;

        if let Some(node) = self.nodes.get_mut(node_id) {
            if node.is_run != is_run {
                node.is_run = is_run;
                self.is_enable_dirty = true;
            }
            
        }
        Ok(())
    }
}



/// 渲染图的 执行 相关
impl<Context: ThreadSync + 'static, DataId: Key + ThreadSync> DependGraph<Context, DataId> {
    /// 执行 渲染
    pub async fn run<A: 'static + AsyncRuntime + Send>(
        &mut self,
        rt: &A,
        context: &mut Context,
    ) -> Result<(), GraphError> {
		// 运行所有图节点的run方法
		let topological_sort = &self.schedule_graph.topological;
        let nodes = self.can_run_nodes();
		let mut map = rt.map_reduce(nodes.len());
		let context: &Context = context;
		let mut index = 0;
		for node_id in nodes.iter() {
			let node = match self.nodes.get(*node_id) {
				Some(r) => r,
				None => panic!("error============={:?}", *node_id),
			};
			// if !node.is_run {
			// 	continue;
			// }
			// let graph_node: &super::graph_data::NGraphNode<NodeId, DataId> = self.schedule_graph.get(*node_id).unwrap();
			// 这里用transmute绕过声明周期， 是安全的，因为在context、self释放之前，map中的任务已完成（外部等待）
			map.map(rt.clone(), (*node.run_node)(
                index, 
                unsafe {transmute(context)}, 
                node.data_id,
                unsafe { transmute(node.from_data_id.as_slice())},
                unsafe { transmute(node.to_data_id.as_slice())},
            )).unwrap();
			index += 1;
		}
		map.reduce(false).await.unwrap();

		// 重置输入输出参数
        // #[cfg(feature = "trace")]
        // let _clear_span = tracing::warn_span!("Clear Graph");
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
        // let t1 = pi_time::Instant::now();
		// let is_topo_dirty = self.is_topo_dirty;
		// 检查图是否改变，如果改变， 需要重构图
		let build_ret: Result<(), GraphError> = self.update();
		build_ret?;
        // let t2 = pi_time::Instant::now();

		// 运行所有激活图节点的build方法
		for node_id in self.build_nodes.iter() {
			let node: &mut ScheduleNode<Context, DataId> = &mut self.nodes[*node_id];
            node.curr_next_build_refs = node.total_next_build_refs as i32;
			// let graph_node = self.schedule_graph.get(*node_id).unwrap();
			(*node.build_node)(
                context, 
                node.data_id,
                unsafe { transmute(node.from_data_id.as_slice())},
                unsafe { transmute(node.to_data_id.as_slice())},
            ).unwrap();

            let graph_node = match self.schedule_graph.get(*node_id) {
                Some(r) => r,
                None => continue,
            };
            for from in graph_node.from().iter() {
                let node = &mut self.nodes[*from];

                // 用完了 前置，引用计数 减 1
                node.curr_next_build_refs -= 1;

                
                if node.curr_next_build_refs == 0 {
                    log::debug!("from buildend, from_data_id:{:?}, data_id:{:?}, total_next_build_refs: {:?}", node.data_id, node_id, node.total_next_build_refs);
                    // SAFE: 此处强转可变是安全的，因为单线程执行build
                    node.state.0.borrow_mut().build_end(context, node.data_id);
                }
            }
            let node = &self.nodes[*node_id];
            if node.curr_next_build_refs == 0 { 
                log::debug!("self buildend, data_id:{:?}, total_next_build_refs: {:?}", node.data_id, node.total_next_build_refs);
                node.state.0.borrow_mut().build_end(context, node.data_id);
            }
		}

        if self.need_init_nodes.len() > 0 {
            // let t3 = pi_time::Instant::now();
            // log::warn!("build========");
            for node_id in self.need_init_nodes.iter() {
                let node = &self.nodes[*node_id];
                (*node.state.0.borrow_mut()).init(context).unwrap();
            }
            self.need_init_nodes.clear();
        }
        // let t3 = pi_time::Instant::now();
        // log::warn!("build=================={:?}", (t2 - t1, t3 - t2, self.enable_nodes.len()));
		Ok(())
    }

	/// 更新图
    pub fn update(&mut self) -> Result<(), GraphError> {
        self.update_graph()?;

        // 构建 run_ng，返回 构建图
		if self.is_topo_dirty || self.is_finish_dirty {
            // log::warn!("update_run_ng=================");
			if let Err(GraphError::ParamFillRepeat(f1, f2, t)) = self.update_run_ng() {
                log::error!("param fill with repeat, graph: {:}", self.dump_graphviz());
                return Err(GraphError::ParamFillRepeat(f1, f2, t));
            }

		}

        if self.is_topo_dirty || self.is_enable_dirty || self.is_finish_dirty {
            // 重新生成激活节点
            self.build_nodes.clear();
            // 计算可运行节点
            self.can_run_node.clear();
            for id in self.schedule_graph.topological.iter() {
                let graph_node = match self.schedule_graph.get(*id) {
                    Some(r) => r,
                    None => continue,
                };
                let node = &mut self.nodes[*id];
                if node.is_run {
                    self.can_run_node.push(id.clone());
                }

                node.total_next_build_refs = graph_node.to().len() as i32; // 初始化总数量
                if self.nodes[*id].is_build {
                    self.build_nodes.push(id.clone());
                } else {
                    // 如果节点不可build， 需要将前直接点的next数量减1
                    for from in graph_node.from() {
                        let node = &mut self.nodes[*from];
                        node.total_next_build_refs -= 1;
                    }
                }
               

                pi_print_any::out_any!(log::debug, "enable_nodes======{:?}", &self.nodes[*id].is_run);
            }
            // log::warn!("enable_nodes======{:?}", (self.is_topo_dirty, self.is_enable_dirty, self.is_finish_dirty));
            // log::warn!("enable_nodes======{:?}", &self.enable_nodes);
            // log::warn!("can_run_node======{:?}", &self.can_run_node);
        }

        self.is_enable_dirty = false;
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
	pub fn can_run_nodes(&self) -> &[NodeId] {
		&self.can_run_node
	}

    pub fn can_build_nodes(&self) -> &[NodeId] {
        &self.build_nodes
    }

    
}

// ================== 以下方法 仅供 crate 使用

impl<Context: ThreadSync + 'static, DataId: Key + ThreadSync> DependGraph<Context, DataId> {
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
            // 根据节点连接关系，更新拓扑图（用户将节点与节点、节点与子图连接在一起， 需要修改为节点之间的链接关系）
            let r = self.topo_graph.build();
            if r.is_err() {
                log::error!("topo graph: {:?}", self.dump_toop_graphviz());
            }
            r.unwrap();
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
            let node = &mut self.nodes[*id];
            node.from_data_id.clear();
            node.to_data_id.clear();
            let mut from_data_id = std::mem::replace(&mut node.from_data_id, Vec::new());
            let mut to_data_id = std::mem::replace(&mut node.to_data_id, Vec::new());
            for from in graph_node.from() {
                let data_id = self.nodes[*from].data_id;
                if !data_id.is_null() {
                    from_data_id.push(self.nodes[*from].data_id);
                }
            }
            for to in graph_node.to() {
                let data_id = self.nodes[*to].data_id;
                if !data_id.is_null() {
                    to_data_id.push(data_id);
                }
            }

            let node = &mut self.nodes[*id];
            node.from_data_id = from_data_id;
            node.to_data_id = to_data_id;
            // let node = &mut self.nodes[*id];
            // let mut node_state = node.state.0.as_ref().borrow_mut();
           
            // node_state.set_next_count(graph_node.to().len() as i32);
        }
		Ok(())
    }

    // 创建 构建 节点
    fn create_build_node(
        &self,
        node_state: NodeState<Context, DataId>,
    ) -> Result<BuildFunc<Context, DataId>, GraphError> {
		let f = move |context: &mut Context, id: DataId, from: &'static [DataId], to: &'static [DataId]| -> std::io::Result<()> {
			Ok(node_state.0.as_ref().borrow_mut().build(context, id, from, to).unwrap())
        };
		Ok(Box::new(f))
    }

    // 创建 渲染 节点
    fn create_run_node(
        &self,
		node_state: NodeState<Context, DataId>,
        // node_id: NodeId,
    ) -> Result<RunFunc<Context, DataId>, GraphError> {
        // 该函数 会在 ng 图上，每帧每节点 执行一次
        let f = move |index: usize, context: &'static Context, id: DataId, from: &'static [DataId], to: &'static [DataId]| -> BoxFuture<'static, std::io::Result<()>> {
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

pub trait BuildFuncTrait<C: ThreadSync + 'static, DataId: Key + ThreadSync>: Fn(&mut C, DataId, &'static [DataId], &'static [DataId]) -> std::io::Result<()> + ThreadSync + 'static {}
impl<Context: ThreadSync + 'static, DataId: Key + ThreadSync, T: Fn(&mut Context, DataId, &'static [DataId], &'static [DataId]) -> std::io::Result<()> + ThreadSync + 'static> BuildFuncTrait<Context, DataId> for T {}

pub trait RunFuncTrait<C: ThreadSync + 'static, DataId: Key + ThreadSync>: Fn(usize, &'static C, DataId, &'static [DataId], &'static [DataId]) -> BoxFuture<'static, std::io::Result<()>> + ThreadSync + 'static{}
impl<Context: ThreadSync + 'static, DataId: Key + ThreadSync, T: Fn(usize, &'static Context, DataId, &'static [DataId], &'static [DataId]) -> BoxFuture<'static, std::io::Result<()>> + ThreadSync + 'static> RunFuncTrait<Context, DataId> for T {}

type BuildFunc<Context, DataId> = Box<dyn BuildFuncTrait<Context, DataId>>;

type RunFunc<Context, DataId> = Box<dyn RunFuncTrait<Context, DataId>>;


pub struct InternalNodeEmptyImpl;

impl<Context: ThreadSync + 'static, DataId: Key + ThreadSync> DependNode<Context, DataId> for InternalNodeEmptyImpl {

    fn init<'a>(
        &'a mut self,
        _context: &'a mut Context,
        // input: &'a Self::Input,
        // usage: &'a ParamUsage,
		// id: NodeId, 
		// from: &[NodeId],
		// to: &[NodeId],
    ) -> Result<(), String> {
        Ok(())
    }

    fn build<'a>(
        &'a mut self,
        _context: &'a mut Context,
        // _input: &'a Self::Input,
        // _usage: &'a ParamUsage,
		_id: DataId, 
		_from: &[DataId],
		_to: &[DataId],
    ) -> Result<(), String> {
        Ok(())
    }

    fn run<'a>(
        &'a mut self,
		_index: usize,
        _context: &'a Context,
        // _input: &'a Self::Input,
        // _usage: &'a ParamUsage,
		_id: DataId, 
		_from: &'static [DataId],
		_to: &'static [DataId],

    ) -> BoxFuture<'a, Result<(), String>> {
        // async {Ok(())}.
		todo!()
    }
    
    fn reset<'a>(
        &'a mut self,
        _context: &'a mut Context,
        _id: DataId,
    ) {
        todo!()
    }

}

struct ScheduleNode<Context: 'static + ThreadSync, DataId: ThreadSync + Key> {
	build_node: BuildFunc<Context, DataId>, // build方法， 如果是图，build为 empty_build
	run_node: RunFunc<Context, DataId>,// run方法， 如果是图，run为 empty_run
	name: String, // 节点名字
	state: NodeState<Context, DataId>, // 节点状态
	is_run: bool,
    is_build: bool, // 是否需要build）

    // from节点和to节点的数据id， 当toop图发生改变时，需要重置from_data_id和 to_data_id
    from_data_id: Vec<DataId>,
    to_data_id: Vec<DataId>,

    data_id: DataId,

    curr_next_build_refs: i32,
    total_next_build_refs: i32,
}


// #[derive(Debug)]
// pub enum RunWay {
// 	Schedule, // 运行方式为图运行时派发
// 	Require(AtomicBool), // 运行方式为外部主动请求运行, AtomicBool表示当前是否正在运行
// }










