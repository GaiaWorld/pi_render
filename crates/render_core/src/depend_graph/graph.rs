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
use pi_async_graph::{async_graph, ExecNode, RunFactory, Runner};
use pi_futures::BoxFuture;
use pi_graph::{DirectedGraph, DirectedGraphNode, NGraph, NGraphBuilder};
use pi_hash::{XHashMap, XHashSet};
use pi_share::{Share, ShareCell, ThreadSync};
use pi_slotmap::SlotMap;
use std::{borrow::Cow, marker::PhantomData};

/// 依赖图
pub struct DependGraph<Context: ThreadSync + 'static> {
    // ================== 拓扑信息

    // 名字 和 NodeId 映射
    node_names: XHashMap<Cow<'static, str>, NodeId>,

    // 所有 节点的 集合
    nodes: SlotMap<NodeId, (String, NodeState<Context>)>,

    // 边 (before, after)
    edges: XHashMap<NodeId, NodeSlot>,

    // 最终节点，渲染到屏幕的节点
    finish_nodes: XHashSet<NodeId>,

    // 有没有 修改 nodes, edges
    is_topo_dirty: bool,

    // 拓扑图，is_topo_dirty 为 false 则 不会 构建
    // 注：ng 的 边 和 edges 的 (before, after) 是 相反的
    topo_ng: Option<NGraph<NodeId, NodeId>>,

    // 有没有 修改 finish_nodes
    is_finish_dirty: bool,

    // ================== 运行信息

    // 运行图 中 入度为0 的节点
    input_node_ids: Vec<NodeId>,

    // 运行 节点 build 方法的图，当切仅当 图 有所变化时候，每个节点会重新运行一次；
    // build_ng 如果为Some，这一帧会执行，紧接着 build_ng = None
    // build_ng 边 和 edges 的 (before, after) 相同
    build_ng: Option<
        Share<NGraph<NodeId, ExecNode<Context, BuildSyncRun<Context>, BuildSyncRun<Context>>>>,
    >,

    // 录制渲染指令的 异步执行图，用于 更新 GPU 资源
    // 当 渲染图 拓扑改变 或 finish 节点 改变后，会 重新 构建个 新的
    // run_ng边 和 edges 的 (before, after) 相同
    run_ng: Option<
        Share<NGraph<NodeId, ExecNode<Context, BuildSyncRun<Context>, BuildSyncRun<Context>>>>,
    >,
}

impl<Context: ThreadSync + 'static> Default for DependGraph<Context> {
    fn default() -> Self {
        Self {
            node_names: XHashMap::default(),

            nodes: SlotMap::default(),
            edges: XHashMap::default(),

            finish_nodes: XHashSet::default(),

            is_topo_dirty: true,
            topo_ng: None,

            is_finish_dirty: true,

            input_node_ids: vec![],

            build_ng: None,
            run_ng: None,
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

        for (id, (name, _)) in self.nodes.iter() {
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

        for (id, slot) in self.edges.iter() {
            for next_id in slot.next_nodes.iter() {
                v.push(format!("\t \"{id:?}\" -> \"{next_id:?}\""));
            }
        }

        v.push("}".into());

        v.join("\n")
    }

    /// 查 指定节点 的 前驱节点
    pub fn get_prev_ids(&self, id: NodeId) -> Option<&[NodeId]> {
        self.edges.get(&id).map(|v| v.prev_nodes.as_slice())
    }

    /// 查 指定节点 的 后继节点
    pub fn get_next_ids(&self, id: NodeId) -> Option<&[NodeId]> {
        self.edges.get(&id).map(|v| v.next_nodes.as_slice())
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

        let node_id = self.nodes.insert((name.to_string(), node_state));

        self.node_names.insert(name, node_id);
        self.edges.insert(node_id, NodeSlot::default());
		
        Ok(node_id)
    }

    /// 移除 节点
    pub fn remove_node(&mut self, label: impl Into<NodeLabel>) -> Result<(), GraphError> {
        let label = label.into();

        let id = match self.get_node_id(&label) {
            Ok(v) => v,
            Err(_) => return Err(GraphError::NoneNode(format!("{:?}", label))),
        };

        // 拓扑结构改变
        self.is_topo_dirty = true;

        let node = match self.nodes.remove(id) {
            Some(r) => r,
            None => return Err(GraphError::NoneNode(format!("{:?}", label))),
        };
        self.finish_nodes.remove(&id);
        self.node_names.remove(node.0.as_str());

        // 图：删点 必 删边
        if let Some(slot) = self.edges.remove(&id) {
            for prev in slot.prev_nodes.iter() {
                // 删除 所有 前驱节点 的 后继
                self.edges
                    .get_mut(prev)
                    .and_then(|s| s.remove_next_slot(id));
            }

            for next in slot.next_nodes.iter() {
                // 删除 所有 后继节点 的 前驱
                self.edges
                    .get_mut(next)
                    .and_then(|s| s.remove_prev_slot(id));
            }
        }

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
        let after_node = self.get_node_id(&after_label)?;

        if self
            .edges
            .get_mut(&before_node)
            .and_then(|slot| slot.add_next_slot(after_node))
            .is_some()
        {
            // 拓扑结构改变
            self.is_topo_dirty = true;
        }

        if self
            .edges
            .get_mut(&after_node)
            .and_then(|slot| slot.add_prev_slot(before_node))
            .is_some()
        {
            // 拓扑结构改变
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

        if self
            .edges
            .get_mut(&after_node)
            .and_then(|slot| slot.remove_prev_slot(before_node))
            .is_some()
        {
            // 拓扑结构改变
            self.is_topo_dirty = true;
        }

        if self
            .edges
            .get_mut(&before_node)
            .and_then(|slot| slot.remove_next_slot(after_node))
            .is_some()
        {
            // 拓扑结构改变
            self.is_topo_dirty = true;
        }

        Ok(())
    }
}

/// 渲染图的 执行 相关
impl<Context: ThreadSync + 'static> DependGraph<Context> {
    /// 构建图，不需要 运行时
    pub fn build(&mut self) -> Result<(), GraphError> {
        let sub_ng = match self.update_topo()? {
            None => return Ok(()),
            Some(g) => g,
        };

        // 构建 run_ng，返回 构建图
        self.create_run_ng(sub_ng)?;

        Ok(())
    }

    /// 执行 渲染
    pub async fn run<A: 'static + AsyncRuntime + Send>(
        &mut self,
        rt: &A,
        context: &'static Context,
    ) -> Result<(), GraphError> {
        // 注 构建 ng 只运行一次
        match self.build_ng.take() {
            None => {}
            Some(g) => match async_graph(rt.clone(), g.clone(), context).await {
                Ok(_) => {}
                Err(e) => {
                    let err = GraphError::RunNGraphError(format!("run_ng, {e:?}"));

                    error!("{}", err);
                    return Err(err);
                }
            },
        }

        match self.run_ng {
            None => {
                let e = GraphError::NoneNGraph("run_ng".to_string());
                error!("{}", e);
                Err(e)
            }
            Some(ref g) => match async_graph(rt.clone(), g.clone(), context).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    let err = GraphError::RunNGraphError(format!("run_ng, {e:?}"));

                    error!("{}", err);
                    Err(err)
                }
            },
        }
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

    fn update_topo(&mut self) -> Result<Option<NGraph<NodeId, NodeId>>, GraphError> {
        if self.is_topo_dirty {
            // 拓扑结构变，全部 需要 重构
            self.is_finish_dirty = true;

            self.topo_ng = None;
            self.run_ng = None;
        } else if self.is_finish_dirty {
            // finish 变，子图 和 执行部分 需要 重构
            self.run_ng = None;
        } else {
            // 都没改过，就认为 run_ng 全部就绪
            return Ok(None);
        }

        // 有必要的话，修改 拓扑结构
        self.change_topo()?;

        Ok(Some(self.gen_sub()?))
    }

    // // 取 label 对应的 Name
    // fn get_node_name(&self, id: NodeId) -> Result<&str, GraphError> {
    //     self.nodes
    //         .get(id)
    //         .map(|v| v.0.as_str())
    //         .ok_or_else(|| GraphError::NoneNode(format!("id = {id:?}")))
    // }

    // 取 label 对应的 NodeState
    fn get_node_state(&self, label: &NodeLabel) -> Result<&NodeState<Context>, GraphError> {
        self.get_node_id(label).and_then(|id| {
            self.nodes
                .get(id)
                .map(|v| &v.1)
                .ok_or_else(|| GraphError::NoneNode(label.into()))
        })
    }

    // 生成 子图
    fn gen_sub(&mut self) -> Result<NGraph<NodeId, NodeId>, GraphError> {
        // 子图 一定是 修改过的
        assert!(self.is_finish_dirty);

        self.is_finish_dirty = false;

        let ng = self.topo_ng.as_ref().unwrap();

        // 以终为起，构建需要的 节点
        let finishes: Vec<NodeId> = self.finish_nodes.iter().copied().collect();

        let sub_ng = ng.gen_graph_from_keys(&finishes);

        self.input_node_ids = sub_ng.from().to_vec();

        Ok(sub_ng)
    }

    // 取 拓扑图，有必要就重新构建
    fn change_topo(&mut self) -> Result<(), GraphError> {
        // 拓扑 没修改，返回 原图
        if !self.is_topo_dirty {
            return Ok(());
        }
        self.is_topo_dirty = false;

        // 构建成功, ng_builder 就 删掉
        let mut builder = NGraphBuilder::<NodeId, NodeId>::new();
        // 节点 就是 高层添加 的 节点
        for (node_id, _) in &self.nodes {
            builder = builder.node(node_id, node_id);
        }

        {
            let mut access_edges = XHashSet::<(NodeId, NodeId)>::default();
            for (before, slot) in self.edges.iter() {
                for after in slot.next_nodes.iter() {
                    if !access_edges.contains(&(*before, *after)) {
                        // 顺序 必须和 依赖图顺序 相反
                        builder = builder.edge(*after, *before);

                        access_edges.insert((*before, *after));
                    }
                }
            }
        }

        let ng = match builder.build() {
            Ok(ng) => ng,
            Err(e) => {
                let msg = format!("ng build failed, e = {e:?}");
                return Err(GraphError::BuildError(msg));
            }
        };

        self.topo_ng = Some(ng);
        Ok(())
    }

    // 创建真正的 运行图
    // 返回 构建 的 执行图
    fn create_run_ng(&mut self, sub_ng: NGraph<NodeId, NodeId>) -> Result<(), GraphError> {
        let mut build_builder = NGraphBuilder::<
            NodeId,
            ExecNode<Context, BuildSyncRun<Context>, BuildSyncRun<Context>>,
        >::new();

        let mut run_builder = NGraphBuilder::<
            NodeId,
            ExecNode<Context, BuildSyncRun<Context>, BuildSyncRun<Context>>,
        >::new();

        let topo_ids = sub_ng.topological_sort();

        // 异步图 节点
        for id in topo_ids {
            // 先重置 节点
            let n = self.get_node_state(&NodeLabel::from(*id)).unwrap();
            n.0.as_ref().borrow_mut().reset();

            let node = self.create_run_node(*id)?;
            run_builder = run_builder.node(*id, node);

            let node = self.create_build_node(*id)?;
            build_builder = build_builder.node(*id, node);
        }

        // 异步图 边
        // sub_ng 是 以终为起的，所以 sub_ng 的 from 和 to 和 执行顺序 相反；
        for id in topo_ids {
            let to = sub_ng.get(id).unwrap();
            for from in to.to() {
                let from = sub_ng.get(from).unwrap();

                let from = *from.value();
                let to = *to.value();

                // 构造边
                run_builder = run_builder.edge(from, to);
                build_builder = build_builder.edge(from, to);

                let f_n = self.nodes.get_mut(from).unwrap().1.clone();

                // 为 to 天上 prenode = from
                let t_n = self.nodes.get_mut(to).unwrap();
                t_n.1 .0.as_ref().borrow_mut().add_pre_node((from, f_n));
            }
        }

        match run_builder.build() {
            Ok(g) => {
                self.run_ng = Some(Share::new(g));
            }
            Err(e) => {
                let msg = format!("run_ng e = {e:?}");
                error!("{}", msg);
                return Err(GraphError::BuildError(msg));
            }
        }

        // 构建图 只用 一次，用完 就 释放
        match build_builder.build() {
            Ok(g) => {
                self.build_ng = Some(Share::new(g));
                Ok(())
            }
            Err(e) => {
                let msg = format!("build_builder e = {e:?}");
                error!("{}", msg);
                Err(GraphError::BuildError(msg))
            }
        }
    }

    // 创建 构建 节点
    fn create_build_node(
        &self,
        node_id: NodeId,
    ) -> Result<ExecNode<Context, BuildSyncRun<Context>, BuildSyncRun<Context>>, GraphError> {
        let n = self.get_node_state(&NodeLabel::Id(node_id));
        let node = n.unwrap().0.clone();

        // 该函数 会在 ng 图上，每帧每节点 执行一次
        let f = BuildSyncRun::new(node);

        Ok(ExecNode::Sync(f))
    }

    // 创建 渲染 节点
    fn create_run_node(
        &self,
        node_id: NodeId,
    ) -> Result<ExecNode<Context, BuildSyncRun<Context>, BuildSyncRun<Context>>, GraphError> {
        let n = self.get_node_state(&NodeLabel::Id(node_id));
        let node = n.unwrap().0.clone();

        // 该函数 会在 ng 图上，每帧每节点 执行一次
        let f = move |context: &'static Context| -> BoxFuture<'static, std::io::Result<()>> {
            let node = node.clone();
            Box::pin(async move {
                // log::warn!("run graphnode start {:?}", node_id);
                node.as_ref().borrow_mut().run(context).await.unwrap();
                // log::warn!("run graphnode end {:?}", node_id);
                Ok(())
            })
        };

        Ok(ExecNode::Async(Box::new(f)))
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

impl<Context: 'static + ThreadSync> Runner<Context> for BuildSyncRun<Context> {
    fn run(self, context: &'static Context) {
        self.node.as_ref().borrow_mut().build(context).unwrap();
    }
}

impl<Context: 'static + ThreadSync> RunFactory<Context> for BuildSyncRun<Context> {
    type R = BuildSyncRun<Context>;

    fn create(&self) -> Self::R {
        self.clone()
    }
}

#[derive(Default)]
struct NodeSlot {
    prev_nodes: Vec<NodeId>,

    next_nodes: Vec<NodeId>,
}

impl NodeSlot {
    // 添加 next 对应的 slot
    #[inline]
    fn add_next_slot(&mut self, next: NodeId) -> Option<()> {
        match self.next_nodes.iter().position(|s| *s == next) {
            Some(_) => None,
            None => {
                self.next_nodes.push(next);
                Some(())
            }
        }
    }

    // 到 id 的节点 添加 prev 对应的 slot
    #[inline]
    fn add_prev_slot(&mut self, prev: NodeId) -> Option<()> {
        match self.prev_nodes.iter().position(|s| *s == prev) {
            Some(_) => None,
            None => {
                self.prev_nodes.push(prev);
                Some(())
            }
        }
    }

    // 到 id 的节点 删除 next 对应的 slot
    #[inline]
    fn remove_next_slot(&mut self, next: NodeId) -> Option<()> {
        self.next_nodes
            .iter()
            .position(|value| *value == next)
            .map(|index| {
                self.next_nodes.swap_remove(index);
            })
            .or(None)
    }

    // 到 id 的节点 删除 prev 对应的 slot
    #[inline]
    fn remove_prev_slot(&mut self, prev: NodeId) -> Option<()> {
        self.prev_nodes
            .iter()
            .position(|value| *value == prev)
            .map(|index| {
                self.prev_nodes.swap_remove(index);
            })
            .or(None)
    }
}
