//! 渲染 通用图
//!
//! 主要数据结构
//!
//!     + enum GraphError

/// 图模型
pub mod graph;
/// 子图模型
pub mod sub_graph;
pub mod sub_graph_data;
/// 节点
pub mod node;
/// 节点 输入输出 参数
pub mod param;
pub mod graph_data;

use graphviz_rust::dot_structures::Node;
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

    #[error("sub graph input node more than 1")]
    SubGraphInputError,

    #[error("sub graph finish node more than 1")]
    SubGraphOutputError,

    /// 运行 节点 的 build 方法 遇到的错误
    #[error("run DependNode.build() failed, reason = `{0}`")]
    CustomBuildError(String),

    /// 运行 节点 的 run 方法 遇到的错误
    #[error("run DependNode.run() failed, reason = `{0}`")]
    CustomRunError(String),

    #[error("node does not match the given type")]
    WrongNodeType,

    #[error("Input and output types do not match")]
    MismatchedParam,

	#[error("node does not in one graph: before = `{0}`, after: {1:?}")]
	CrossGraphDepend(String, String),

    #[error("param fill with repeat, from: {0:?} {1:?}, to: {2:?}")]
    ParamFillRepeat(NodeId, NodeId, NodeId),
}
