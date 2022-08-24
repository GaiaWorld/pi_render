use std::{any::TypeId, mem::forget};

use derive_deref_rs::Deref;
use pi_hash::XHashMap;
use pi_share::Share;

use crate::graph::node::Node;

use super::{node::{GraphNode, NodeId}, RenderGraphError};

/// 参数， 输入输出参数需要实现该trait
pub trait Param: FillTarget + FillSrc {}

/// 参数(输入或输出)
/// * 安全：src_ptr指针必须有效， 并与ty类型对应
pub trait FillTarget: Send + 'static + Sync {
	/// 将数据填充到对应字段
	unsafe fn fill(&mut self, src_id: NodeId, src_ptr: usize, ty: TypeId);
	/// 检查类型是否匹配
	fn check_macth(&self, ty: TypeId) -> Result<(), RenderGraphError>;
}

pub trait FillSrc: Send + 'static + Sync {
	// 需要填充的数据的裸指针
	fn fill_to<T: FillTarget + ?Sized>(self, src_id: NodeId, target: &mut T);

	// 逐个检查每个字段是否与目标匹配
	fn check_macths<T: FillTarget + ?Sized>(target: &T) -> Result<(), RenderGraphError>;
}

// 为基本类型实现FillTarget， FillSrc
macro_rules! impl_base {
    ($ty: ty) => {
		impl FillTarget for $ty {
			unsafe fn fill(&mut self, src_id: NodeId, src_ptr: usize, ty: TypeId) {
				if ty == TypeId::of::<$ty>() {
					*self = std::ptr::read(src_ptr as *const $ty);
				}
			}
		
			fn check_macth(&self, ty: TypeId) -> Result<(), RenderGraphError> {
				if ty == TypeId::of::<$ty>() {
					return Ok(());
				}
				Err(RenderGraphError::MismatchedParam)
			}
		}
		
		
		impl FillSrc for $ty {
			fn fill_to<T: FillTarget + ?Sized>(self, src_id: NodeId, target: &mut T) {
				unsafe { target.fill(src_id, &self as *const $ty as usize, TypeId::of::<Self>()) };
				// 忘记self
				std::mem::forget(self);
			}
		
			fn check_macths<T: FillTarget + ?Sized>(target: &T) -> Result<(), RenderGraphError> {
				target.check_macth(TypeId::of::<Self>())
			}
		}
	};
}

/// 参数收集器，作为FillTarget，与普通FillTarget不同的是，其匹配输入类型时，不是作为整体进行匹配，而是匹配其泛型参数
/// ParamCollector<T>将T类型的输入，以来源的NodeId为Key，插入Map中
#[derive(Debug, Clone, Deref)]
pub struct ParamCollector<T: FillSrc> (XHashMap<NodeId, T>);

impl<T: FillSrc> Default for ParamCollector<T> {
    fn default() -> Self {
        Self(XHashMap::default())
    }
}

impl<T: FillSrc> FillTarget for ParamCollector<T> {
	unsafe fn fill(&mut self, src_id: NodeId, src_ptr: usize, ty: TypeId) {
		if ty == TypeId::of::<T>() {
			self.0.insert(src_id, std::ptr::read(src_ptr as *const T));
		}
	}

	fn check_macth(&self, ty: TypeId) -> Result<(), RenderGraphError> {
		if ty == TypeId::of::<T>() {
			return Ok(());
		}
		Err(RenderGraphError::MismatchedParam)
	}
}
