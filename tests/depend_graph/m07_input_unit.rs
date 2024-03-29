use pi_async_rt::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_futures::BoxFuture;
use pi_render::depend_graph::{
    graph::DependGraph,
    node::{DependNode, ParamUsage},
};
use render_core::depend_graph::NodeId;
use std::time::Duration;

// 多个 输入为 Unit 的节点
#[test]
fn multi_node() {
    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = DependGraph::default();

    let _ = g.add_node("Node1", Node1);
    let _ = g.add_node("Node2", Node2);
    let _ = g.add_node("Node3", Node3);

    let _ = g.add_depend("Node1", "Node2");
    let _ = g.add_depend("Node1", "Node3");

    let _ = g.add_depend("Node2", "Node3");
    
    g.set_finish("Node3", true).unwrap();
    
    g.dump_graphviz();
    
    let rt = runtime.clone();
    let _ = runtime.spawn( async move {
        let rt = rt.clone();

        g.run(&rt, &()).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
}


struct Node1;
impl DependNode<()> for Node1 {
    type Input = ();
    type Output = ();

    fn run<'a>(
        &'a mut self,
        _context: &'a (),
        _input: &'a Self::Input,
        _usage: &'a ParamUsage,
		_id: NodeId, _from: &'static [NodeId], _to: &'static [NodeId],
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node1 Running");

        Box::pin(async move {
            println!("======== Enter Async Node1 Running");

            Ok(())
        })
    }
}

struct Node2;
impl DependNode<()> for Node2 {
    type Input = ();
    type Output = ();

    fn run<'a>(
        &'a mut self,
        _context: &'a (),
        _input: &'a Self::Input,
        _usage: &'a ParamUsage,
		_id: NodeId, _from: &'static [NodeId], _to: &'static [NodeId],
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node2 Running");

        Box::pin(async move {
            println!("======== Enter Async Node2 Running");

            Ok(())
        })
    }
}

struct Node3;
impl DependNode<()> for Node3 {
    type Input = ();
    type Output = ();

    fn run<'a>(
        &'a mut self,
        _context: &'a (),
        _input: &'a Self::Input,
        _usage: &'a ParamUsage,
		_id: NodeId, _from: &'static [NodeId], _to: &'static [NodeId],
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node3 Running");

        Box::pin(async move {
            println!("======== Enter Async Node3 Running");

            Ok(())
        })
    }
}