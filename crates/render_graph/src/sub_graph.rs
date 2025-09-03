// //! 依赖图 节点
// //!
// //! 主要 数据结构：
// //!
// //!     + trait DependNode 图节点的 对外接口
// //!         - 关联类型：Input, Output
// //!     + NodeId     节点的id
// //!     + NodeLabel  节点标示，可以用 Id 或 String
// //!     + ParamUsage 参数的用途
// //!
// use super::{
//     param::{Assign, InParam, OutParam},
//     GraphError,
// };
// use pi_futures::BoxFuture;
// use pi_hash::{XHashMap, XHashSet};
// use pi_share::{Cell, Share, ThreadSync};
// use pi_slotmap::new_key_type;
// use std::{
//     any::TypeId,
//     borrow::Cow,
//     ops::{Deref, DerefMut},
//     sync::atomic::{AtomicI32, Ordering},
// };


// /// 链接 NodeInteral 和 DependNode 的 结构体
// pub(crate) struct SubGraphNodeImpl<I, O, Context>
// where
//     Context: ThreadSync + 'static,
//     I: InParam + Default,
//     O: OutParam + Default,
// {
//     graph: R,
//     input: I,
//     output: O,

//     context: std::marker::PhantomData<Context>,

//     param_usage: ParamUsage,
//     pre_nodes: Vec<(NodeId, NodeState<Context>)>,

//     // 该节点 的后继 节点数量
//     // 当依赖图改变节点的拓扑关系后，需要调用一次
//     total_next_refs: i32,

//     // 该节点 当前 后继节点数量
//     // 每帧 运行 依赖图 前，让它等于  next_refs
//     curr_next_refs: AtomicI32,
// }

// impl<I, O, R, Context> DependNodeImpl<I, O, R, Context>
// where
//     Context: ThreadSync + 'static,
//     I: InParam + Default,
//     O: OutParam + Default,
//     R: DependNode<Context, Input = I, Output = O>,
// {
//     pub(crate) fn new(node: R) -> Self {
//         Self {
//             context: Default::default(),
//             node,
//             pre_nodes: Default::default(),
//             input: Default::default(),
//             output: Default::default(),

//             param_usage: Default::default(),

//             total_next_refs: 0,
//             curr_next_refs: AtomicI32::new(0),
//         }
//     }
// }

// impl<I, O, R, Context> OutParam for DependNodeImpl<I, O, R, Context>
// where
//     Context: ThreadSync + 'static,
//     I: InParam + Default,
//     O: OutParam + Default,
//     R: DependNode<Context, Input = I, Output = O>,
// {
//     fn can_fill(&self, set: &mut Option<&mut XHashSet<TypeId>>, ty: TypeId) -> bool {
//         assert!(set.is_none());

//         let mut p = self.param_usage.output_usage_set.as_ref().borrow_mut();
//         self.output.can_fill(&mut Some(p.deref_mut()), ty)
//     }

//     fn fill_to(&self, this_id: NodeId, to: &mut dyn Assign, ty: TypeId) -> bool {
//         self.output.fill_to(this_id, to, ty)
//     }
// }

// impl<I, O, R, Context> InternalNode<Context> for DependNodeImpl<I, O, R, Context>
// where
//     Context: ThreadSync + 'static,
//     I: InParam + Default,
//     O: OutParam + Default,
//     R: DependNode<Context, Input = I, Output = O>,
// {
//     fn reset(&mut self) {
//         self.input = Default::default();
//         self.output = Default::default();

//         self.param_usage.reset();
//         self.pre_nodes.clear();

//         self.total_next_refs = 0;
//         self.curr_next_refs = AtomicI32::new(0);
//     }

//     fn inc_next_refs(&mut self) {
//         self.total_next_refs += 1;
//     }

//     fn add_pre_node(&mut self, node: (NodeId, NodeState<Context>)) {
//         node.1 .0.as_ref().borrow_mut().inc_next_refs();

//         {
//             let n = node.1 .0.as_ref().borrow();
//             // 填写 该节点输入 和 前置节点输出 的信息
//             self.input
//                 .can_fill(&mut self.param_usage.input_map_fill, node.0, n.deref());
//         }
		
//         self.pre_nodes.push(node);
//     }

//     fn dec_curr_ref(&self) {
//         // 注：这里 last_count 是 self.curr_next_refs 减1 前 的结果
//         let last_count = self.curr_next_refs.fetch_sub(1, Ordering::SeqCst);
//         assert!(
//             last_count >= 1,
//             "DependNode error, last_count = {last_count}"
//         );

//         if last_count == 1 {
//             // SAFE: 此处强转可变，然后清理self.output是安全的
//             // curr_next_refs 为 原子操作，在一次图运行过程中， 保证了此处代码仅运行一次
//             unsafe { &mut *(self as *const Self as usize as *mut Self) }.output =
//                 Default::default();
//         }
//     }

// 	fn build<'a>(&'a mut self, context: &'a Context, id: NodeId, from: &'a [NodeId], to: &'a [NodeId]) -> Result<(), GraphError> {
//         for (pre_id, pre_node) in &self.pre_nodes {
// 			let p = pre_node.0.as_ref();
// 			let p = p.borrow();
// 			self.input.fill_from(*pre_id, p.deref());
// 			// // 用完了 一个前置，引用计数 减 1
// 			// build阶段不减1，在run中减一
// 			// p.deref().dec_curr_ref();
// 		}

// 		let runner = self.node.build(context, &self.input, &self.param_usage, id, from, to);

// 		match runner {
// 			Ok(output) => {
// 				// 结束前，先 重置 引用数
// 				self.curr_next_refs
// 					.store(self.total_next_refs, Ordering::SeqCst);

// 				// 运行完，重置 输入
// 				self.input = Default::default();

// 				// 替换 输出
// 				if self.total_next_refs == 0 {
// 					// 注：如果此节点 无 后继节点，则 该节点输出 会 直接为 Default
// 					self.output = Default::default();
// 				} else {
// 					self.output = output;
// 				}

// 				Ok(())
// 			}
// 			Err(msg) => Err(GraphError::CustomRunError(msg)),
// 		}
//     }

//     // fn build<'a>(&'a mut self, context: &'a Context) -> Result<(), GraphError> {
//     //     self.node
//     //         .build(context, &self.param_usage)
//     //         .map_err(GraphError::CustomBuildError)
//     // }

//     fn run<'a>(&'a mut self, index: usize, context: &'a Context, id: NodeId, from: &'static [NodeId], to: &'static [NodeId]) -> BoxFuture<'a, Result<(), GraphError>> {
//         Box::pin(async move {
//             for (_pre_id, pre_node) in &self.pre_nodes {
//                 let p = pre_node.0.as_ref();
//                 let p = p.borrow();
//                 // self.input.fill_from(*pre_id, p.deref());
//                 // 用完了 一个前置，引用计数 减 1
//                 p.deref().dec_curr_ref();
//             }

//             let runner = self.node.run(index, context, &self.input, &self.param_usage, id, from, to);

//             match runner.await {
//                 Ok(_output) => {
//                     // // 结束前，先 重置 引用数
//                     // self.curr_next_refs
//                     //     .store(self.total_next_refs, Ordering::SeqCst);

//                     // // 运行完，重置 输入
//                     // self.input = Default::default();

//                     // // 替换 输出
//                     // if self.total_next_refs == 0 {
//                     //     // 注：如果此节点 无 后继节点，则 该节点输出 会 直接为 Default
//                     //     self.output = Default::default();
//                     // } else {
//                     //     self.output = output;
//                     // }

//                     Ok(())
//                 }
//                 Err(msg) => Err(GraphError::CustomRunError(msg)),
//             }
//         })
//     }
// }

// // 节点 状态
// pub(crate) struct NodeState<Context: ThreadSync + 'static>(
//     pub Share<Cell<dyn InternalNode<Context>>>,
// );

// impl<Context: ThreadSync + 'static> Clone for NodeState<Context> {
//     fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }


// impl<Context: ThreadSync + 'static> NodeState<Context> {
//     pub(crate) fn new<I, O, R>(node: R) -> Self
//     where
//         I: InParam + Default,
//         O: OutParam + Default + Clone,
//         R: DependNode<Context, Input = I, Output = O>,
//     {
//         let imp = DependNodeImpl::new(node);

//         let imp = Share::new(Cell::new(imp));

//         Self(imp)
//     }
// }
