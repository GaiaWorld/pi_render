//! 渲染图 模块
//!
//! 主要类
//!     + struct RenderContext
//!
pub mod graph;
pub mod node;
pub mod param;

use crate::rhi::{device::RenderDevice, RenderQueue};
use pi_ecs::world::World;

/// 渲染图 执行过程中 遇到的 相关错误信息
pub use crate::depend_graph::GraphError;

pub use node::{NodeId, NodeLabel};

/// 渲染图 执行过程需要的环境
#[derive(Clone)]
pub struct RenderContext {
    /// 渲染 设备，用于 创建资源
    pub device: RenderDevice,

    /// 队列，用于 创建 和 提交 CommandEncoder
    pub queue: RenderQueue,

    /// ECS world
    pub world: World,
}
