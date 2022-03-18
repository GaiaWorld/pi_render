//! 渲染图

use super::{
    node::{Node, NodeId, NodeLabel, NodeState},
    node_slot::{SlotId, SlotLabel},
    RenderGraphError,
};
use pi_graph::NGraphBuilder;
use pi_hash::{XHashMap, XHashSet};
use std::{borrow::Cow, fmt::Debug};

pub type NGNodeKey = usize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NGNodeValue {
    Node(NodeId),

    InputSlot(NodeId, SlotId),

    OutputSlot(NodeId, SlotId),
}

impl NGNodeValue {
    /// 创建 节点
    pub fn new_with_node(id: NodeId) -> Self {
        NGNodeValue::Node(id)
    }

    /// 创建 输入
    pub fn new_with_input(node: NodeId, slot: SlotId) -> Self {
        NGNodeValue::InputSlot(node, slot)
    }

    /// 创建 输出
    pub fn new_with_output(node: NodeId, slot: SlotId) -> Self {
        NGNodeValue::OutputSlot(node, slot)
    }
}

/// 渲染图
#[derive(Default)]
pub struct RenderGraph {
    // 当前 已经分配到的 id 数字
    nid_curr: NGNodeKey,
    pub(crate) ng_builder: Option<NGraphBuilder<NGNodeKey, NGNodeValue>>,

    pub(crate) finish_nodes: XHashSet<NodeId>,

    nodes: XHashMap<NodeId, NodeState>,
    node_names: XHashMap<Cow<'static, str>, NodeId>,
    slots: XHashMap<NGNodeValue, NGNodeKey>,
}

impl RenderGraph {
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
        T: Node,
    {
        self.nid_curr += 1;
        let node_id = self.nid_curr;
        let name = name.into();

        let mut node_state = NodeState::new(node_id, node);
        node_state.name = Some(name.clone());
        self.node_names.insert(name, node_id);

        let mut builder = self.ng_builder.take().unwrap();
        builder = builder.node(node_id, NGNodeValue::new_with_node(node_id));

        for (index, _) in node_state.node.input().iter().enumerate() {
            self.nid_curr += 1;
            let input_id = self.nid_curr;

            let node_value = NGNodeValue::InputSlot(node_id, index);
            self.slots.insert(node_value.clone(), input_id);
            builder = builder.node(input_id, node_value);

            // 这里要注意，NGraph 和  RenderGraph 的依赖 是相反的
            builder = builder.edge(node_id, input_id);
        }

        for (index, _) in node_state.node.output().iter().enumerate() {
            self.nid_curr += 1;
            let output_id = self.nid_curr;

            let node_value = NGNodeValue::OutputSlot(node_id, index);
            self.slots.insert(node_value.clone(), output_id);
            builder = builder.node(output_id, node_value);

            // 这里要注意，NGraph 和  RenderGraph 的依赖 是相反的
            builder = builder.edge(output_id, node_id);
        }

        self.ng_builder.replace(builder);
        self.nodes.insert(node_id, node_state);

        node_id
    }

    /// 建立 两个节点的 Slot 之间的 边 [`Edge::SlotEdge`]
    /// 建立 顺序 `output_node` 先于 `input_node` 执行
    pub fn add_slot_edge(
        &mut self,

        output_node: impl Into<NodeLabel>,
        output_slot: impl Into<SlotLabel>,

        input_node: impl Into<NodeLabel>,
        input_slot: impl Into<SlotLabel>,
    ) -> Result<(), RenderGraphError> {
        let input_node = input_node.into();
        let input_slot = input_slot.into();

        let input_node_id = self.get_node_id(input_node.clone())?;
        let input_slot_id = match self.get_node(input_node.clone()) {
            None => {
                return Err(RenderGraphError::InvalidInputNodeSlot(
                    input_node, input_slot,
                ));
            }
            Some(n) => match n.input_slot_id(input_slot.clone()) {
                None => {
                    return Err(RenderGraphError::InvalidInputNodeSlot(
                        input_node, input_slot,
                    ));
                }
                Some(id) => id,
            },
        };

        let ng_input = NGNodeValue::new_with_input(input_node_id, input_slot_id);
        let ng_input = self
            .slots
            .get(&ng_input)
            .ok_or(RenderGraphError::InvalidInputNodeSlot(
                input_node, input_slot,
            ))?;

        let output_node = output_node.into();
        let output_slot = output_slot.into();
        let output_node_id = self.get_node_id(output_node.clone())?;
        let output_slot_id = match self.get_node(output_node.clone()) {
            None => {
                return Err(RenderGraphError::InvalidOutputNodeSlot(
                    output_node,
                    output_slot,
                ));
            }
            Some(n) => match n.output_slot_id(output_slot.clone()) {
                None => {
                    return Err(RenderGraphError::InvalidOutputNodeSlot(
                        output_node,
                        output_slot,
                    ));
                }
                Some(id) => id,
            },
        };
        let ng_output = NGNodeValue::new_with_output(output_node_id, output_slot_id);
        let ng_output =
            self.slots
                .get(&ng_output)
                .ok_or(RenderGraphError::InvalidOutputNodeSlot(
                    output_node,
                    output_slot,
                ))?;

        let mut builder = self.ng_builder.take().unwrap();

        // 和 渲染圖 依賴 相反
        builder = builder.edge(*ng_input, *ng_output);

        self.ng_builder.replace(builder);

        Ok(())
    }

    /// 建立 两个节点的 Slot 之间的 边 [`Edge::NodeEdge`]
    /// 建立 顺序 `output_node` 先于 `input_node` 执行
    pub fn add_node_edge(
        &mut self,
        output_node: impl Into<NodeLabel>,
        input_node: impl Into<NodeLabel>,
    ) -> Result<(), RenderGraphError> {
        let output_node_id = self.get_node_id(output_node)?;
        let input_node_id = self.get_node_id(input_node)?;

        let mut builder = self.ng_builder.take().unwrap();

        // 和 渲染圖 依賴 相反
        builder = builder.edge(input_node_id, output_node_id);

        self.ng_builder.replace(builder);

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

    pub fn get_node(&self, label: impl Into<NodeLabel>) -> Option<&NodeState> {
        let id = self.get_node_id(label);
        match id {
            Ok(id) => self.nodes.get(&id),
            Err(_) => None,
        }
    }

    pub fn get_node_mut(&mut self, label: impl Into<NodeLabel>) -> Option<&mut NodeState> {
        let id = self.get_node_id(label);
        match id {
            Ok(id) => self.nodes.get_mut(&id),
            Err(_) => None,
        }
    }
}
