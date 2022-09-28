//! 利用 DependGraph 实现 渲染图
use super::{
    param::{InParam, OutParam},
    RenderContext,
};
use crate::depend_graph::node::DependNode;
use pi_futures::BoxFuture;
use pi_share::{Share, ShareRefCell, ThreadSync};
use wgpu::CommandEncoder;

pub use crate::depend_graph::node::{NodeId, NodeLabel, ParamUsage};

/// 渲染节点，给 外部 扩展 使用
pub trait Node: 'static + ThreadSync {
    /// 输入参数
    type Input: InParam + Default;

    /// 输出参数
    type Output: OutParam + Default + Clone;

    /// 构建，当渲染图 构建时候，会调用一次
    /// 一般 用于 准备 渲染 资源的 创建
    fn build<'a>(
        &'a self,
        _context: RenderContext,
        _usage: &'a ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        None
    }

    /// 执行，每帧会调用一次
    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>>;
}

// ====================== crate内 使用的 数据结构

pub(crate) struct NodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Input = I, Output = O>,
{
    node: R,
    context: RenderContext,
}

impl<I, O, R> NodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Input = I, Output = O>,
{
    #[inline]
    pub(crate) fn new(node: R, context: RenderContext) -> Self {
        Self { node, context }
    }
}

impl<I, O, R> DependNode for NodeImpl<I, O, R>
where
    I: InParam + Default,
    O: OutParam + Default + Clone,
    R: Node<Input = I, Output = O>,
{
    type Input = I;

    type Output = O;

    #[inline]
    fn build<'a>(&'a mut self, usage: &'a ParamUsage) -> Option<BoxFuture<'a, Result<(), String>>> {
        self.node.build(self.context.clone(), usage)
    }

    #[inline]
    fn run<'a>(
        &'a mut self,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        let context = self.context.clone();

        Box::pin(async move {
            // 每节点 一个 CommandEncoder
            let commands = self
                .context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            let commands = ShareRefCell::new(commands);

            let output = self
                .node
                .run(context, commands.clone(), input, usage)
                .await
                .unwrap();

            // CommandEncoder --> CommandBuffer
            let commands = Share::try_unwrap(commands.0).unwrap();
            let commands = commands.into_inner();

            // CommandBuffer --> Queue
            self.context.queue.submit(vec![commands.finish()]);

            Ok(output)
        })
    }
}
