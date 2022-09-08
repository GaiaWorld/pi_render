use futures::{future::BoxFuture, FutureExt};
use pi_async::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_render::{
    graph::{
        graph::RenderGraph,
        node::{Node, ParamUsage},
        RenderContext,
    },
    rhi::{device::RenderDevice, RenderQueue},
};
use pi_share::Share;
use std::{any::TypeId, time::Duration};

// 两个 节点
// 输入 输出 基本类型
#[test]
fn two_node_with_simple_param() {
    struct Node1;
    impl Node for Node1 {
        type Input = ();
        type Output = f32;

        fn run<'a>(
            &'a self,
            context: RenderContext,
            commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
            input: &Self::Input,
            usage: &'a ParamUsage,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node1 Running");

            // 输入 自然就是 空元组
            assert_eq!(input, &());

            // 输入：空元组 没前缀节点填，为 false
            assert!(!usage.is_input_fill(TypeId::of::<()>()));

            // 输出：f32 有 后继节点使用，为 true
            assert!(usage.is_output_usage(TypeId::of::<f32>()));

            // 输出：u64 根本 不属于 该输出，自然不会有 后继节点使用，为 false
            assert!(!usage.is_output_usage(TypeId::of::<u64>()));

            async move {
                println!("======== Enter Async Node1 Running");
                Ok(30.25)
            }
            .boxed()
        }
    }

    struct Node2;
    impl Node for Node2 {
        type Input = f32;
        type Output = ();

        fn run<'a>(
            &'a self,
            context: RenderContext,
            commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
            input: &Self::Input,
            usage: &'a ParamUsage,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node2 Running");

            // 输入就是 Node1 的 输出
            assert_eq!(*input, 30.25);

            // f32 被 前置节点 Node1 填充，所以 这里 返回 true
            assert!(usage.is_input_fill(TypeId::of::<f32>()));

            async move {
                println!("======== Enter Async Node2 Running");
                Ok(())
            }
            .boxed()
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = RenderGraph::default();

    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);

    g.add_depend("Node1", "Node2");

    g.set_finish("Node2", true).unwrap();

    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        let (device, queue) = init_render().await;

        g.build(&rt, device, queue).await.unwrap();

        println!("======== 1 run graph");
        g.run(&rt).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(5));
}

// 两个 节点
// 输入 输出 基本类型
// 不匹配 的 参数
#[test]
fn two_node_with_no_match() {
    struct Node1;
    impl Node for Node1 {
        type Input = ();
        type Output = f32;

        fn run<'a>(
            &'a self,
            context: RenderContext,
            commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
            input: &Self::Input,
            usage: &'a ParamUsage,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node1 Running");

            // 输入 自然就是 空元组
            assert_eq!(input, &());

            // 输入：空元组 没前缀节点填，为 false
            assert!(!usage.is_input_fill(TypeId::of::<()>()));

            // 输出：f32 没有 后继节点使用，为 false
            assert!(!usage.is_output_usage(TypeId::of::<f32>()));

            async move {
                println!("======== Enter Async Node1 Running");
                Ok(30.25)
            }
            .boxed()
        }
    }

    struct Node2;
    impl Node for Node2 {
        type Input = u64;
        type Output = ();

        fn run<'a>(
            &'a self,
            context: RenderContext,
            commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
            input: &Self::Input,
            usage: &'a ParamUsage,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            println!("======== Enter Node2 Running");

            // 输入就是 Node1 的 输出
            assert_eq!(*input, 0);

            // u64 没有被 前置节点 Node1 填充，返回 false
            assert!(!usage.is_input_fill(TypeId::of::<u64>()));

            async move {
                println!("======== Enter Async Node2 Running");
                Ok(())
            }
            .boxed()
        }
    }

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = RenderGraph::default();

    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);

    g.add_depend("Node1", "Node2");

    g.set_finish("Node2", true).unwrap();
    
    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        let (device, queue) = init_render().await;
        g.build(&rt, device, queue).await.unwrap();

        println!("======== 1 run graph");
        g.run(&rt).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(5));
}

async fn init_render() -> (RenderDevice, RenderQueue) {
    let backends = wgpu::Backends::all();
    let instance = wgpu::Instance::new(backends);

    let adapter_options = wgpu::RequestAdapterOptions {
        ..Default::default()
    };
    let adapter = instance
        .request_adapter(&adapter_options)
        .await
        .expect("Unable to find a GPU! Make sure you have installed required drivers!");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                ..Default::default()
            },
            None,
        )
        .await
        .unwrap();

    let device = Share::new(device);
    let queue = Share::new(queue);

    (RenderDevice::from(device), queue)
}
