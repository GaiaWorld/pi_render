//! 参数 的 展开逻辑
//!
//! 节点1 有 struct Output {A, B}
//! 节点2 有 struct Input {A, C}
//!
//! 节点2 是 节点1 的 后继
//!
//! 节点1 被 抽象成 dyn InternalNode
//!
//! Input.fill_from(1.id, <dyn InternalNode>1);
//!     A.fill_from(1.id, <dyn InternalNode>1);
//!         let a = <dyn InternalNode>1.get_content(typeid A);
//!               = Output.get_content(typeid A);
//!         Input.A = ptr::read(a);
//!     C.fill_from(1.id, <dyn InternalNode>1);
//!         let c = <dyn InternalNode>1.get_content(typeid C);
//!               = Output.get_content(typeid C);
//!         c 为 0
//!
//! Output.get_content(typeid A) -> uszie
//!     let c = A.get_content(typeid A)
//!     if c !== 0 { return c }
//!     let c = B.get_content(typeid A)
//!
//! A.get_content(id)  {
//!     if id == typeof(A) {
//!         let c = Clone::clone(*self);
//!         std::mem::forget(c);
//!         &c as *const A as usize
//!     } else {
//!         0
//!     }
//! }

use super::node::NodeId;
use pi_hash::XHashMap;
use std::any::TypeId;

/// 渲染图节点的 参数，用于 派生宏
///
/// 注1：在 struct 定义 前 用 派生宏 展开 #[derive(NodeParam)]
/// 注2：派生宏 #[derive(NodeParam)] 后 可以 定义 属性宏 #[field_slots] 声明 里面每个 pub 字段 展开 NodeParam
/// 注3：用 #[field_slots] 展开 pub 字段的前提是，里面每个 参数 都要实现 #[derive(NodeParam)]
/// 注4：为了确保 该 结构体 可以 作为 输出，所以 派生宏 只为 NodeParam 实现，不为单独的 InParam 或 OutParam 实现
pub trait NodeParam: InParam + OutParam {}

impl<T: InParam + OutParam> NodeParam for T {}

/// 渲染图节点的 输入参数，用于 trait Node 的 关联类型 Input
pub trait InParam: 'static + Send + Sync {
    // 由 节点 在 运行前 主动调用，每个 前置节点的 输出参数 调用一次
    fn fill_from<T: OutParam + ?Sized>(&mut self, pre_id: NodeId, out_param: &T) -> bool;
}

/// 渲染图节点的 输出参数，用于 trait Node 的 关联类型 Output
pub trait OutParam: 'static + Send + Sync {
    /// clone 一个 指定 ty 类型 的 指针返回
    /// 如果不匹配，则返回 0
    fn get_content(&self, ty: TypeId) -> usize;
}

/// 输入参数 收集器
/// 如 某个节点A 的 多个前置节点 的 输出类型 都是 T，那么可以 在 A 的 输入中 指定 InCollector<T>，将多个前置节点输出的T收集起来
/// 哈希表 Key = 前置节点的 NodeId，值 = 该 前置节点的输出
#[derive(Debug, Default)]
pub struct InCollector<T: OutParam>(pub XHashMap<NodeId, T>);

impl<T: OutParam + Default> InParam for InCollector<T> {
    fn fill_from<Ty: OutParam + ?Sized>(&mut self, pre_id: NodeId, out_param: &Ty) -> bool {
        let v = out_param.get_content(TypeId::of::<T>());
        if v != 0 {
            let v = unsafe { std::ptr::read(v as *const T) };
            self.0.insert(pre_id, v);
        }

        v != 0
    }
}

// 为基本类型实现 InParam
// 为 实现了 Copy 的 基本类型 实现 OutParam
macro_rules! impl_base_copy {
    ($ty: ty) => {
        impl InParam for $ty {
            fn fill_from<T: OutParam + ?Sized>(&mut self, _: NodeId, out_param: &T) -> bool {
                let v = out_param.get_content(TypeId::of::<Self>());
                println!("impl_base_copy, InParam ty = $ty, v = {}", v);
                if v != 0 {
                    *self = unsafe { std::ptr::read(v as *const Self) };
                }

                v != 0
            }
        }

        impl OutParam for $ty {
            fn get_content(&self, ty: TypeId) -> usize {
                println!("impl_base_copy, OutParam ty = $ty");
                if ty == TypeId::of::<Self>() {
                    // Self 必须 实现 Copy
                    let c = *self;
                    
                    let p = &c as *const Self as usize;
                    
                    p
                } else {
                    0
                }
            }
        }
    };
}

// 为基本类型实现 InParam
// 为 实现了 Clone 没有实现 Copy 的 基本类型 实现 OutParam
macro_rules! impl_base_noncopy {
    ($ty: ty) => {
        impl InParam for $ty {
            fn fill_from<T: OutParam + ?Sized>(&mut self, _: NodeId, out_param: &T) -> bool {
                let v = out_param.get_content(TypeId::of::<Self>());
                println!("impl_base_noncopy, InParam ty = $ty, v = {}", v);
                if v != 0 {
                    *self = unsafe { std::ptr::read(v as *const Self) };
                }

                v != 0
            }
        }

        impl OutParam for $ty {
            fn get_content(&self, ty: TypeId) -> usize {
                println!("impl_base_noncopy, OutParam ty = $ty");
                if ty == TypeId::of::<Self>() {
                    // 为了让外部调用者 更清楚的知道 必须为 Self 实现 Clone
                    let c = Clone::clone(self);
                    
                    let p = &c as *const Self as usize;
                    
                    // 注: Copy 和 Drop 不能 共存
                    // 不能 释放放这个 c，因为 c 是要拿去 填充 输入的
                    std::mem::forget(c);
                    
                    p
                } else {
                    0
                }
            }
        }
    };
}

impl_base_noncopy!(String);

impl_base_copy!(());
impl_base_copy!(bool);
impl_base_copy!(i8);
impl_base_copy!(i16);
impl_base_copy!(i32);
impl_base_copy!(i64);
impl_base_copy!(i128);
impl_base_copy!(u8);
impl_base_copy!(u16);
impl_base_copy!(u32);
impl_base_copy!(u64);
impl_base_copy!(u128);
impl_base_copy!(f32);
impl_base_copy!(f64);