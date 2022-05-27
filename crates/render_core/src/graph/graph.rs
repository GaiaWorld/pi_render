//! 渲染图

use super::{
    node::{Node, NodeId, NodeLabel, NodeState, NodeOutputType},
    RenderGraphError,
};
use log::error;
use pi_graph::{NGraph, NGraphBuilder};
use pi_hash::{XHashMap, XHashSet};
use pi_slotmap::SlotMap;
use std::borrow::Cow;

/// 渲染图
pub struct RenderGraph<O: NodeOutputType> {
    // 一旦构建成 ng后，ng_builder 就删掉
    pub(crate) ng_builder: Option<NGraphBuilder<NodeId, NodeId>>,
    // 只要 高层不改 链接关系，ng就一直用这个；
    pub(crate) ng: Option<NGraph<NodeId, NodeId>>,
    // 最终节点，渲染到屏幕的节点
    pub(crate) finish_nodes: XHashSet<NodeId>,
    // 所有 节点的 集合
    nodes: SlotMap<NodeId, NodeState<O>>,
    // 名字 和 NodeId 映射
    node_names: XHashMap<Cow<'static, str>, NodeId>,
}

impl<O: NodeOutputType> Default for RenderGraph<O> {
    fn default() -> Self {
        Self {
            ng_builder: Some(NGraphBuilder::new()),
            ng: None,
            finish_nodes: XHashSet::default(),
            nodes: SlotMap::default(),
            node_names: XHashMap::default(),
        }
    }
}

impl<O: NodeOutputType> RenderGraph<O> {
    /// 添加 节点
    pub fn add_node<T>(&mut self, name: impl Into<Cow<'static, str>>, node: T) -> NodeId
    where
        T: Node<Output = O>,
    {
        let name = name.into();

        let mut node_state = NodeState::new(node);
        node_state.name = Some(name.clone());
        let node_id = self.nodes.insert(node_state);

        self.node_names.insert(name, node_id);

        let mut builder = self.ng_builder.take().unwrap();
        builder = builder.node(node_id, node_id);

        self.ng_builder.replace(builder);
        
        node_id
    }

    /// 建立 Node 的 依赖关系
    /// 执行顺序 `before` 先于 `after`
    pub fn set_depend(
        &mut self,
        before: impl Into<NodeLabel>,
        after: impl Into<NodeLabel>,
    ) -> Result<(), RenderGraphError> {
        let before_id = self.get_node_id(before)?;
        let after_id = self.get_node_id(after)?;

        let mut builder = self.ng_builder.take().unwrap();

        // 和 渲染圖 依賴 相反
        builder = builder.edge(after_id, before_id);

        self.ng_builder.replace(builder);

        Ok(())
    }

    /// 设置 是否 是 最终节点
    pub fn set_finish(
        &mut self,
        node: impl Into<NodeLabel>,
        is_finish: bool,
    ) -> Result<(), RenderGraphError> {
        let node = node.into();
        let node_id = self.get_node_id(node)?;

        if is_finish {
            self.finish_nodes.insert(node_id);
        } else {
            self.finish_nodes.remove(&node_id);
        }

        Ok(())
    }

    pub fn get_node_id(&self, label: impl Into<NodeLabel>) -> Result<NodeId, RenderGraphError> {
        let label = label.into();
        match label {
            NodeLabel::Id(id) => Ok(id),
            NodeLabel::Name(ref name) => self
                .node_names
                .get(name)
                .cloned()
                .ok_or(RenderGraphError::InvalidNode(label)),
        }
    }

    pub fn get_node(&self, label: impl Into<NodeLabel>) -> Option<&NodeState<O>> {
        let id = self.get_node_id(label);
        match id {
            Ok(id) => self.nodes.get(id),
            Err(_) => None,
        }
    }

    pub fn get_node_mut(&mut self, label: impl Into<NodeLabel>) -> Option<&mut NodeState<O>> {
        let id = self.get_node_id(label);
        match id {
            Ok(id) => self.nodes.get_mut(id),
            Err(_) => None,
        }
    }

    pub fn get_graph_impl(&mut self) -> Option<&mut NGraph<NodeId, NodeId>> {
        if self.ng.is_none() {
            let ng_builder = self.ng_builder.take().unwrap();
            let ng = match ng_builder.build() {
                Ok(ng) => ng,
                Err(e) => {
                    error!("get_graph_impl, ng_builder.build error, e = {:?}", e);
                    return None;
                }
            };
            self.ng = Some(ng);
        }
        self.ng.as_mut()
    }

    pub fn reset(&mut self) -> Result<(), RenderGraphError> {
        if self.ng.is_some() && self.ng_builder.is_none() {
            let ng = self.ng.take().unwrap();
            self.ng_builder = Some(NGraphBuilder::new_with_graph(ng));
            Ok(())
        } else {
            Err(RenderGraphError::NoneNGraph)
        }
    }

    /// 供 GraphRunner 使用，和 使用者 无关
    pub fn clone_finish_nodes(&self) -> Vec<NodeId> {
        self.finish_nodes.iter().copied().collect()
    }
}
