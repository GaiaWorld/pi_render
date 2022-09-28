//! RenderGraph
//!

use super::{
    node::{Node, NodeId, NodeImpl, NodeLabel},
    param::{InParam, OutParam},
    GraphError, RenderContext,
};
use crate::{
    depend_graph::graph::DependGraph,
    rhi::{device::RenderDevice, RenderQueue},
};
use pi_async::prelude::AsyncRuntime;
use pi_ecs::world::World;
use std::borrow::Cow;

/// 渲染图
pub struct RenderGraph {
    world: World,
    device: RenderDevice,
    queue: RenderQueue,

    imp: DependGraph,
}

/// 渲染图的 拓扑信息 相关 方法
impl RenderGraph {
    /// 创建
    pub fn new(world: World, device: RenderDevice, queue: RenderQueue) -> Self {
        Self {
            world,
            device,
            queue,
            imp: Default::default(),
        }
    }

    /// 添加 名为 name 的 节点
    #[inline]
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
        let context = RenderContext {
            world: self.world.clone(),
            device: self.device.clone(),
            queue: self.queue.clone(),
        };

        let node = NodeImpl::new(node, context);

        self.imp.add_node(name, node)
    }

    /// 移除 节点
    pub fn remove_node(&mut self, label: impl Into<NodeLabel>) -> Result<(), GraphError> {
        self.imp.remove_node(label)
    }

    /// 建立 Node 的 依赖关系
    /// 执行顺序 `before` 先于 `after`
    #[inline]
    pub fn add_depend(
        &mut self,
        before_label: impl Into<NodeLabel>,
        before_slot: impl Into<Cow<'static, str>>,
        after_label: impl Into<NodeLabel>,
        after_slot: impl Into<Cow<'static, str>>,
    ) -> Result<(), GraphError> {
        self.imp
            .add_depend(before_label, before_slot, after_label, after_slot)
    }

    /// 移除依赖
    #[inline]
    pub fn remove_depend(
        &mut self,
        before_label: impl Into<NodeLabel>,
        before_slot: impl Into<Cow<'static, str>>,
        after_label: impl Into<NodeLabel>,
        after_slot: impl Into<Cow<'static, str>>,
    ) -> Result<(), GraphError> {
        self.imp
            .remove_depend(before_label, before_slot, after_label, after_slot)
    }

    /// 设置 是否 是 最终节点
    #[inline]
    pub fn set_finish(
        &mut self,
        label: impl Into<NodeLabel>,
        is_finish: bool,
    ) -> Result<(), GraphError> {
        self.imp.set_finish(label, is_finish)
    }
}

/// 渲染图的 执行 相关
impl RenderGraph {
    #[inline]
    pub fn build(&mut self) -> Result<(), GraphError> {
        self.imp.build()
    }

    /// 执行 渲染
    #[inline]
    pub async fn run<A: 'static + AsyncRuntime + Send>(
        &mut self,
        rt: &A,
    ) -> Result<(), GraphError> {
        self.imp.run(rt).await
    }
}
