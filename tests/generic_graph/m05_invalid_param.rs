use futures::{future::BoxFuture, FutureExt};
use pi_async::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_render::generic_graph::{
    graph::GenericGraph,
    node::{GenericNode, ParamUsage},
};
use pi_share::Share;
use render_derive::NodeParam;
use std::{any::TypeId, time::Duration};

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
    impl GenericNode for Node1 {
        type Input = ();
        type Output = Output1;

        fn run<'a>(
            &'a self,
            input: &Self::Input,
            usage: &'a ParamUsage,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            async move {
                Ok(Output1 {
                    a: 1.0,
                    b: 2,
                    c: 3.0,
                })
            }
            .boxed()
        }
    }

    struct Node2;
    impl GenericNode for Node2 {
        type Input = ();
        type Output = ();

        fn run<'a>(
            &'a self,
            input: &Self::Input,
            usage: &'a ParamUsage,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            async move { Ok(()) }.boxed()
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = GenericGraph::default();
    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);

    g.add_depend("Node1", "Node2");

    g.set_finish("Node2", true).unwrap();

    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        g.build(&rt).await.unwrap();

        println!("======== run graph");
        g.run(&rt).await.unwrap();
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
    impl GenericNode for Node1 {
        type Input = ();
        type Output = ();

        fn run<'a>(
            &'a self,
            input: &Self::Input,
            usage: &'a ParamUsage,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            async move { Ok(()) }.boxed()
        }
    }

    struct Node2;
    impl GenericNode for Node2 {
        type Input = Input1;
        type Output = ();

        fn run<'a>(
            &'a self,
            input: &Self::Input,
            usage: &'a ParamUsage,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            async move { Ok(()) }.boxed()
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = GenericGraph::default();
    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);

    g.add_depend("Node1", "Node2");

    g.set_finish("Node2", true).unwrap();

    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        g.build(&rt).await.unwrap();

        println!("======== run graph");
        g.run(&rt).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
}

// 同一个输入 的 多个前置节点 匹配到 相同类型的输出
// 预期：崩溃
#[test]
fn multi_output_type() {
    struct Node1;
    impl GenericNode for Node1 {
        type Input = ();
        type Output = u32;

        fn run<'a>(
            &'a self,
            input: &Self::Input,
            usage: &'a ParamUsage,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            async move { Ok(1) }.boxed()
        }
    }

    struct Node2;
    impl GenericNode for Node2 {
        type Input = u32;
        type Output = ();

        fn run<'a>(
            &'a self,
            input: &Self::Input,
            usage: &'a ParamUsage,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            async move { Ok(()) }.boxed()
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = GenericGraph::default();
    g.add_node("Node11", Node1);
    g.add_node("Node12", Node1);

    g.add_node("Node2", Node2);

    g.add_depend("Node11", "Node2");
    g.add_depend("Node12", "Node2");

    g.set_finish("Node2", true).unwrap();

    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        g.build(&rt).await.unwrap();

        println!("======== run graph");
        g.run(&rt).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
}
