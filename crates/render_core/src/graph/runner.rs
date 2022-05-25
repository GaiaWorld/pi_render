use super::{
    graph::RenderGraph,
    node::{NodeLabel, NodeRunError, NodeId},
    // node_slot::{SlotLabel, SlotType},
    RenderContext,
};
use crate::{
    graph::node::NodeState,
    rhi::{device::RenderDevice, RenderQueue},
};
use futures::{future::BoxFuture, FutureExt};
use log::{error, info};
use pi_async::rt::{AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt};
use pi_async_graph::{async_graph, ExecNode, RunFactory, Runner};
use pi_ecs::prelude::World;
use pi_graph::{DirectedGraph, DirectedGraphNode, NGraph, NGraphBuilder};
use pi_hash::XHashMap;
use pi_share::ShareRefCell;
use std::{borrow::Cow, sync::Arc};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum RenderGraphRunnerError {
    #[error(transparent)]
    NodeRunError(#[from] NodeRunError),
}

// 渲染图 执行器
pub struct RenderGraphRunner<P>
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    // 异步运行时
    rt: AsyncRuntime<(), P>,

    // prepare 异步图，build 阶段 用一次
    pub(crate) prepare_graph: Option<Arc<NGraph<NodeId, ExecNode<DumpNode, DumpNode>>>>,

    // run 异步图，一直用，直到 render_graph 改变为止
    pub(crate) run_graph: Option<Arc<NGraph<NodeId, ExecNode<DumpNode, DumpNode>>>>,
}

impl<P> RenderGraphRunner<P>
where
    P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
{
    /// 创建
    pub fn new(rt: AsyncRuntime<(), P>) -> Self {
        Self {
            rt,
            prepare_graph: None,
            run_graph: None,
        }
    }

    /// 构建
    pub fn build<O: Clone>(
        &mut self,
        world: World,
        device: RenderDevice,
        queue: RenderQueue,
        rg: &mut RenderGraph<O>,
    ) -> Result<(), String> {

        if self.prepare_graph.is_some() && self.run_graph.is_some() {
			info!("RenderGraphRunner::build prepare and run graph is ready, prepare is_some = {}, run = {}", self.prepare_graph.is_some(), self.run_graph.is_some());
            return Ok(());
        }

        // 遍历，找 终点
        let finishes = rg.clone_finish_nodes();

        let ng = rg.get_graph_impl().unwrap();

        // 以终为起，构建需要的 节点
        let sub_ng = ng.gen_graph_from_keys(&finishes);

        // // 构造异步图
        // let mut run_builder = NGraphBuilder::<usize>::new();
        // let mut prepare_builder = NGraphBuilder::<usize>::new();

        // fn create_slots_vec(n: &NodeState) -> NodeSlot {
        //     let len = n.output_slots.len();
        //     let mut outputs = Vec::<Option<RealValue>>::with_capacity(len);
        //     for _ in 0..len {
        //         outputs.push(None);
        //     }

        //     (inputs, outputs)
        // }

        // // 从 sub_ng 直接分析 资源依赖
        // let mut map = XHashMap::default();
        // for id in sub_ng.topological_sort() {
        //     let ng_node = sub_ng.get(id).unwrap();
        //     if let NGNodeValue::OutputSlot(nid, sid) = ng_node.value() {
        //         let slots = map.entry(nid).or_insert_with(|| {
        //             let n = rg.get_node(*nid).unwrap();
        //             create_slots_vec(n)
        //         });

        //         slots.1[*sid] = Some(RealValue::default());
        //     }
        // }

        // for id in sub_ng.topological_sort() {
        //     let ng_node = sub_ng.get(id).unwrap();
        //     if let NGNodeValue::InputSlot(nid, sid) = ng_node.value() {
        //         if ng_node.to().len() != 1 {
        //             panic!("================================= InputSlot's len != 1");
        //         }

        //         let mut value = None;
        //         let to = ng_node.to()[0];
        //         let to = sub_ng.get(&to).unwrap();
        //         if let NGNodeValue::OutputSlot(next_node, next_slot) = to.value() {
        //             let v = map.get(next_node).unwrap();
        //             let v = &v.1[*next_slot];
        //             let v = v.as_ref().unwrap().clone();
        //             value = Some(v);
        //         }

        //         let slots = map.entry(nid).or_insert_with(|| {
        //             let n = rg.get_node(*nid).unwrap();
        //             create_slots_vec(n)
        //         });
        //         slots.0[*sid] = value;
        //     }
        // }

        // // 异步图 节点
        // for id in sub_ng.topological_sort() {
        //     let ng_node = sub_ng.get(id).unwrap();
        //     let ng_node_clone = ng_node.value().clone();
        //     match ng_node.value() {
        //         NGNodeValue::Node(n) => {
        //             prepare_builder = prepare_builder.node(
        //                 ng_node_clone.clone(),
        //                 crate_prepare_node(
        //                     &map,
        //                     rg,
        //                     *n,
        //                     device.clone(),
        //                     queue.clone(),
        //                     world.clone(),
        //                 ),
        //             );

        //             run_builder = run_builder.node(
        //                 ng_node_clone,
        //                 crate_run_node(&map, rg, *n, device.clone(), queue.clone(), world.clone()),
        //             );
        //         }
        //         NGNodeValue::InputSlot(_nid, _sid) => {
        //             prepare_builder = prepare_builder.node(ng_node_clone.clone(), ExecNode::None);
        //             run_builder = run_builder.node(ng_node_clone, ExecNode::None);
        //         }
        //         NGNodeValue::OutputSlot(_nid, _sid) => {
        //             prepare_builder = prepare_builder.node(ng_node_clone.clone(), ExecNode::None);
        //             run_builder = run_builder.node(ng_node_clone, ExecNode::None);
        //         }
        //     }
        // }

        // // 异步图 边
        // for id in sub_ng.topological_sort() {
        //     let ng_node = sub_ng.get(id).unwrap();
        //     for _ in ng_node.to() {
        //         let next_node = sub_ng.get(id).unwrap();

        //         prepare_builder =
        //             prepare_builder.edge(ng_node.value().clone(), next_node.value().clone());

        //         run_builder = run_builder.edge(ng_node.value().clone(), next_node.value().clone());
        //     }
        // }

        // match prepare_builder.build() {
        //     Ok(g) => {
        //         self.prepare_graph = Some(Arc::new(g));
        //     }
        //     Err(e) => {
        //         error!(
        //             "render_graph::build prepare_builder graph failed, reason = {:?}",
        //             e
        //         );
        //         panic!("!!!!!!!!!!!!!!!!!!!!!!!!!!! 3");
        //     }
        // };

        // match run_builder.build() {
        //     Ok(g) => {
        //         self.run_graph = Some(Arc::new(g));
        //     }
        //     Err(e) => {
        //         error!(
        //             "render_graph::build run_builder graph failed, reason = {:?}",
        //             e
        //         );
		// 		panic!("!!!!!!!!!!!!!!!!!!!!!!!!!!! 4");
        //     }
        // };

        Ok(())
    }

    /// 每个节点 调用 prepare 方法
    /// 目的：创建资源
    pub async fn prepare(&mut self) {
        match self.prepare_graph {
            None => {
                error!("render_graph::prepare failed, prepare_graph is none");
                panic!("prepare fail");
            }
            Some(ref g) => {
                let ag = async_graph(self.rt.clone(), g.clone());
                ag.await.unwrap();
            }
        }

        // 移除 prepare，因为它只能执行一次
        let t = self.prepare_graph.take();
		t.unwrap();
    }

    /// 执行
    pub async fn run(&mut self) {
        match self.run_graph {
            None => {
                error!("render_graph::run failed, run_graph is none");
                panic!("");
            }
            Some(ref g) => {
                let r = async_graph(self.rt.clone(), g.clone()).await;
                r.unwrap();
            }
        }
    }
}

// 异步图: 哑节点，异步函数不需要的类型
pub struct DumpNode;
impl Runner for DumpNode {
    fn run(self) {}
}
impl RunFactory for DumpNode {
    type R = DumpNode;
    fn create(&self) -> Self::R {
        DumpNode
    }
}

// // 创建异步 节点
// fn crate_run_node(
//     map: &XHashMap<&usize, NodeSlot>,
//     render_graph: &RenderGraph,
//     n: usize,
//     device: RenderDevice,
//     queue: RenderQueue,
//     world: World,
// ) -> ExecNode<DumpNode, DumpNode> {
//     let node = render_graph.get_node(NodeLabel::Id(n)).unwrap();
//     let node = node.node.clone();

//     let (inputs, outputs) = match map.get(&n) {
//         None => (vec![], vec![]),
//         Some((i, o)) => (i.clone(), o.clone()),
//     };
//     let inputs = inputs.as_slice().to_vec();
//     let outputs = outputs.as_slice().to_vec();

//     let f = move || -> BoxFuture<'static, std::io::Result<()>> {
//         let device = device.clone();
//         let queue = queue.clone();
//         let world = world.clone();
//         let node = node.clone();
//         let inputs = inputs.clone();
//         let outputs = outputs.clone();

//         let context = RenderContext {
//             world,
//             device: device.clone(),
//             queue: queue.clone(),
//         };

//         async move {
//             let commands =
//                 device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

//             let commands = ShareRefCell::new(commands);

//             let runner = node.run(
//                 context,
//                 commands.clone(),
//                 inputs.as_slice(),
//                 outputs.as_slice(),
//             );

//             runner.await.unwrap();
           
//             let commands = Arc::try_unwrap(commands.0).unwrap();
//             let commands = commands.into_inner();

//             queue.submit(vec![commands.finish()]);
//             Ok(())
//         }
//         .boxed()
//     };

//     ExecNode::Async(Box::new(f))
// }

// // 创建异步 节点
// fn crate_prepare_node(
//     map: &XHashMap<&usize, NodeSlot>,
//     render_graph: &RenderGraph,
//     n: usize,
//     device: RenderDevice,
//     queue: RenderQueue,
//     world: World,
// ) -> ExecNode<DumpNode, DumpNode> {
//     let node = render_graph.get_node(NodeLabel::Id(n)).unwrap();
//     let node = node.node.clone();

//     let (inputs, outputs) = match map.get(&n) {
//         None => (vec![], vec![]),
//         Some((i, o)) => (i.clone(), o.clone()),
//     };

//     let f = move || -> BoxFuture<'static, std::io::Result<()>> {
//         let device = device.clone();
//         let queue = queue.clone();
//         let world = world.clone();
//         let node = node.clone();
//         let inputs = inputs.clone();
//         let outputs = outputs.clone();

//         async move {
//             let context = RenderContext {
//                 world,
//                 device,
//                 queue,
//             };

//             match node.prepare(context, inputs.as_slice(), outputs.as_slice()) {
//                 None => {}
//                 Some(r) => {
//                     r.await.unwrap();
//                 }
//             }

//             Ok(())
//         }
//         .boxed()
//     };

//     ExecNode::Async(Box::new(f))
// }