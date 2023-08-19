use pi_async_rt::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_futures::BoxFuture;
use pi_render::depend_graph::{
    graph::DependGraph,
    node::{DependNode, ParamUsage},
};
use render_core::depend_graph::NodeId;
use render_derive::NodeParam;
use std::time::Duration;

// 输出 同类型
// 预期：会崩溃
#[test]
fn out_same_type() {
    #[derive(NodeParam, Default, Clone)]
    #[field_slots]
    pub struct Output1 {
        pub a: f32,
        pub b: u32,
        pub c: f32,
    }

    struct Node1;
    impl DependNode<()> for Node1 {
        type Input = ();
        type Output = Output1;

        fn run<'a>(
            &'a mut self,
            _context: &'a (),
            _input: &'a Self::Input,
            _usage: &'a ParamUsage,
			_id: NodeId, _from: &'static [NodeId], _to: &'static [NodeId],
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            Box::pin(async move {
                Ok(Output1 {
                    a: 1.0,
                    b: 2,
                    c: 3.0,
                })
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
            Box::pin(async move { Ok(()) })
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = DependGraph::default();
    let _ = g.add_node("Node1", Node1);
    let _ = g.add_node("Node2", Node2);

    let _ = g.add_depend("Node1", "Node2");

    g.set_finish("Node2", true).unwrap();

    let rt = runtime.clone();
    let _ = runtime.spawn(async move {

        println!("======== run graph");
        g.run(&rt, &()).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
}

// 输入 同类型
// 预期：崩溃
#[test]
fn in_same_type() {
    #[derive(NodeParam, Default, Clone)]
    #[field_slots]
    pub struct Input1 {
        pub a: f32,
        pub b: u32,
        pub c: f32,
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
            Box::pin(async move { Ok(()) })
        }
    }

    struct Node2;
    impl DependNode<()> for Node2 {
        type Input = Input1;
        type Output = ();

        fn run<'a>(
            &'a mut self,
            _context: &'a (),
            _input: &'a Self::Input,
            _usage: &'a ParamUsage,
			_id: NodeId, _from: &'static [NodeId], _to: &'static [NodeId],
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            Box::pin(async move { Ok(()) })
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = DependGraph::default();
    let _ = g.add_node("Node1", Node1);
    let _ = g.add_node("Node2", Node2);

    let _ = g.add_depend("Node1", "Node2");

    g.set_finish("Node2", true).unwrap();

    let rt = runtime.clone();
    let _ = runtime.spawn(async move {

        println!("======== run graph");
        g.run(&rt, &()).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
}

// 同一个输入 的 多个前置节点 匹配到 相同类型的输出
// 预期：崩溃
#[test]
fn multi_output_type() {
    struct Node1;
    impl DependNode<()> for Node1 {
        type Input = ();
        type Output = u32;

        fn run<'a>(
            &'a mut self,
            _context: &'a (),
            _input: &'a Self::Input,
            _usage: &'a ParamUsage,
			_id: NodeId, _from: &'static [NodeId], _to: &'static [NodeId],
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            Box::pin(async move { Ok(1) })
        }
    }

    struct Node2;
    impl DependNode<()> for Node2 {
        type Input = u32;
        type Output = ();

        fn run<'a>(
            &'a mut self,
            _context: &'a (),
            _input: &'a Self::Input,
            _usage: &'a ParamUsage,
			_id: NodeId, _from: &'static [NodeId], _to: &'static [NodeId],
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            Box::pin(async move { Ok(()) })
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = DependGraph::default();
    let _ = g.add_node("Node11", Node1);
    let _ = g.add_node("Node12", Node1);

    let _ = g.add_node("Node2", Node2);

    let _ = g.add_depend("Node11", "Node2");
    let _ = g.add_depend("Node12", "Node2");

    g.set_finish("Node2", true).unwrap();

    let rt = runtime.clone();
    let _ = runtime.spawn(async move {

        println!("======== run graph");
        g.run(&rt, &()).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
}
