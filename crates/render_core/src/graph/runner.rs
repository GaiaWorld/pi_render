use super::{
    graph::RenderGraph,
    node::{NodeId, NodeLabel, NodeOutputType, NodeRunError},
    RenderContext,
};
use crate::rhi::{device::RenderDevice, RenderQueue};
use futures::{future::BoxFuture, FutureExt};
use log::{error, info};
use pi_async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use pi_async_graph::{async_graph, ExecNode, RunFactory, Runner};
use pi_ecs::prelude::World;
use pi_graph::{DirectedGraph, DirectedGraphNode, NGraph, NGraphBuilder};
use pi_hash::XHashMap;
use pi_share::ShareRefCell;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderGraphRunnerError {
    #[error(transparent)]
    NodeRunError(#[from] NodeRunError),
}

// 渲染图 执行器
pub struct RenderGraphRunner<O, P>
where
    O: NodeOutputType,
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    // 异步运行时
    rt: AsyncRuntime<(), P>,

    // 读写锁
    pub(crate) slot_lock: Arc<Mutex<XHashMap<NodeId, Vec<O>>>>,

    // 互斥锁
    pub(crate) finish_lock: Arc<Mutex<()>>,

    // prepare 异步图，build 阶段 用一次
    pub(crate) prepare_graph: Option<Arc<NGraph<NodeId, ExecNode<DumpNode, DumpNode>>>>,

    // run 异步图，一直用，直到 render_graph 改变为止
    pub(crate) run_graph: Option<Arc<NGraph<NodeId, ExecNode<DumpNode, DumpNode>>>>,
}

impl<O, P> RenderGraphRunner<O, P>
where
    O: NodeOutputType,
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    /// 创建
    pub fn new(rt: AsyncRuntime<(), P>) -> Self {
        Self {
            rt,
            prepare_graph: None,
            run_graph: None,

            slot_lock: Arc::new(Mutex::new(XHashMap::default())),

            finish_lock: Arc::new(Mutex::new(())),
        }
    }

    /// 构建
    pub fn build(
        &mut self,
        world: World,
        device: RenderDevice,
        queue: RenderQueue,
        rg: &mut RenderGraph<O>,
    ) -> Result<(), String> {
        if self.prepare_graph.is_some() && self.run_graph.is_some() {
            info!("RenderGraphRunner::build prepare and run graph is ready, prepare is_some = {}, run = {}", self.prepare_graph.is_some(), self.run_graph.is_some());
            return Ok(());
        }

        // 遍历，找 终点
        let finishes = rg.clone_finish_nodes();

        let ng = rg.get_graph_impl().unwrap();

        // 以终为起，构建需要的 节点
        let sub_ng = ng.gen_graph_from_keys(&finishes);

        // 构造异步图
        let mut run_builder = NGraphBuilder::<NodeId, ExecNode<DumpNode, DumpNode>>::new();
        let mut prepare_builder = NGraphBuilder::<NodeId, ExecNode<DumpNode, DumpNode>>::new();

        // 重新构建 输入输出
        self.build_slots(rg, &sub_ng);

        let sub_rg = sub_ng.topological_sort();

        // 异步图 节点
        for id in sub_rg {
            prepare_builder = prepare_builder.node(
                *id,
                crate_prepare_node(rg, *id, device.clone(), queue.clone(), world.clone()),
            );

            run_builder = run_builder.node(
                *id,
                crate_run_node(
                    self.slot_lock.clone(),
                    self.finish_lock.clone(),
                    rg,
                    *id,
                    device.clone(),
                    queue.clone(),
                    world.clone(),
                ),
            );
        }

        // 异步图 边
        // sub_ng 是 以终为起的，所以 sub_ng 的 from 和 to 必须和 执行顺序 相反；
        for id in sub_ng.topological_sort() {
            let to = sub_ng.get(id).unwrap();
            for from in to.to() {
                let from = sub_ng.get(from).unwrap();

                let from = *from.value();
                let to = *to.value();

                prepare_builder = prepare_builder.edge(from, to);

                run_builder = run_builder.edge(from, to);
            }
        }

        match prepare_builder.build() {
            Ok(g) => {
                self.prepare_graph = Some(Arc::new(g));
            }
            Err(e) => {
                error!(
                    "render_graph::build prepare_builder graph failed, reason = {:?}",
                    e
                );
                panic!("render_graph::build prepare_builder graph failed");
            }
        };

        match run_builder.build() {
            Ok(g) => {
                self.run_graph = Some(Arc::new(g));
            }
            Err(e) => {
                error!(
                    "render_graph::build run_builder graph failed, reason = {:?}",
                    e
                );
                panic!("render_graph::build run_builder graph failed");
            }
        };

        Ok(())
    }

    /// 每个节点 调用 prepare 方法
    /// 目的：创建资源
    pub async fn prepare(&mut self) {
        match self.prepare_graph {
            None => {
                error!("render_graph::prepare failed, prepare_graph is none");
                panic!("render_graph::prepare failed");
            }
            Some(ref g) => {
                let ag = async_graph(self.rt.clone(), g.clone());
                println!("@@@@@@@@@@@@@@ ");
				ag.await.unwrap();
            }
        }

        // 移除 prepare，因为它只能执行一次
        let t = self.prepare_graph.take();
        t.unwrap();
    }

    /// 执行
    pub async fn run(&mut self) {
		println!("run===================");
        match self.run_graph {
            None => {
                error!("render_graph::run failed, run_graph is none");
                panic!("render_graph::run failed");
            }
            Some(ref g) => {
                let r = async_graph(self.rt.clone(), g.clone()).await;
                r.unwrap();
            }
        }
    }

    // 重新 构建 输入输出
    fn build_slots(&mut self, render_graph: &mut RenderGraph<O>, sub_ng: &NGraph<NodeId, NodeId>) {
        let sub_rg = sub_ng.topological_sort();
        for id in sub_rg {
            let node = render_graph.get_node_mut(NodeLabel::Id(*id)).unwrap();
            node.clear_input_output();
        }

        for to in sub_rg {
            let from_ids = sub_ng.get(to).unwrap().to();

            {
                let mut v = Vec::with_capacity(from_ids.len());
                for _ in from_ids {
                    v.push(O::default());
                }

                self.slot_lock.lock().unwrap().insert(*to, v);
            }

            for (id, from) in from_ids.iter().enumerate() {
                let from_node = render_graph.get_node_mut(NodeLabel::Id(*from)).unwrap();
                from_node.output.push((*to, id));
            }
        }
    }
}

// 异步图: 哑节点，异步函数不需要的类型
pub struct DumpNode;
impl Runner for DumpNode {
    fn run(self) {}
}
impl RunFactory for DumpNode {
    type R = DumpNode;
    fn create(&self) -> Self::R {
        DumpNode
    }
}

// 创建异步 节点
fn crate_run_node<O: NodeOutputType>(
    slot_lock: Arc<Mutex<XHashMap<NodeId, Vec<O>>>>,
    finish_lock: Arc<Mutex<()>>,
    render_graph: &RenderGraph<O>,
    node_id: NodeId,
    device: RenderDevice,
    queue: RenderQueue,
    world: World,
) -> ExecNode<DumpNode, DumpNode> {
    let node = render_graph.get_node(NodeLabel::Id(node_id)).unwrap();

    let output_slots = node.output.clone();

    let node = node.node.clone();

    let f = move || -> BoxFuture<'static, std::io::Result<()>> {
        let device = device.clone();
        let queue = queue.clone();
        let world = world.clone();
        let node = node.clone();

        let finish_lock = finish_lock.clone();
        let output_slots = output_slots.clone();
        let slot_lock = slot_lock.clone();

        let context = RenderContext {
            world: world.clone(),
            device: device.clone(),
            queue: queue.clone(),
        };

        let context1 = RenderContext {
            world,
            device: device.clone(),
            queue: queue.clone(),
        };

        async move {
            let commands =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            let finish_lock = finish_lock.clone();
            let output_slots = output_slots.clone();
            let slot_lock = slot_lock.clone();

            let commands = ShareRefCell::new(commands);

            let inputs = { slot_lock.lock().unwrap().remove(&node_id).unwrap() };

            let runner = node.run(context, commands.clone(), inputs.as_slice());
            let output = runner.await.unwrap();
            {
                let mut slots = slot_lock.lock().unwrap();
                for (oid, oslot) in output_slots {
                    let a = slots.get_mut(&oid).unwrap().get_mut(oslot).unwrap();
                    *a = output.clone();
                }
            }

            let commands = Arc::try_unwrap(commands.0).unwrap();
            let commands = commands.into_inner();

            queue.submit(vec![commands.finish()]);

            {
                finish_lock.lock().unwrap();
                let runner = node.finish(context1, inputs.as_slice());
                let _ = runner.await.unwrap();
            }

            Ok(())
        }
        .boxed()
    };

    ExecNode::Async(Box::new(f))
}

// 创建异步 节点
fn crate_prepare_node<O: NodeOutputType>(
    render_graph: &RenderGraph<O>,
    node_id: NodeId,
    device: RenderDevice,
    queue: RenderQueue,
    world: World,
) -> ExecNode<DumpNode, DumpNode> {
    let node = render_graph.get_node(NodeLabel::Id(node_id)).unwrap();
    let node = node.node.clone();

    let f = move || -> BoxFuture<'static, std::io::Result<()>> {
        let device = device.clone();
        let queue = queue.clone();
        let world = world.clone();
        let node = node.clone();

        async move {
            let context = RenderContext {
                world,
                device,
                queue,
            };

            match node.prepare(context) {
                None => {}
                Some(r) => {
                    r.await.unwrap();
                }
            }

            Ok(())
        }
        .boxed()
    };

    ExecNode::Async(Box::new(f))
}
