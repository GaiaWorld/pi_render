// //! 子图 模型
// //!
// //! + 可以 clone，多个副本 指向 同一个子图
// //! + 可以 被高层 修改 拓扑结构（仅限于 Render System 不执行 时）
// //! + 在 构建时，会 拆分成 普通的 图节点（有可能递归 生成，因为 子图 节点 可能还有子图）
// //! + 构建时，子图的所有 输入/输出 节点（入度/出度 为0的节点）的 输入/输出 的并集，就是 这个 父图节点的输入/输出
// //!     - 限制：要求 每个节点的输入结构体的字段都不能同名
// //! + 运行时，一个图，只有 一个 运行时，不会为 子图 产生多个运行时

// use pi_async::rt::AsyncRuntime;
// use pi_futures::BoxFuture;
// use pi_share::{cell::TrustCell, Share};

// use super::{
//     graph::DependGraph,
//     node::{DependNode, ParamUsage},
//     param::{InParam, OutParam},
//     GraphError, NodeId,
// };

// /// 子图
// pub struct SubGraph<A, GI, GO>
// where
//     A: 'static + AsyncRuntime + Send,
//     GI: InParam + OutParam + Default + Clone,
//     GO: InParam + OutParam + Default + Clone,
// {
//     rt: A,
//     graph: Option<DependGraph>,

//     input: InputNode<GI>,
//     output: OutputNode<GO>,

//     input_id: NodeId,
//     output_id: NodeId,
// }

// impl<A, GI, GO> SubGraph<A, GI, GO>
// where
//     A: 'static + AsyncRuntime + Send,
//     GI: InParam + OutParam + Default + Clone,
//     GO: InParam + OutParam + Default + Clone,
// {
//     /// 创建子图
//     pub fn new(rt: A, mut graph: Option<DependGraph>) -> Result<Self, GraphError> {
//         let input = InputNode::default();
//         let output = OutputNode::default();

//         let (input_id, output_id) = if let Some(ref mut g) = graph {
//             Self::inject_input_outuput(g, input.clone(), output.clone())?
//         } else {
//             (NodeId::default(), NodeId::default())
//         };

//         Ok(Self {
//             rt,
//             graph,

//             input,
//             output,

//             input_id,
//             output_id,
//         })
//     }

//     /// 更换 异步运行时
//     pub fn set_async_runtime(&mut self, rt: A) {
//         self.rt = rt;
//     }

//     /// 更换 运行的子图
//     pub fn set_graph(&mut self, mut g: DependGraph) -> Result<(), GraphError> {
//         let (i, o) = Self::inject_input_outuput(&mut g, self.input.clone(), self.output.clone())?;

//         self.input_id = i;
//         self.output_id = o;

//         self.graph = Some(g);

//         Ok(())
//     }

//     /// 取目前的子图，以便对子图 做 拓扑修改
//     pub fn get_graph(&mut self) -> Option<DependGraph> {
//         let input_id = self.input_id;
//         let output_id = self.output_id;
//         if let Some(ref mut g) = self.graph {
//             g.remove_node(input_id).unwrap();
//             g.remove_node(output_id).unwrap();
//         }

//         self.input_id = Default::default();
//         self.output_id = Default::default();

//         self.graph.take()
//     }

//     // 输入输入输出
//     fn inject_input_outuput(
//         g: &mut DependGraph,
//         input: InputNode<GI>,
//         output: OutputNode<GO>,
//     ) -> Result<(NodeId, NodeId), GraphError> {
//         let finish_id = match g.get_once_finsh_id() {
//             Some(id) => id,
//             None => return Err(GraphError::SubGraphOutputError),
//         };

//         let from_id = g.get_input_nodes();
//         let from_id = if from_id.len() != 1 {
//             // 子图 输入节点不得多于一个
//             return Err(GraphError::SubGraphOutputError);
//         } else if let Some(id) = from_id.iter().next() {
//             *id
//         } else {
//             // 子图 输入节点不得为 0
//             return Err(GraphError::SubGraphOutputError);
//         };

//         let input_id = g.add_node("_$pi_m_sub_input$_", input)?;
//         let output_id = g.add_node("_$pi_m_sub_output$_", output)?;

//         // input -> g.入度为0的节点
//         g.add_depend(input_id, from_id).unwrap();

//         // g.finish --> output
//         g.add_depend(finish_id, output_id).unwrap();

//         Ok((input_id, output_id))
//     }
// }

// impl<A, GI, GO> DependNode for SubGraph<A, GI, GO>
// where
//     A: 'static + AsyncRuntime + Send,
//     GI: InParam + OutParam + Default + Clone,
//     GO: InParam + OutParam + Default + Clone,
// {
//     type Input = GI;
//     type Output = GO;

//     fn build<'a>(
//         &'a mut self,
//         _usage: &'a ParamUsage,
//     ) -> Option<BoxFuture<'a, Result<(), String>>> {
//         Some(Box::pin(async move {
//             match self.graph {
//                 Some(ref mut g) => g.build().map_err(|e| e.to_string()),
//                 None => Err("SubGraph build failed, self.graph = None".to_string()),
//             }
//         }))
//     }

//     fn run<'a>(
//         &'a mut self,
//         input: &'a Self::Input,
//         _usage: &'a ParamUsage,
//     ) -> BoxFuture<'a, Result<Self::Output, String>> {
//         Box::pin(async move {
//             // 将 input 扔到 self.input
//             self.input.set_input(input);

//             let rt = self.rt.clone();

//             match self.graph {
//                 Some(ref mut g) => {
//                     match g.run(&rt).await {
//                         Ok(_) => {
//                             // 将 Output的值拿出来用
//                             Ok(self.output.get_output())
//                         }
//                         Err(e) => {
//                             let msg = format!("sub_graph run_ng, {:?}", e);
//                             log::error!("{}", msg);
//                             Err(msg)
//                         }
//                     }
//                 }
//                 None => Err("sub_graph: no sub_graph".to_string()),
//             }
//         })
//     }
// }

// // 输入节点
// #[derive(Clone)]
// struct InputNode<I: InParam + OutParam + Default + Clone>(Share<TrustCell<I>>);

// impl<I> InputNode<I>
// where
//     I: InParam + OutParam + Default + Clone,
// {
//     fn set_input(&self, data: &I) {
//         *self.0.as_ref().borrow_mut() = data.clone();
//     }
// }

// impl<I> Default for InputNode<I>
// where
//     I: InParam + OutParam + Default + Clone,
// {
//     fn default() -> Self {
//         Self(Share::new(TrustCell::new(I::default())))
//     }
// }

// impl<I> DependNode for InputNode<I>
// where
//     I: InParam + OutParam + Default + Clone,
// {
//     type Input = ();
//     type Output = I;

//     fn build<'a>(
//         &'a mut self,
//         _usage: &'a super::node::ParamUsage,
//     ) -> Option<BoxFuture<'a, Result<(), String>>> {
//         None
//     }

//     fn run<'a>(
//         &'a mut self,
//         _input: &'a Self::Input,
//         _usage: &'a super::node::ParamUsage,
//     ) -> BoxFuture<'a, Result<Self::Output, String>> {
//         let input = self.0.as_ref().borrow().clone();
//         Box::pin(async move { Ok(input) })
//     }
// }

// // 输出节点
// #[derive(Clone)]
// struct OutputNode<O: InParam + OutParam + Default + Clone>(Share<TrustCell<O>>);

// impl<O> OutputNode<O>
// where
//     O: InParam + OutParam + Default + Clone,
// {
//     fn get_output(&self) -> O {
//         let mut p = self.0.as_ref().borrow_mut();
//         let r = p.clone();
//         *p = Default::default();
//         r
//     }
// }

// impl<O> Default for OutputNode<O>
// where
//     O: InParam + OutParam + Default + Clone,
// {
//     fn default() -> Self {
//         Self(Share::new(TrustCell::new(O::default())))
//     }
// }

// impl<O> DependNode for OutputNode<O>
// where
//     O: InParam + OutParam + Default + Clone,
// {
//     type Input = O;
//     type Output = ();

//     fn build<'a>(
//         &'a mut self,
//         _usage: &'a super::node::ParamUsage,
//     ) -> Option<BoxFuture<'a, Result<(), String>>> {
//         None
//     }

//     fn run<'a>(
//         &'a mut self,
//         input: &'a Self::Input,
//         _usage: &'a super::node::ParamUsage,
//     ) -> BoxFuture<'a, Result<Self::Output, String>> {
//         // 将 Input 保存起来
//         *self.0.as_ref().borrow_mut() = input.clone();

//         Box::pin(async move { Ok(()) })
//     }
// }
