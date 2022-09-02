use super::{node::NodeId, GraphError};
use derive_deref_rs::Deref;
use pi_hash::XHashMap;
use std::{any::TypeId, mem::forget};

/// 渲染图节点的 参数，用于 派生宏
///
/// 注1：在 struct 定义 前 用 派生宏 展开 #[derive(NodeParam)]
/// 注2：派生宏 #[derive(NodeParam)] 后 可以 定义 属性宏 #[field_slot] 声明 里面每个 pub 字段 展开 NodeParam
/// 注3：用 #[field_slot] 展开 pub 字段的前提是，里面每个 参数 都要实现 #[derive(NodeParam)]
/// 注4：为了确保 该 结构体 可以 作为 输出，所以 派生宏 只为 NodeParam 实现，不为单独的 InParam 或 OutParam 实现
pub trait NodeParam: InParam + OutParam {}

/// 渲染图节点的 输入参数，用于 trait Node 的 关联类型 Input
pub trait InParam: 'static + Send + Sync {
    // 检查 输出参数 out_param 是否与 本输入 匹配
    // 由 节点 在 运行前 主动调用，每个 前置节点的 输出参数 调用一次
    fn is_in_macth<T: OutParam + ?Sized>(out_param: &T, ty: TypeId) -> bool;

    // 需要填充的数据的裸指针
    // 由 节点 在 运行前 主动调用，每个 前置节点的 输出参数 调用一次
    fn fill_from<T: OutParam + ?Sized>(&mut self, pre_id: NodeId, out_param: T, ty: TypeId);
}

/// 渲染图节点的 输出参数，用于 trait Node 的 关联类型 Output
pub trait OutParam: 'static + Send + Sync {
    /// 检查 类型 是否与 本参数 匹配
    fn is_out_macth(&self, ty: TypeId) -> bool;

    /// 将数据填充 到 类型 ty 对应的 字段
    fn fill_to(self, next_id: NodeId, ty: TypeId);
}

// 为基本类型实现 InParam, OutParam
macro_rules! impl_base {
    ($ty: ty) => {
        impl InParam for $ty {
            unsafe fn fill(&mut self, src_id: NodeId, src_ptr: usize, ty: TypeId) {
                if ty == TypeId::of::<$ty>() {
                    *self = std::ptr::read(src_ptr as *const $ty);
                }
            }

            fn check_macth(&self, ty: TypeId) -> bool {
                ty == TypeId::of::<$ty>()
            }
        }

        impl OutParam for $ty {
            fn fill_to<T: InParam + ?Sized>(self, src_id: NodeId, target: &mut T) {
                unsafe { target.fill(src_id, &self as *const $ty as usize, TypeId::of::<Self>()) };
                // 忘记self
                std::mem::forget(self);
            }

            fn check_macths<T: InParam + ?Sized>(target: &T) -> bool {
                target.check_macth(TypeId::of::<Self>())
            }
        }
    };
}

// /// 输入参数 收集器
// /// 如 某个节点A 的 多个前置节点 的 输出类型 都是 T，那么可以 在 A 的 输入中 指定 InCollector<T>，将多个前置节点输出的T收集起来
// /// 哈希表 Key = 前置节点的 NodeId，值 = 该 前置节点的输出
// #[derive(Debug, Default, Clone, Deref)]
// pub struct InCollector<T: OutParam>(pub XHashMap<NodeId, T>);

// impl<T: OutParam> InParam for InCollector<T> {
//     unsafe fn fill(&mut self, src_id: NodeId, src_ptr: usize, ty: TypeId) {
//         if ty == TypeId::of::<T>() {
//             self.0.insert(src_id, std::ptr::read(src_ptr as *const T));
//         }
//     }

//     fn is_macth(&self, ty: TypeId) -> bool {
//         if ty == TypeId::of::<T>() {
//             return Ok(());
//         }
//         Err(GraphError::MismatchedParam)
//     }
// }