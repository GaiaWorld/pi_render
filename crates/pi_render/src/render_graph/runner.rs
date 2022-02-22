use super::{
    graph::RenderGraph,
    node::{NodeId, NodeRunError, NodeState},
    node_slot::{SlotLabel, SlotType, SlotValue},
    RenderContext,
};
use crate::rhi::{device::RenderDevice, RenderQueue};
use async_graph::{async_graph, ExecNode, RunFactory, Runner};
use futures::future::BoxFuture;
use graph::{NGraph, NGraphBuilder};
use hash::XHashMap;
use pi_ecs::prelude::World;
use r#async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use smallvec::SmallVec;
use std::{borrow::Cow, cell::RefCell, sync::Arc};
use thiserror::Error;

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
pub(crate) struct RenderGraphRunner<O, P>
where
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
{
    // 异步运行时
    rt: AsyncRuntime<O, P>,
    // 渲染图, RefCell 是为了 外部 可以 根据情况 修改
    render_graph: Arc<RefCell<RenderGraph>>,
    // 异步图, build 实现
    agraph: Option<Arc<NGraph<NodeId, ExecNode<DumpNode, DumpNode>>>>,
}

impl<O, P> RenderGraphRunner<O, P>
where
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
{
    /// 创建
    pub fn new(rt: AsyncRuntime<O, P>, render_graph: Arc<RefCell<RenderGraph>>) -> Self {
        Self {
            rt,
            render_graph,
            agraph: None,
        }
    }

    /// 构建
    pub fn build(
        &mut self,
        device: RenderDevice,
        queue: RenderQueue,
        world: World,
    ) -> std::io::Result<()> {
        let mut builder = NGraphBuilder::new();

        let res_mgr = XHashMap::<NodeId, SmallVec<SlotValue>>::new();

        // 最终的 渲染节点
        let mut finish_nodes: Vec<&NodeState> = vec![];

        // 扫描出 最终的 渲染节点
        for node in self.render_graph.borrow().iter_nodes() {
            if node.node.is_finish() {
                finish_nodes.push(node);
            }
        }
        
        for node in finish_nodes {
            let id = node.id;
            
            // TODO
            builder = builder.node(id, asyn(...));

            for output in node.node.output() {
                builder = builder.node(id, ExecNode::None);
            }
        }

        let agraph = builder.build()?;
        self.agraph = Arc::new(agraph);
    }

    /// 执行
    pub async fn run(&mut self) {
        let agraph = self.agraph;
        
        let _ = async_graph(AsyncRuntime::Multi(self.rt.clone()), agraph).await;

        // TODO 执行 present
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

// 异步调用
fn asyn(
    world: World,
    node: &NodeState,
    device: RenderDevice,
    queue: RenderQueue,
    res_mgr: XHashMap<NodeId, SmallVec<SlotValue>>,
) -> ExecNode<DumpNode, DumpNode> {

    let runner = node.node.run;

    let f = move || -> BoxFuture<'static, Result<()>> {
        let commands = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        async move {
            let mut render_context = RenderContext {
                device,
                commands,
                res_mgr,
            };
            
            runner(world, render_context);

            queue.submit(vec![render_context.commands.finish()]);

            Ok(())
        }
        .boxed()
    };

    ExecNode::Async(Box::new(f))
}
