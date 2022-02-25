//! 节点 对应的 槽位 Slot
//!
//! 主要概念
//!
//! 渲染节点 槽(Slot) 有 3种： [`Buffer`], [`TextureView`], [`Sampler`]
//!
//! + [`SlotValue`]
//! + [`SlotType`]
//! + [`SlotLabel`]
//! + [`SlotInfo`]
//! + [`SlotInfos`]

use crate::rhi::{
    buffer::Buffer,
    texture::{Sampler, TextureView},
};
use std::borrow::Cow;

/// 用于 在 [`Nodes`](super::Node) 传递的 值
/// 对应 [`RenderGraph`](super::RenderGraph) 的 [`SlotType`]
///
/// Slots 由 3种不同的值 [`Buffer`], [`TextureView`], [`Sampler`]
#[derive(Debug, Clone)]
pub enum SlotValue {
    /// GPU [`Buffer`].
    Buffer(Buffer),
    /// [`TextureView`] 描述 在 Pipeline 使用 的 Texture
    TextureView(TextureView),
    /// 纹理 [`Sampler`] 定义 管线 如何 采样 [`TextureView`].
    Sampler(Sampler),
}

impl SlotValue {
    /// 返回 对应的 类型
    pub fn slot_type(&self) -> SlotType {
        match self {
            SlotValue::Buffer(_) => SlotType::Buffer,
            SlotValue::TextureView(_) => SlotType::TextureView,
            SlotValue::Sampler(_) => SlotType::Sampler,
        }
    }
}

impl From<Buffer> for SlotValue {
    fn from(value: Buffer) -> Self {
        SlotValue::Buffer(value)
    }
}

impl From<TextureView> for SlotValue {
    fn from(value: TextureView) -> Self {
        SlotValue::TextureView(value)
    }
}

impl From<Sampler> for SlotValue {
    fn from(value: Sampler) -> Self {
        SlotValue::Sampler(value)
    }
}

/// Slot 类型
///
/// 被 渲染 [`Nodes`](super::Node) 写(output) 或 读(input) 的 渲染资源 类型
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SlotType {
    /// GPU [`Buffer`].
    Buffer,
    /// [`TextureView`] 描述 在 Pipeline 使用 的 Texture
    TextureView,
    /// 纹理 [`Sampler`] 定义 管线 如何 采样 [`TextureView`]
    Sampler,
}

pub type SlotId = usize;

/// [`SlotLabel`] 用于 从 名字 或 位置 来 引用
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SlotLabel {
    /// 位置索引
    Index(SlotId),
    /// 名字
    Name(Cow<'static, str>),
}

impl From<&SlotLabel> for SlotLabel {
    fn from(value: &SlotLabel) -> Self {
        value.clone()
    }
}

impl From<String> for SlotLabel {
    fn from(value: String) -> Self {
        SlotLabel::Name(value.into())
    }
}

impl From<&'static str> for SlotLabel {
    fn from(value: &'static str) -> Self {
        SlotLabel::Name(value.into())
    }
}

impl From<Cow<'static, str>> for SlotLabel {
    fn from(value: Cow<'static, str>) -> Self {
        SlotLabel::Name(value.clone())
    }
}

impl From<usize> for SlotLabel {
    fn from(value: usize) -> Self {
        SlotLabel::Index(value)
    }
}

/// Slot 的 内部表示，描述 [`SlotType`] 和 名字
#[derive(Clone, Debug)]
pub struct SlotInfo {
    /// Slot 名
    pub name: Cow<'static, str>,
    /// Slot 类型
    pub slot_type: SlotType,
}

impl SlotInfo {
    /// 用 名字 和 类型 创建 SlotInfo
    pub fn new(name: impl Into<Cow<'static, str>>, slot_type: SlotType) -> Self {
        SlotInfo {
            name: name.into(),
            slot_type,
        }
    }
}

/// SlotInfo 的 集合，供 [`NodeSlot`] 使用
#[derive(Default, Debug)]
pub struct SlotInfos {
    slots: Vec<SlotInfo>,
}

impl<T: IntoIterator<Item = SlotInfo>> From<T> for SlotInfos {
    fn from(slots: T) -> Self {
        SlotInfos {
            slots: slots.into_iter().collect(),
        }
    }
}

impl SlotInfos {
    /// 返回 有多少个 槽位
    #[inline]
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// 如果 没有 槽位，返回 true
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// 返回 label 对应的 [`SlotInfo`]
    pub fn get_slot(&self, label: impl Into<SlotLabel>) -> Option<&SlotInfo> {
        let label = label.into();
        let index = self.get_slot_index(&label)?;
        self.slots.get(index)
    }

    /// 返回 label 对应的 [`SlotInfo`]
    pub fn get_slot_mut(&mut self, label: impl Into<SlotLabel>) -> Option<&mut SlotInfo> {
        let label = label.into();
        let index = self.get_slot_index(&label)?;
        self.slots.get_mut(index)
    }

    /// 返回 label 对应的 索引值
    pub fn get_slot_index(&self, label: impl Into<SlotLabel>) -> Option<usize> {
        let label = label.into();
        match label {
            SlotLabel::Index(index) => Some(index),
            SlotLabel::Name(ref name) => self
                .slots
                .iter()
                .enumerate()
                .find(|(_i, s)| s.name == *name)
                .map(|(i, _s)| i),
        }
    }

    /// 返回 迭代器
    pub fn iter(&self) -> impl Iterator<Item = &SlotInfo> {
        self.slots.iter()
    }
}
