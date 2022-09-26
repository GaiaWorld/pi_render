//! 渲染图 模块
//! 
//! 主要类
//!     + struct GenerateGraph
//!     + struct RenderGraph
//!     + trait Node
//!     
pub mod graph;
pub mod node;
pub mod param;

use derive_deref_rs::{Deref};
pub use node::{NodeId, NodeLabel};
use pi_ecs::world::World;

use crate::rhi::{device::RenderDevice, RenderQueue};
use thiserror::Error;

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

/// 渲染图 执行过程中 遇到的 相关错误信息
#[derive(Error, Debug, Eq, PartialEq)]
pub enum GraphError {
    #[error("ngraph is null: `{0}`")]
    NoneNGraph(String),

    #[error("node does not exist")]
    NoneNode(NodeLabel),

    #[error("node is already exist")]
    ExitNode(NodeLabel),

    #[error("run ngraph failed, reason = `{0}`")]
    RunNGraphError(String),

    #[error("run custom node method failed, reason = `{0}`")]
    RunNodeError(String),

    #[error("build ng failed, reason = `{0}`")]
    BuildError(String),

    #[error("node does not match the given type")]
    WrongNodeType,

    #[error("Input and output types do not match")]
    MismatchedParam,
}

