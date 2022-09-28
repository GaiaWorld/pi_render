//! 图节点的 输入输出 参数
//!
//! 主要数据结构：
//!
//!     + trait InParam
//!     + trait OutParam
//!     + 输入 收集器 InParamCollector
//!

use super::node::NodeId;
use pi_hash::{XHashMap, XHashSet};
use pi_share::ThreadSync;
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
pub trait InParam: 'static + ThreadSync + Assign {
    // 返回 out_param 能否 填充 本参数
    fn can_fill<O: OutParam + ?Sized>(
        &self,
        map: &mut XHashMap<TypeId, Vec<NodeId>>,
        pre_id: NodeId,
        out_param: &O,
    ) -> bool;

    // 由 节点 在 运行前 主动调用，每个 前置节点的 输出参数 调用一次
    fn fill_from<O: OutParam + ?Sized>(&mut self, pre_id: NodeId, out_param: &O) -> bool;
}

/// 渲染图节点的 输出参数，用于 trait Node 的 关联类型 Output
pub trait OutParam: 'static + ThreadSync {
    /// 判断 本 参数 能否 填充 ty
    fn can_fill(&self, set: &mut Option<&mut XHashSet<TypeId>>, ty: TypeId) -> bool;

    fn fill_to(&self, this_id: NodeId, to: &mut dyn Assign, ty: TypeId) -> bool;
}

// 赋值
pub trait Assign {
    fn assign(&mut self, pre_id: NodeId, ptr: usize);
}

/// 输入参数 收集器
/// 如 某个节点A 的 多个前置节点 的 输出类型 都是 T，那么可以 在 A 的 输入中 指定 InCollector<T>，将多个前置节点输出的T收集起来
/// 哈希表 Key = 前置节点的 NodeId，值 = 该 前置节点的输出
#[derive(Debug, Default, Clone)]
pub struct InParamCollector<T: OutParam + Clone>(pub XHashMap<NodeId, T>);

impl<T: OutParam + Clone> InParam for InParamCollector<T> {
    fn can_fill<O: OutParam + ?Sized>(
        &self,
        map: &mut XHashMap<TypeId, Vec<NodeId>>,
        pre_id: NodeId,
        out_param: &O,
    ) -> bool {
        let ty = TypeId::of::<T>();
        let r = out_param.can_fill(&mut None, ty.clone());
        if r {
            let v = map.entry(ty).or_insert(vec![]);
            v.push(pre_id);
        }
        r
    }

    fn fill_from<O: OutParam + ?Sized>(&mut self, pre_id: NodeId, out_param: &O) -> bool {
        out_param.fill_to(pre_id, self, TypeId::of::<T>())
    }
}

impl<T: OutParam + Clone> OutParam for InParamCollector<T> {
    fn can_fill(&self, _set: &mut Option<&mut XHashSet<TypeId>>, _ty: TypeId) -> bool {
        panic!("can_fillInParamCollector can't as Output");
    }

    fn fill_to(&self, _this_id: NodeId, _to: &mut dyn Assign, _ty: TypeId) -> bool {
        panic!("fill_to InParamCollector can't as Output");
    }
}

impl<T: OutParam + Clone> Assign for InParamCollector<T> {
    fn assign(&mut self, pre_id: NodeId, ptr: usize) {
        if ptr != 0 {
            let v = unsafe { std::ptr::read(ptr as *const T) };
            self.0.insert(pre_id, v);
        }
    }
}

impl<T> Assign for T {
    default fn assign(&mut self, _: NodeId, ptr: usize) {
        if ptr != 0 {
            let v = unsafe { std::ptr::read(ptr as *const T) };
            *self = v;
        }
    }
}

// 为基本类型实现 InParam
// 为 实现了 Copy 的 基本类型 实现 OutParam
macro_rules! impl_base_copy {
    // 用逗号 将 多个类型 impl_base_copy!(i8, i16, i32, f32)
    ($first:ty, $($rest:ty),+) => {
        impl_base_copy!($first);
        $(
            impl_base_copy!($rest);
        )+
    };

    // 基本情况，只有一个参数时
    ($typ:ty) => {
        impl InParam for $typ {
            fn can_fill<O: OutParam + ?Sized>(
                &self,
                map: &mut XHashMap<TypeId, Vec<NodeId>>,
                pre_id: NodeId,
                out_param: &O,
            ) -> bool {
                let ty = TypeId::of::<Self>();
                let r = out_param.can_fill(&mut None, ty.clone());
                if r {
                    if map.get(&ty).is_some() {
                        // 输入 类型 不能 相同
                        panic!("impl_base_copy: input type same, type = {:?}", ty);
                    }
                    map.insert(ty, vec![pre_id]);
                }
                r
            }

            fn fill_from<O: OutParam + ?Sized>(&mut self, pre_id: NodeId, out_param: &O) -> bool {
                out_param.fill_to(pre_id, self, TypeId::of::<Self>())
            }
        }

        impl OutParam for $typ {
            fn can_fill(&self, set: &mut Option<&mut XHashSet<TypeId>>, ty: TypeId) -> bool {
                let r = ty == TypeId::of::<Self>();
                if r && set.is_some() {
                    match set {
                        None => {}
                        Some(s) => {
                            s.insert(ty);
                        }
                    }
                }
                r
            }

            fn fill_to(&self, this_id: NodeId, to: &mut dyn Assign, ty: TypeId) -> bool {
                let r = ty == TypeId::of::<Self>();
                if r {
                    // Self 必须 实现 Copy
                    let c = *self;
                    let p = &c as *const Self as usize;
                    to.assign(this_id, p);
                }
                r
            }
        }
    };
}

// 为基本类型实现 InParam
// 为 实现了 Clone 没有实现 Copy 的 基本类型 实现 OutParam
macro_rules! impl_base_noncopy {
    // 用逗号 将 多个类型 串起来调用，例子：impl_base_noncopy!(Arc, Rc, String)
    ($first:ty, $($rest:ty),+) => {
        impl_base_noncopy!($first);
        $(
            impl_base_noncopy!($rest);
        )+
    };

    // 基本情况，只有一个参数时
    ($typ: ty) => {
        impl InParam for $typ {
            fn can_fill<O: OutParam + ?Sized>(
                &self,
                map: &mut XHashMap<TypeId, Vec<NodeId>>,
                pre_id: NodeId,
                out_param: &O,
            ) -> bool {
                let ty = TypeId::of::<Self>();
                let r = out_param.can_fill(&mut None, ty.clone());
                if r {
                    if map.get(&ty).is_some() {
                        // 输入 类型 不能 相同
                        panic!("impl_base_noncopy: input type same, type = {:?}", ty);
                    }
                    map.insert(ty, vec![pre_id]);
                }
                r
            }

            fn fill_from<T: OutParam + ?Sized>(&mut self, pre_id: NodeId, out_param: &T) -> bool {
                out_param.fill_to(pre_id, self, TypeId::of::<Self>())
            }
        }

        impl OutParam for $typ {
            fn can_fill(&self, set: &mut Option<&mut XHashSet<TypeId>>, ty: TypeId) -> bool {
                let r = ty == TypeId::of::<Self>();
                if r && set.is_some() {
                    match set {
                        None => {}
                        Some(s) => {
                            s.insert(ty);
                        }
                    }
                }
                r
            }

            fn fill_to(&self, this_id: NodeId, to: &mut dyn Assign, ty: TypeId) -> bool {
                let r = ty == TypeId::of::<Self>();
                if r {
                    // 为了让外部调用者 更清楚的知道 必须为 Self     实现 Clone
                    let c = Clone::clone(self);

                    let p = &c as *const Self as usize;

                    to.assign(this_id, p);

                    // 注: Copy 和 Drop 不能 共存
                    // 不能 释放放这个 c，因为 c 是要拿去 填充 输入的
                    std::mem::forget(c);
                }
                r
            }
        }
    };
}

impl_base_noncopy!(String);

impl_base_copy!(
    (),
    bool,
    i8,
    i16,
    i32,
    i64,
    i128,
    u8,
    u16,
    u32,
    u64,
    u128,
    f32,
    f64
);
