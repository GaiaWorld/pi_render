use crate::quad::RenderItem;
use futures::future::BoxFuture;
use pi_ecs::prelude::{QueryState, World};
use pi_render::{
    graph::{
        node::{Node, NodeRunError, RealValue},
        node_slot::SlotInfo,
        RenderContext,
    },
    phase::RenderPhase,
};

pub struct ScenePass {
    query: QueryState<(&RenderPhase<RenderItem>,)>,
}

impl ScenePass {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl Node for ScenePass {
    /// 返回 输入 槽位 信息
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    /// 返回 输出 槽位 信息
    fn output(&self) -> Vec<SlotInfo> {
        vec![]
    }

    /// 异步 创建 gpu 资源
    /// 该函数 在 所有节点的 run方法 之前
    /// 存放资源的地方
    /// 资源可以来自 渲染图 之外，也可以来自 渲染节点
    fn prepare(
        &self,
        _context: RenderContext,
        _inputs: &[Option<RealValue>],
        _outputs: &[Option<RealValue>],
    ) -> Option<BoxFuture<'static, Result<(), NodeRunError>>> {
        None
    }

    /// 异步执行 渲染方法
    /// 一个渲染节点，通常是 开始 RenderPass
    /// 将 渲染指令 录制在 wgpu 中
    fn run(
        &self,
        context: pi_render::graph::RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        inputs: &[Option<pi_render::graph::node::RealValue>],
        outputs: &[Option<pi_render::graph::node::RealValue>],
    ) -> futures::future::BoxFuture<'static, Result<(), pi_render::graph::node::NodeRunError>> {
        
    }
}
