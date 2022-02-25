//! 渲染图

use super::{
    node::{Node, NodeId, NodeLabel, NodeState},
    node_slot::{SlotId, SlotInfo, SlotLabel},
    RenderGraphError,
};
use graph::{NGraph, NGraphBuilder};
use hash::XHashMap;
use nalgebra::SimdValue;
use std::{borrow::Cow, fmt::Debug};

pub type NGNodeKey = usize;

#[derive(Debug, Clone, Hash)]
pub enum NGNodeValue {
    Node(NodeState),

    InputSlot(NodeId, SlotId),

    OutputSlot(NodeId, SlotId),
}

impl NGNodeValue {
    /// 创建 节点
    pub fn new_with_node(state: NodeState) -> Self {
        NGNodeValue::Node(state)
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
    // 实际上 是 render-graph 的 反向图
    ng: Option<NGraph<NGNodeKey, NGNodeValue>>,
    ng_builder: Option<NGraphBuilder<NGNodeKey, NGNodeValue>>,

    slot_map: XHashMap<NGNodeValue, NGNodeKey>,
    node_names: XHashMap<Cow<'static, str>, NodeId>,
}

impl RenderGraph {
    /// 创建
    pub fn new() -> Self {
        Self {
            nid_curr: 0,
            ng: None,
            ng_builder: Some(NGraphBuilder::new()),
            slot_map: XHashMap::new(),
            node_names: XHashMap::new(),
        }
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
        builder = builder.node(node_id, NGNodeValue::new_Node(node_state));

        for (index, _) in node.input().iter().enumarate() {
            self.nid_curr += 1;
            let input_id = self.nid_curr;

            let node_value = NGNodeValue::InputSlot(node_id, index);
            self.slot_map.insert(node_value.clone(), input_id);
            builder = builder.node(input_id, node_value);

            // 这里要注意，NGraph 和  RenderGraph 的依赖 是相反的
            builder = builder.edge(node_id, input_id);
        }

        for (index, _) in node.output().iter().enumarate() {
            self.nid_curr += 1;
            let output_id = self.nid_curr;

            let node_value = NGNodeValue::OutputSlot(node_id, index);
            self.slot_map.insert(node_value.clone(), output_id);
            builder = builder.node(output_id, node_value);

            // 这里要注意，NGraph 和  RenderGraph 的依赖 是相反的
            builder = builder.edge(output_id, node_id);
        }

        self.ng_builder.replace(builder);

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
        let input_slot = input_slot.into();
        let input_node_id = self.get_node_id(input_node)?;

        let ng_input = NGNodeValue::new_with_input(input_node_id, input_slot);
        let ng_input =
            self.slot_map
                .get(&ng_input)
                .ok_or(RenderGraphError::InvalidInputNodeSlot(
                    input_node, input_slot,
                ))?;

        let output_slot = output_slot.into();
        let output_node_id = self.get_node_id(output_node)?;

        let ng_output = NGNodeValue::new_with_output(output_node_id, output_slot);
        let ng_output =
            self.slot_map
                .get(&ng_output)
                .ok_or(RenderGraphError::InvalidOutputNodeSlot(
                    output_node,
                    output_slot,
                ))?;

        let mut builder = self.ng_builder.take().unwrap();

        // 和 渲染圖 依賴 相反
        builder = builder.edge(ng_input, ng_output);

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
}
