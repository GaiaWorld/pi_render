//! RenderGraph
//!

use super::{
    node::{Node, NodeId, NodeLabel, NodeState},
    param::{InParam, OutParam},
    GraphError, RenderContext,
};
use crate::rhi::{device::RenderDevice, RenderQueue};
use futures::{future::BoxFuture, FutureExt};
use log::error;
use pi_async::rt::AsyncRuntime;
use pi_async_graph::{async_graph, ExecNode, RunFactory, Runner};
use pi_graph::{DirectedGraph, DirectedGraphNode, NGraph, NGraphBuilder};
use pi_hash::{XHashMap, XHashSet};
use pi_share::{Share, ShareRefCell};
use pi_slotmap::SlotMap;
use std::{borrow::Cow, sync::Arc};

/// 渲染图
pub struct RenderGraph {
    // ================== 拓扑信息

    // 名字 和 NodeId 映射
    node_names: XHashMap<Cow<'static, str>, NodeId>,

    // 所有 节点的 集合
    nodes: SlotMap<NodeId, NodeState>,

    // 边 (before, after)
    edges: XHashSet<(NodeId, NodeId)>,

    // 最终节点，渲染到屏幕的节点
    finish_nodes: XHashSet<NodeId>,

    // 有没有 修改 nodes, edges
    is_topo_dirty: bool,
    // 拓扑图，is_topo_dirty 为 false 则 不会 构建
    // 注：ng 的 边 和 edges 的 (before, after) 是 相反的
    topo_ng: Option<NGraph<NodeId, NodeId>>,

    // 有没有 修改 finish_nodes
    is_finish_dirty: bool,
    // 拓扑 子图，is_topo_dirty 和 is_finish_ng 同时为 false 则 不会 重新构建
    // 注：ng 的 边 和 edges 的 (before, after) 是 相反的
    topo_sub_ng: Option<NGraph<NodeId, NodeId>>,
    // ================== 运行信息

    // 录制渲染指令的 异步执行图，用于 更新 GPU 资源
    // 当 渲染图 拓扑改变 或 finish 节点 改变后，会 重新 构建个 新的
    // run_ng边 和 edges 的 (before, after) 相同
    run_ng: Option<Share<NGraph<NodeId, ExecNode<EmptySyncRun, EmptySyncRun>>>>,
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self {
            node_names: XHashMap::default(),

            nodes: SlotMap::default(),
            edges: XHashSet::default(),

            finish_nodes: XHashSet::default(),

            is_topo_dirty: true,
            topo_ng: None,

            is_finish_dirty: true,
            topo_sub_ng: None,

            run_ng: None,
        }
    }
}

/// 渲染图的 拓扑信息 相关 方法
impl RenderGraph {
    /// 添加 名为 name 的 节点
    pub fn add_node<I, O, R>(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        node: R,
    ) -> Result<NodeId, GraphError>
    where
        I: InParam + Default,
        O: OutParam + Default + Clone,
        R: Node<Input = I, Output = O>,
    {
        let name = name.into();
        // 如果存在同名节点，返回 Err
        if let Some(id) = self.node_names.get(&name) {
            return Err(GraphError::ExitNode(NodeLabel::from(*id)));
        }

        self.is_topo_dirty = true;

        let node_state = NodeState::new(node);

        let node_id = self.nodes.insert(node_state);

        self.node_names.insert(name, node_id);

        Ok(node_id)
    }

    /// 移除 节点
    pub fn remove_node(&mut self, id: NodeId, name: impl Into<Cow<'static, str>>) {
        if self.nodes.get(id).is_some() {
            self.is_topo_dirty = true;

            self.nodes.remove(id);
            self.finish_nodes.remove(&id);
            self.node_names.remove(&name.into());

            // 图：删点 必 删边
            let mut remove_edges = vec![];
            for (before, after) in self.edges.iter() {
                if id == *before || id == *after {
                    remove_edges.push((*before, *after));
                }
            }
            for pair in remove_edges {
                self.edges.remove(&pair);
            }
        }
    }

    /// 建立 Node 的 依赖关系
    /// 执行顺序 `before` 先于 `after`
    pub fn add_depend(
        &mut self,
        before: impl Into<NodeLabel>,
        after: impl Into<NodeLabel>,
    ) -> Result<(), GraphError> {
        let before_id = self.get_node_id(before)?;
        let after_id = self.get_node_id(after)?;

        if self.edges.get(&(before_id, after_id)).is_none() {
            self.is_topo_dirty = true;
            self.edges.insert((before_id, after_id));
        }

        Ok(())
    }

    /// 移除依赖
    pub fn remove_depend(
        &mut self,
        before: impl Into<NodeLabel>,
        after: impl Into<NodeLabel>,
    ) -> Result<(), GraphError> {
        let before_id = self.get_node_id(before)?;
        let after_id = self.get_node_id(after)?;

        if self.edges.get(&(before_id, after_id)).is_some() {
            self.is_topo_dirty = true;
            self.edges.remove(&(before_id, after_id));
        }

        Ok(())
    }

    /// 设置 是否 是 最终节点
    pub fn set_finish(
        &mut self,
        node: impl Into<NodeLabel>,
        is_finish: bool,
    ) -> Result<(), GraphError> {
        let node = node.into();
        let node_id = self.get_node_id(node)?;

        if is_finish {
            if !self.finish_nodes.contains(&node_id) {
                self.is_finish_dirty = true;
                self.finish_nodes.insert(node_id);
            }
        } else if self.finish_nodes.contains(&node_id) {
            self.is_finish_dirty = true;
            self.finish_nodes.remove(&node_id);
        }

        Ok(())
    }
}

/// 渲染图的 执行 相关
impl RenderGraph {
    pub async fn build<A: 'static + AsyncRuntime + Send>(
        &mut self,
        rt: &A,
        device: RenderDevice,
        queue: RenderQueue,
    ) -> Result<(), GraphError> {
        if self.is_topo_dirty {
            self.is_finish_dirty = true;

            self.topo_ng = None;
            self.topo_sub_ng = None;
            self.run_ng = None;
        } else if self.is_finish_dirty {
            self.topo_sub_ng = None;
            self.run_ng = None;
        } else {
            // 都没改过，就认为 run_ng 全部就绪
            return Ok(());
        }

        // 拓扑结构
        self.get_topo_ng().unwrap();

        // 子图结构
        self.get_sub_ng().unwrap();

        // 构建 run_ng，返回 构建图
        let g = self.create_run_ng(&device, &queue)?;

        // 立即 执行 构建图
        match async_graph(rt.clone(), Arc::new(g)).await {
            Ok(_) => Ok(()),
            Err(e) => {
                let err = GraphError::RunNGraphError(format!("run_ng, {:?}", e));

                error!("{}", err);
                return Err(err);
            }
        }
    }

    /// 执行 渲染
    pub async fn run<A: 'static + AsyncRuntime + Send>(
        &mut self,
        rt: &A,
    ) -> Result<(), GraphError> {
        match self.run_ng {
            None => {
                let e = GraphError::NoneNGraph("run_ng".to_string());
                error!("{}", e);

                Err(e)
            }
            Some(ref g) => match async_graph(rt.clone(), g.clone()).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    let err = GraphError::RunNGraphError(format!("run_ng, {:?}", e));

                    error!("{}", err);
                    Err(err)
                }
            },
        }
    }
}

// ================== 以下方法 仅供 crate 使用

impl RenderGraph {
    fn get_node_id(&self, label: impl Into<NodeLabel>) -> Result<NodeId, GraphError> {
        let label = label.into();
        match label {
            NodeLabel::Id(id) => Ok(id),
            NodeLabel::Name(ref name) => self
                .node_names
                .get(name)
                .cloned()
                .ok_or(GraphError::NoneNode(label)),
        }
    }

    // 取 label 对应的 NodeState
    fn get_node_state(&self, label: impl Into<NodeLabel>) -> Option<&NodeState> {
        let id = self.get_node_id(label);
        match id {
            Ok(id) => self.nodes.get(id),
            Err(_) => None,
        }
    }

    // 取 拓扑子图，有必要就重新构建
    fn get_sub_ng(&mut self) -> Option<()> {
        // 子图没修改，返回 原图
        if !self.is_finish_dirty {
            return Some(());
        }
        self.is_finish_dirty = false;

        let finishes = self.clone_finish_nodes();

        let ng = self.topo_ng.as_ref().unwrap();

        // 以终为起，构建需要的 节点
        let sub_ng = ng.gen_graph_from_keys(&finishes);

        self.topo_sub_ng = Some(sub_ng);

        Some(())
    }

    // 取 拓扑图，有必要就重新构建
    fn get_topo_ng(&mut self) -> Option<()> {
        // 拓扑 没修改，返回 原图
        if !self.is_topo_dirty {
            return Some(());
        }
        self.is_topo_dirty = false;

        // 构建成功, ng_builder 就 删掉
        let mut builder = NGraphBuilder::<NodeId, NodeId>::new();

        // 节点 就是 高层添加 的 节点
        for (node_id, _) in &self.nodes {
            builder = builder.node(node_id, node_id);
        }

        for (before_id, after_id) in &self.edges {
            assert!(self.nodes.get(*after_id).is_some() && self.nodes.get(*before_id).is_some());

            // 和 渲染图 依赖 相反
            builder = builder.edge(*after_id, *before_id);
        }

        let ng = match builder.build() {
            Ok(ng) => ng,
            Err(e) => {
                error!("get_topo_ng, ng_builder.build error, e = {:?}", e);
                return None;
            }
        };

        self.topo_ng = Some(ng);
        Some(())
    }

    // 创建真正的 运行图
    // 返回 构建 的 执行图
    fn create_run_ng(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> Result<NGraph<NodeId, ExecNode<EmptySyncRun, EmptySyncRun>>, GraphError> {
        let mut build_builder =
            NGraphBuilder::<NodeId, ExecNode<EmptySyncRun, EmptySyncRun>>::new();
        let mut run_builder = NGraphBuilder::<NodeId, ExecNode<EmptySyncRun, EmptySyncRun>>::new();

        let sub_ng = self.topo_sub_ng.as_ref().unwrap();
        let topo_ids = sub_ng.topological_sort();

        // 异步图 节点
        for id in topo_ids {
            // 先重置 节点
            let n = self.get_node_state(*id).unwrap();
            n.0.as_ref().borrow_mut().reset();

            let node = self.create_run_node(*id, device, queue)?;
            run_builder = run_builder.node(*id, node);

            let node = self.create_build_node(*id, device, queue)?;
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

                let f_n = self.nodes.get_mut(from).unwrap().clone();

                // 为 to 天上 prenode = from
                let t_n = self.nodes.get_mut(to).unwrap();
                t_n.0.as_ref().borrow_mut().add_pre_node((from, f_n));
            }
        }

        match run_builder.build() {
            Ok(g) => {
                self.run_ng = Some(Share::new(g));
            }
            Err(e) => {
                let msg = format!("run_ng e = {:?}", e);
                error!("{}", msg);
                return Err(GraphError::BuildError(msg));
            }
        }

        // 构建图 只用 一次，用完 就 释放
        match build_builder.build() {
            Ok(g) => Ok(g),
            Err(e) => {
                let msg = format!("build_builder e = {:?}", e);
                error!("{}", msg);
                Err(GraphError::BuildError(msg))
            }
        }
    }

    // 创建 构建 节点
    fn create_build_node(
        &self,
        node_id: NodeId,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> Result<ExecNode<EmptySyncRun, EmptySyncRun>, GraphError> {
        let n = self.get_node_state(NodeLabel::Id(node_id));
        let node = n.unwrap().0.clone();

        let device = device.clone();
        let queue = queue.clone();

        // 该函数 会在 ng 图上，每帧每节点 执行一次
        let f = move || -> BoxFuture<'static, std::io::Result<()>> {
            let node = node.clone();
            let device = device.clone();
            let queue = queue.clone();

            async move {
                let context = RenderContext {
                    device: device.clone(),
                    queue: queue.clone(),
                };

                node.as_ref().borrow().build(context).await.unwrap();

                Ok(())
            }
            .boxed()
        };

        Ok(ExecNode::Async(Box::new(f)))
    }

    // 创建 渲染 节点
    fn create_run_node(
        &self,
        node_id: NodeId,
        device: &RenderDevice,
        queue: &RenderQueue,
    ) -> Result<ExecNode<EmptySyncRun, EmptySyncRun>, GraphError> {
        let n = self.get_node_state(NodeLabel::Id(node_id));
        let node = n.unwrap().0.clone();

        let device = device.clone();
        let queue = queue.clone();

        // 该函数 会在 ng 图上，每帧每节点 执行一次
        let f = move || -> BoxFuture<'static, std::io::Result<()>> {
            let node = node.clone();
            let device = device.clone();
            let queue = queue.clone();

            async move {
                let commands =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                let commands = ShareRefCell::new(commands);

                let context = RenderContext {
                    device: device.clone(),
                    queue: queue.clone(),
                };

                node.as_ref()
                    .borrow_mut()
                    .run(context, commands.clone())
                    .await
                    .unwrap();

                let commands = Share::try_unwrap(commands.0).unwrap();
                let commands = commands.into_inner();

                queue.submit(vec![commands.finish()]);

                Ok(())
            }
            .boxed()
        };

        Ok(ExecNode::Async(Box::new(f)))
    }

    // 供 GraphRunner 使用，和 使用者 无关
    fn clone_finish_nodes(&self) -> Vec<NodeId> {
        self.finish_nodes.iter().copied().collect()
    }
}

// 渲染图 不需要 同步节点，故这里写个空方法
struct EmptySyncRun;

impl Runner for EmptySyncRun {
    fn run(self) {}
}

impl RunFactory for EmptySyncRun {
    type R = EmptySyncRun;

    fn create(&self) -> Self::R {
        EmptySyncRun
    }
}
