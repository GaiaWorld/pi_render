use pi_async::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_futures::BoxFuture;
use pi_render::depend_graph::{
    graph::DependGraph,
    node::{DependNode, ParamUsage},
};
use pi_share::Share;
use std::{any::TypeId, time::Duration};

// 两个 节点
// 输入 输出 基本类型
#[test]
fn two_node_with_simple_param() {
    struct Node1;
    impl DependNode<()> for Node1 {
        type Input = ();
        type Output = f32;

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

            // 输出：f32 有 后继节点使用，为 true
            assert!(usage.is_output_usage(TypeId::of::<f32>()));

            // 输出：u64 根本 不属于 该输出，自然不会有 后继节点使用，为 false
            assert!(!usage.is_output_usage(TypeId::of::<u64>()));

            Box::pin(async move {
                println!("======== Enter Async Node1 Running");
                Ok(30.25)
            })
        }
    }

    struct Node2;
    impl DependNode<()> for Node2 {
        type Input = f32;
        type Output = ();

        fn run<'a>(
            &'a mut self,
            context: &'a (),
            input: &'a Self::Input,
            usage: &'a ParamUsage,
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node2 Running");

            // 输入就是 Node1 的 输出
            assert_eq!(*input, 30.25);

            // f32 被 前置节点 Node1 填充，所以 这里 返回 true
            assert!(usage.is_input_fill(TypeId::of::<f32>()));

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

    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        g.build().unwrap();

        println!("======== 1 run graph");
        g.run(&rt, &()).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
}

// 两个 节点
// 输入 输出 基本类型
// 不匹配 的 参数
#[test]
fn two_node_with_no_match() {
    struct Node1;
    impl DependNode<()> for Node1 {
        type Input = ();
        type Output = f32;

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

            // 输出：f32 没有 后继节点使用，为 false
            assert!(!usage.is_output_usage(TypeId::of::<f32>()));

            Box::pin(async move {
                println!("======== Enter Async Node1 Running");
                Ok(30.25)
            })
        }
    }

    struct Node2;
    impl DependNode<()> for Node2 {
        type Input = u64;
        type Output = ();

        fn run<'a>(
            &'a mut self,
            context: &'a (),
            input: &'a Self::Input,
            usage: &'a ParamUsage,
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node2 Running");

            // 输入就是 Node1 的 输出
            assert_eq!(*input, 0);

            // u64 没有被 前置节点 Node1 填充，返回 false
            assert!(!usage.is_input_fill(TypeId::of::<u64>()));

            Box::pin(async move {
                println!("======== Enter Async Node2 Running");
                Ok(())
            })
        }
    }

    struct Node3;
    impl DependNode<()> for Node3 {
        type Input = u64;
        type Output = ();

        fn run<'a>(
            &'a mut self,
            context: &'a (),
            input: &'a Self::Input,
            usage: &'a ParamUsage,
        ) -> BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node3 Running");

            // 输入就是 Node1 的 输出
            assert_eq!(*input, 0);

            // u64 没有被 前置节点 Node1 填充，返回 false
            assert!(!usage.is_input_fill(TypeId::of::<u64>()));

            Box::pin(async move {
                println!("======== Enter Async Node3 Running");
                Ok(())
            })
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = DependGraph::default();

    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);
    g.add_node("Node3", Node3);

    g.add_depend("Node1", "Node2");
    g.add_depend("Node2", "Node3");

    g.set_finish("Node3", true).unwrap();

    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        g.build().unwrap();

        println!("======== 1 run graph");
        g.run(&rt, &()).await.unwrap();

        g.remove_node("Node3");
        g.set_finish("Node2", true).unwrap();
        g.build().unwrap();

        println!("\n\n======== 2 run graph");
        g.run(&rt, &()).await.unwrap();

    });

    std::thread::sleep(Duration::from_secs(3));
}
