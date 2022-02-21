//! 渲染图的 执行环境

use super::{
    graph::RenderGraph,
    node::NodeState,
    node_slot::{SlotInfos, SlotLabel, SlotType, SlotValue},
};
use crate::rhi::{
    buffer::Buffer,
    texture::{Sampler, TextureView},
};
use pi_ecs::entity::Entity;
use thiserror::Error;

/// 执行 [`Node`](super::Node) 所需要的 所有 graph 信息
///
/// 环境 由 `RenderGraphRunner` 创建
///
/// `node` 执行时，从 `inputs` 读 对应的 输入，产生 `output`，输出到 下个 [`Node`](super::Node)
pub struct RenderNodeContext<'a> {
    // 渲染图
    graph: &'a RenderGraph,
    // 对应的 RenderNode
    node: &'a NodeState,
    // 输入
    inputs: &'a [SlotValue],
    // 输出
    outputs: &'a mut [Option<SlotValue>],
}

impl<'a> RenderNodeContext<'a> {
    /// 创建 渲染节点 执行环境
    pub fn new(
        graph: &'a RenderGraph,
        node: &'a NodeState,
        inputs: &'a [SlotValue],
        outputs: &'a mut [Option<SlotValue>],
    ) -> Self {
        Self {
            graph,
            node,
            inputs,
            outputs,
        }
    }

    /// 返回 该环境的 输入
    #[inline]
    pub fn inputs(&self) -> &[SlotValue] {
        self.inputs
    }

    /// 返回 该环境的 输入 `SlotInfos`
    pub fn input_info(&self) -> &SlotInfos {
        &self.node.input_slots
    }

    /// 返回 该环境的 输出 `SlotInfos`
    pub fn output_info(&self) -> &SlotInfos {
        &self.node.output_slots
    }

    /// 返回 `label` 对应的 输入 `SlotValue`
    pub fn get_input(&self, label: impl Into<SlotLabel>) -> Result<&SlotValue, InputSlotError> {
        let label = label.into();
        let index = self
            .input_info()
            .get_slot_index(label.clone())
            .ok_or(InputSlotError::InvalidSlot(label))?;
        Ok(&self.inputs[index])
    }

    // TODO: should this return an Arc or a reference?
    /// Retrieves the input slot value referenced by the `label` as a [`TextureView`].
    pub fn get_input_texture(
        &self,
        label: impl Into<SlotLabel>,
    ) -> Result<&TextureView, InputSlotError> {
        let label = label.into();
        match self.get_input(label.clone())? {
            SlotValue::TextureView(value) => Ok(value),
            value => Err(InputSlotError::MismatchedSlotType {
                label,
                actual: value.slot_type(),
                expected: SlotType::TextureView,
            }),
        }
    }

    /// 返回 `label` 对应的 输入 [`Sampler`]
    /// 如其值非[`Sampler`]，返回 Err
    pub fn get_input_sampler(
        &self,
        label: impl Into<SlotLabel>,
    ) -> Result<&Sampler, InputSlotError> {
        let label = label.into();
        match self.get_input(label.clone())? {
            SlotValue::Sampler(value) => Ok(value),
            value => Err(InputSlotError::MismatchedSlotType {
                label,
                actual: value.slot_type(),
                expected: SlotType::Sampler,
            }),
        }
    }

    /// 返回 `label` 对应的 输入 [`Buffer`]
    /// 如其值非[`Buffer`]，返回 Err
    pub fn get_input_buffer(&self, label: impl Into<SlotLabel>) -> Result<&Buffer, InputSlotError> {
        let label = label.into();
        match self.get_input(label.clone())? {
            SlotValue::Buffer(value) => Ok(value),
            value => Err(InputSlotError::MismatchedSlotType {
                label,
                actual: value.slot_type(),
                expected: SlotType::Buffer,
            }),
        }
    }

    /// 返回 `label` 对应的 输入 [`Entity`]
    /// 如其值非[`Entity`]，返回 Err
    pub fn get_input_entity(&self, label: impl Into<SlotLabel>) -> Result<Entity, InputSlotError> {
        let label = label.into();
        match self.get_input(label.clone())? {
            SlotValue::Entity(value) => Ok(*value),
            value => Err(InputSlotError::MismatchedSlotType {
                label,
                actual: value.slot_type(),
                expected: SlotType::Entity,
            }),
        }
    }

    /// 设置 `label` 对应节点的 输出 值
    pub fn set_output(
        &mut self,
        label: impl Into<SlotLabel>,
        value: impl Into<SlotValue>,
    ) -> Result<(), OutputSlotError> {
        let label = label.into();
        let value = value.into();
        let slot_index = self
            .output_info()
            .get_slot_index(label.clone())
            .ok_or_else(|| OutputSlotError::InvalidSlot(label.clone()))?;
        let slot = self
            .output_info()
            .get_slot(slot_index)
            .expect("slot is valid");
        if value.slot_type() != slot.slot_type {
            return Err(OutputSlotError::MismatchedSlotType {
                label,
                actual: slot.slot_type,
                expected: value.slot_type(),
            });
        }
        self.outputs[slot_index] = Some(value);
        Ok(())
    }
}

/// 输出 Slot 错误
#[derive(Error, Debug, Eq, PartialEq)]
pub enum OutputSlotError {
    #[error("slot does not exist")]
    InvalidSlot(SlotLabel),
    #[error("attempted to assign the wrong type to slot")]
    MismatchedSlotType {
        label: SlotLabel,
        expected: SlotType,
        actual: SlotType,
    },
}

/// 输入 Slot 错误
#[derive(Error, Debug, Eq, PartialEq)]
pub enum InputSlotError {
    #[error("slot does not exist")]
    InvalidSlot(SlotLabel),
    #[error("attempted to retrieve the wrong type from input slot")]
    MismatchedSlotType {
        label: SlotLabel,
        expected: SlotType,
        actual: SlotType,
    },
}
