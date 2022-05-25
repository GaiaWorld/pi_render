//! 渲染图

use super::{
    node::{Node, NodeId, NodeLabel, NodeState},
    RenderGraphError,
};
use log::error;
use pi_graph::{NGraph, NGraphBuilder};
use pi_hash::{XHashMap, XHashSet};
use std::{borrow::Cow, fmt::Debug};

/// 渲染图
pub struct RenderGraph<O: Clone> {
    // 当前 已经分配到的 id 数字
    nid_curr: usize,
    // 一旦构建成 ng后，ng_builder 就删掉
    pub(crate) ng_builder: Option<NGraphBuilder<NodeId, NodeId>>,
    // 只要 高层不改 链接关系，ng就一直用这个；
    pub(crate) ng: Option<NGraph<NodeId, NodeId>>,

    pub(crate) finish_nodes: XHashSet<NodeId>,

    nodes: XHashMap<NodeId, NodeState<O>>,
    node_names: XHashMap<Cow<'static, str>, NodeId>,
}

impl<O: Clone> Default for RenderGraph<O> {
    fn default() -> Self {
        Self {
            ng_builder: Some(NGraphBuilder::new()),
            nid_curr: 0,
            ng: None,
            finish_nodes: XHashSet::default(),
            nodes: XHashMap::default(),
            node_names: XHashMap::default(),
        }
    }
}

impl<O: Clone> RenderGraph<O> {
    pub fn clone_finish_nodes(&self) -> Vec<NodeId> {
        self.finish_nodes.iter().copied().collect()
    }

    /// 设置 是否 是 最终节点
    pub fn set_node_finish(
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

    /// 添加 节点
    pub fn add_node<T>(&mut self, name: impl Into<Cow<'static, str>>, node: T) -> NodeId
    where
        T: Node<Output = O>,
    {
        self.nid_curr += 1;
        let node_id = self.nid_curr;
        let name = name.into();

        let mut node_state = NodeState::new(node_id, node);
        node_state.name = Some(name.clone());
        self.node_names.insert(name, node_id);

        let mut builder = self.ng_builder.take().unwrap();
        builder = builder.node(node_id, node_id);

        self.ng_builder.replace(builder);
        self.nodes.insert(node_id, node_state);

        node_id
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
            Ok(id) => self.nodes.get(&id),
            Err(_) => None,
        }
    }

    pub fn get_node_mut(&mut self, label: impl Into<NodeLabel>) -> Option<&mut NodeState<O>> {
        let id = self.get_node_id(label);
        match id {
            Ok(id) => self.nodes.get_mut(&id),
            Err(_) => None,
        }
    }

    pub fn get_graph_impl(&mut self) -> Option<&mut NGraph<usize, usize>> {
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
}
