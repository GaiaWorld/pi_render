use super::{
    graph::{NGNodeValue, RenderGraph},
    node::{NodeId, NodeLabel, NodeRunError, RealValue},
    node_slot::{SlotLabel, SlotType},
    RenderContext,
};
use crate::{
    render_graph::{node::NodeState, node_slot::SlotId},
    rhi::{device::RenderDevice, RenderQueue},
};
use async_graph::{async_graph, ExecNode, RunFactory, Runner};
use futures::{future::BoxFuture, FutureExt};
use graph::{DirectedGraph, DirectedGraphNode, NGraph, NGraphBuilder};
use hash::XHashMap;
use log::error;
use pi_ecs::prelude::World;
use r#async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use std::{borrow::Cow, cell::RefCell, io::Read, sync::Arc};
use thiserror::Error;
use wgpu::CommandEncoder;

#[derive(Error, Debug)]
pub enum RenderGraphRunnerError {
    #[error(transparent)]
    NodeRunError(#[from] NodeRunError),
    #[error("node output slot not set (index {slot_index}, name {slot_name})")]
    EmptyNodeOutputSlot {
        type_name: &'static str,
        slot_index: usize,
        slot_name: Cow<'static, str>,
    },
    #[error("graph (name: '{graph_name:?}') could not be run because slot '{slot_name}' at index {slot_index} has no value")]
    MissingInput {
        slot_index: usize,
        slot_name: Cow<'static, str>,
        graph_name: Option<Cow<'static, str>>,
    },
    #[error("attempted to use the wrong type for input slot")]
    MismatchedInputSlotType {
        slot_index: usize,
        label: SlotLabel,
        expected: SlotType,
        actual: SlotType,
    },
}

// 渲染图 执行器
pub(crate) struct RenderGraphRunner<P>
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    // 异步运行时
    rt: AsyncRuntime<(), P>,
    // 渲染图
    render_graph: RenderGraph,

    // prepare 异步图，build 阶段 用一次
    prepare_graph: Option<Arc<NGraph<NGNodeValue, ExecNode<DumpNode, DumpNode>>>>,

    // run 异步图，一直用，直到 render_graph 改变为止
    run_graph: Option<Arc<NGraph<NGNodeValue, ExecNode<DumpNode, DumpNode>>>>,
}

impl<P> RenderGraphRunner<P>
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    /// 创建
    pub fn new(rt: AsyncRuntime<(), P>, render_graph: RenderGraph) -> Self {
        Self {
            rt,
            render_graph,
            prepare_graph: None,
            run_graph: None,
        }
    }

    /// 构建
    pub fn build(
        &mut self,
        device: RenderDevice,
        queue: RenderQueue,
        world: World,
    ) -> Result<(), String> {
        let ng_builder = self.render_graph.ng_builder.take().unwrap();
        let ng = match ng_builder.build() {
            Ok(ng) => ng,
            Err(e) => return Err(format!("render build error: {:?}", e)),
        };

        // 遍历，找 终点
        let mut finishes = Vec::<NodeId>::new();
        for node in ng.iter() {
            match node {
                NGNodeValue::Node(n) => {
                    let node = self.render_graph.get_node(NodeLabel::Id(*n)).unwrap();
                    if node.node.is_finish() {
                        finishes.push(*n);
                    }
                }
                _ => {}
            }
        }

        // 以终为起，构建需要的 节点
        let sub_ng = ng.gen_graph_from_keys(&finishes);

        // 构造异步图
        let mut run_builder = NGraphBuilder::new();
        let mut prepare_builder = NGraphBuilder::new();

        fn create_slots_vec(n: &NodeState) -> (Vec<Option<RealValue>>, Vec<Option<RealValue>>) {
            let len = n.input_slots.len();
            let mut inputs = Vec::<Option<RealValue>>::with_capacity(len);
            for i in 0.. {
                inputs.push(None);
            }

            let len = n.output_slots.len();
            let mut outputs = Vec::<Option<RealValue>>::with_capacity(len);
            for i in 0..len {
                outputs.push(None);
            }

            (inputs, outputs)
        }

        // 从 sub_ng 直接分析 资源依赖
        let mut map = XHashMap::default();
        for id in sub_ng.topological_sort() {
            let ng_node = sub_ng.get(id).unwrap();
            if let NGNodeValue::OutputSlot(nid, sid) = ng_node.value() {
                let slots = map.entry(nid).or_insert_with(|| {
                    let n = self.render_graph.get_node(*nid).unwrap();
                    create_slots_vec(n)
                });

                slots.1[*sid] = Some(RealValue::default());
            }
        }

        for id in sub_ng.topological_sort() {
            let ng_node = sub_ng.get(id).unwrap();
            if let NGNodeValue::InputSlot(nid, sid) = ng_node.value() {
                let slots = map.entry(nid).or_insert_with(|| {
                    let n = self.render_graph.get_node(*nid).unwrap();
                    create_slots_vec(n)
                });

                for to in ng_node.to() {
                    let to = sub_ng.get(to).unwrap();
                    if let NGNodeValue::OutputSlot(next_node, next_slot) = to.value() {
                        let v = map.get(next_node).unwrap().1[*next_slot].unwrap().clone();
                        slots.0[*sid] = Some(v);
                    }
                }
            }
        }

        // 异步图 节点
        for id in sub_ng.topological_sort() {
            let ng_node = sub_ng.get(id).unwrap();
            let ng_node_clone = ng_node.value().clone();
            match ng_node.value() {
                NGNodeValue::Node(n) => {
                    prepare_builder = prepare_builder.node(
                        ng_node_clone.clone(),
                        crate_prepare_node(&map, &self.render_graph, *n, device, queue, world),
                    );

                    run_builder = run_builder.node(
                        ng_node_clone,
                        crate_run_node(&map, &self.render_graph, *n, device, queue, world),
                    );
                }
                NGNodeValue::InputSlot(nid, sid) => {
                    prepare_builder = prepare_builder.node(ng_node_clone.clone(), ExecNode::None);
                    run_builder = run_builder.node(ng_node_clone, ExecNode::None);
                }
                NGNodeValue::OutputSlot(nid, sid) => {
                    prepare_builder = prepare_builder.node(ng_node_clone.clone(), ExecNode::None);
                    run_builder = run_builder.node(ng_node_clone, ExecNode::None);
                }
            }
        }

        // 异步图 边
        for id in sub_ng.topological_sort() {
            let ng_node = sub_ng.get(id).unwrap();
            for to in ng_node.to() {
                let next_node = sub_ng.get(id).unwrap();

                prepare_builder =
                    prepare_builder.edge(ng_node.value().clone(), next_node.value().clone());

                run_builder = run_builder.edge(ng_node.value().clone(), next_node.value().clone());
            }
        }

        let value_map = XHashMap::<NGNodeValue, RealValue>::default();

        match prepare_builder.build() {
            Ok(g) => {
                self.prepare_graph = Some(Arc::new(g));
            }
            Err(e) => {
                error!(
                    "render_graph::build prepare_builder graph failed, reason = {:?}",
                    e
                );
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
            }
            Some(ref g) => {
                let _ = async_graph(self.rt.clone(), g.clone()).await;
            }
        }

        // 移除 prepare，因为它只能执行一次
        let _ = self.prepare_graph.take();
    }

    /// 执行
    pub async fn run(&mut self) {
        match self.run_graph {
            None => {
                error!("render_graph::run failed, run_graph is none");
            }
            Some(ref g) => {
                let _ = async_graph(self.rt.clone(), g.clone()).await;
            }
        }
    }
}

// 异步图: 哑节点，异步函数不需要的类型
struct DumpNode;
impl Runner for DumpNode {
    fn run(self) {}
}
impl RunFactory for DumpNode {
    type R = DumpNode;
    fn create(&self) -> Self::R {
        DumpNode
    }
}

#[derive(Clone, Debug)]
pub struct CommandEncoderWrap(pub Arc<RefCell<Option<CommandEncoder>>>);
unsafe impl Sync for CommandEncoderWrap {}
unsafe impl Send for CommandEncoderWrap {}

// 创建异步 节点
fn crate_run_node(
    map: &XHashMap<&usize, (Vec<Option<RealValue>>, Vec<Option<RealValue>>)>,
    render_graph: &RenderGraph,
    n: usize,
    device: RenderDevice,
    queue: RenderQueue,
    world: World,
) -> ExecNode<DumpNode, DumpNode> {
    let node = render_graph.get_node(NodeLabel::Id(n)).unwrap();
    let node = node.node.clone();

    let (inputs, outputs) = map.get(&n).unwrap();
    let inputs = inputs.as_slice().to_vec();
    let outputs = outputs.as_slice().to_vec();

    let f = move || -> BoxFuture<'static, std::io::Result<()>> {
        let device = device.clone();
        let queue = queue.clone();
        let world = world.clone();
        let node = node.clone();
        let inputs = inputs.clone();
        let outputs = outputs.clone();

        let context = RenderContext {
            world,
            device,
            queue,
        };

        async move {
            let commands =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            let commands = CommandEncoderWrap(Arc::new(RefCell::new(Some(commands))));

            let runner = node.run(
                context,
                commands.clone(),
                inputs.as_slice(),
                outputs.as_slice(),
            );

            runner.await;

            let mut lck = commands.0.get_mut();
            let commands = std::mem::take(lck).unwrap();
            context.queue.submit(vec![commands.finish()]);
            Ok(())
        }
        .boxed()
    };

    ExecNode::Async(Box::new(f))
}

// 创建异步 节点
fn crate_prepare_node(
    map: &XHashMap<&usize, (Vec<Option<RealValue>>, Vec<Option<RealValue>>)>,
    render_graph: &RenderGraph,
    n: usize,
    device: RenderDevice,
    queue: RenderQueue,
    world: World,
) -> ExecNode<DumpNode, DumpNode> {
    let node = render_graph.get_node(NodeLabel::Id(n)).unwrap();
    let node = node.node.clone();

    let (inputs, outputs) = map.get(&n).unwrap();
    let inputs = inputs.as_slice().to_vec();
    let outputs = outputs.as_slice().to_vec();

    let f = move || -> BoxFuture<'static, std::io::Result<()>> {
        let device = device.clone();
        let queue = queue.clone();
        let world = world.clone();
        let node = node.clone();
        let inputs = inputs.clone();
        let outputs = outputs.clone();

        async move {
            let context = RenderContext {
                world,
                device,
                queue,
            };

            match node.prepare(context, inputs.as_slice(), outputs.as_slice()) {
                None => {}
                Some(r) => {
                    r.await;
                }
            }

            Ok(())
        }
        .boxed()
    };

    return ExecNode::Async(Box::new(f));
}
