use super::{
    graph::RenderGraph,
    node::{NodeId, NodeRunError, NodeState},
    node_slot::{SlotLabel, SlotType, SlotValue},
    RenderContext,
};
use crate::{
    graph::{context::RenderNodeContext, edge::Edge},
    rhi::{device::RenderDevice, RenderQueue},
};
use async_graph::async_graph;
use graph::{NGraph, NGraphBuilder};
use hash::XHashMap;
use pi_ecs::prelude::World;
use r#async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use std::{borrow::Cow, cell::RefCell, collections::VecDeque, sync::Arc};
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
    agraph: Arc<NGraph>,
}

impl<O, P> RenderGraphRunner<O, P>
where
    O: Default + 'static,
    P: AsyncTaskPoolExt<O> + AsyncTaskPool<O>,
{
    // 根据 渲染图 构建 执行器
    pub fn new(
        rt: AsyncRuntime<O, P>,
        render_graph: Arc<RefCell<RenderGraph>>,
        device: RenderDevice,
        queue: &wgpu::Queue,
        world: World,
    ) -> std::io::Result<Self> {
        let builder = NGraphBuilder::new();

        let res_mgr = XHashMap::new();

        // TODO 遍历每个 Node
        let commands = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        let mut render_context = RenderContext {
            device,
            commands,
            res_mgr,
        };
        queue.submit(vec![render_context.commands.finish()]);

        // TODO 从 终点 反向遍历每条边

        let agraph = builder.build()?;
        let agraph = Arc::new(agraph);
        Some(Self {
            rt,
            agraph,
            render_graph,
        })
    }

    pub async fn run(&mut self, queue: RenderQueue) {
        let agraph = self.agraph;
        let _ = async_graph(AsyncRuntime::Multi(self.rt.clone()), agraph).await;

        // TODO 执行 present
    }
}
