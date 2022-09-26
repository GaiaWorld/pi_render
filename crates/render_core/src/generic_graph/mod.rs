//! 渲染 通用图
//!
//! 主要数据结构
//!
//!     + enum GraphError

/// 图模型
pub mod graph;
/// 节点
pub mod node;
/// 节点 输入输出 参数
pub mod param;

pub use node::{NodeId, NodeLabel};
use thiserror::Error;

/// 图 执行的 错误
#[derive(Error, Debug, Eq, PartialEq)]
pub enum GraphError {
    #[error("ngraph is null: `{0}`")]
    NoneNGraph(String),

    #[error("node does not exist: label = `{0}`")]
    NoneNode(String),

    #[error("node is already exist, label = `{0}`")]
    ExitNode(String),

    #[error("run ngraph failed, reason = `{0}`")]
    RunNGraphError(String),

    #[error("build ng failed, reason = `{0}`")]
    BuildError(String),

    /// 运行 节点 的 build 方法 遇到的错误
    #[error("run GenericNode.build() failed, reason = `{0}`")]
    CustomBuildError(String),

    /// 运行 节点 的 run 方法 遇到的错误
    #[error("run GenericNode.run() failed, reason = `{0}`")]
    CustomRunError(String),

    #[error("node does not match the given type")]
    WrongNodeType,

    #[error("Input and output types do not match")]
    MismatchedParam,
}
