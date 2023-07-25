use pi_async_rt::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_futures::BoxFuture;
use pi_render::depend_graph::{
    graph::DependGraph,
    node::{DependNode, ParamUsage},
};
use pi_share::Share;
use render_derive::NodeParam;
use std::{any::TypeId, time::Duration};

// 两个 节点
// 输入 输出 结构体，不展开
#[test]
fn two_node_with_noslot() {
    #[derive(NodeParam, Clone, Debug, Default, PartialEq, Eq)]
    pub struct A(u32);

    struct Node1;
    impl DependNode<()> for Node1 {
        type Input = ();
        type Output = A;

        fn run<'a>(
            &'a mut self,
            context: &'a (),
            input: &'a Self::Input,
            usage: &'a ParamUsage,
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node1 Running");

            // 输入 自然就是 空元组
            assert_eq!(input, &());

            // 输入：空元组 没前缀节点填，为 false
            assert!(!usage.is_input_fill(TypeId::of::<()>()));

            // 输出：A 有 后继节点使用，为 true
            assert!(usage.is_output_usage(TypeId::of::<A>()));

            Box::pin(async move {
                println!("======== Enter Async Node1 Running");
                Ok(A(12))
            })
        }
    }

    struct Node2;
    impl DependNode<()> for Node2 {
        type Input = A;
        type Output = ();

        fn run<'a>(
            &'a mut self,
            context: &'a (),
            input: &'a Self::Input,
            usage: &'a ParamUsage,
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node2 Running");

            // 输入就是 Node1 的 输出
            assert_eq!(*input, A(12));

            // A 被 前置节点 Node1 填充，返回 true
            assert!(usage.is_input_fill(TypeId::of::<A>()));

            Box::pin(async move {
                println!("======== Enter Async Node2 Running");
                Ok(())
            })
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = DependGraph::<()>::default();

    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);

    g.add_depend("Node1", "Node2");

    g.set_finish("Node2", true).unwrap();
    g.dump_graphviz();

    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        g.build().unwrap();

        println!("======== 1 run graph");
        g.run(&rt, &()).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
}

// 两个 节点
// 输入 输出 结构体，展开 字段
#[test]
fn two_node_with_slot() {
    #[derive(NodeParam, Clone, Debug, Default, PartialEq, Eq)]
    pub struct A(u32);

    #[derive(NodeParam, Clone, Debug, Default)]
    #[field_slots]
    pub struct Output1 {
        pub a: A,
        pub b: u32,
        pub c: String,

        d: f32,
    }

    #[derive(NodeParam, Clone, Debug, Default)]
    #[field_slots]
    pub struct Input2 {
        pub i1: u32,
        pub i2: String,
        pub i3: f32,
    }

    struct Node1;
    impl DependNode<()> for Node1 {
        type Input = ();
        type Output = Output1;

        fn run<'a>(
            &'a mut self,
            context: &'a (),
            input: &'a Self::Input,
            usage: &'a ParamUsage,
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node1 Running");

            // 输入 自然就是 空元组
            assert_eq!(input, &());

            // 输入：空元组 没前缀节点填，为 false
            assert!(!usage.is_input_fill(TypeId::of::<()>()));

            assert!(usage.is_output_usage(TypeId::of::<u32>()));
            assert!(usage.is_output_usage(TypeId::of::<String>()));
            assert!(!usage.is_output_usage(TypeId::of::<A>()));

            Box::pin(async move {
                println!("======== Enter Async Node1 Running");
                Ok(Output1 {
                    a: A(34),
                    b: 67,
                    c: "abcdefg".to_string(),
                    d: 89.34,
                })
            })
        }
    }

    struct Node2;
    impl DependNode<()> for Node2 {
        type Input = Input2;
        type Output = ();

        fn run<'a>(
            &'a mut self,
            context: &'a (),
            input: &'a Self::Input,
            usage: &'a ParamUsage,
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node2 Running");

            // 输入就是 Node1 的 输出
            assert_eq!(input.i1, 67);
            assert_eq!(input.i2, "abcdefg".to_string());
            assert_eq!(input.i3, 0.0);

            assert!(usage.is_input_fill(TypeId::of::<u32>()));
            assert!(usage.is_input_fill(TypeId::of::<String>()));
            assert!(!usage.is_input_fill(TypeId::of::<f32>()));

            Box::pin(async move {
                println!("======== Enter Async Node2 Running");
                Ok(())
            })
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = DependGraph::default();

    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);

    g.add_depend("Node1", "Node2");

    g.set_finish("Node2", true).unwrap();
    g.dump_graphviz();

    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        g.build().unwrap();

        println!("======== 1 run graph");
        g.run(&rt, &()).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
}
