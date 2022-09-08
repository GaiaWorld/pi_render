use futures::{future::BoxFuture, FutureExt};
use pi_async::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_ecs::world::{World, WorldId};
use pi_render::{
    graph::{
        graph::RenderGraph,
        node::{Node, ParamUsage},
        RenderContext,
    },
    rhi::{device::RenderDevice, RenderQueue},
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
    impl Node for Node1 {
        type Input = ();
        type Output = A;

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

            // 输出：A 有 后继节点使用，为 true
            assert!(usage.is_output_usage(TypeId::of::<A>()));

            async move {
                println!("======== Enter Async Node1 Running");
                Ok(A(12))
            }
            .boxed()
        }
    }

    struct Node2;
    impl Node for Node2 {
        type Input = A;
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
            assert_eq!(*input, A(12));

            // A 被 前置节点 Node1 填充，返回 true
            assert!(usage.is_input_fill(TypeId::of::<A>()));
            
            async move {
                println!("======== Enter Async Node2 Running");
                Ok(())
            }
            .boxed()
        }
    }

    let world = World::new();
    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = RenderGraph::default();

    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);

    g.add_depend("Node1", "Node2");

    g.set_finish("Node2", true).unwrap();
    
    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        let (device, queue) = init_render().await;

        g.build(&rt, device, queue, world).await.unwrap();

        println!("======== 1 run graph");
        g.run(&rt).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(5));
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
    impl Node for Node1 {
        type Input = ();
        type Output = Output1;

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

            assert!(usage.is_output_usage(TypeId::of::<u32>()));
            assert!(usage.is_output_usage(TypeId::of::<String>()));
            assert!(!usage.is_output_usage(TypeId::of::<A>()));

            async move {
                println!("======== Enter Async Node1 Running");
                Ok(Output1 {
                    a: A(34),
                    b: 67,
                    c: "abcdefg".to_string(),
                    d: 89.34,
                })
            }
            .boxed()
        }
    }

    struct Node2;
    impl Node for Node2 {
        type Input = Input2;
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
            assert_eq!(input.i1, 67);
            assert_eq!(input.i2, "abcdefg".to_string());
            assert_eq!(input.i3, 0.0);
            
            assert!(usage.is_input_fill(TypeId::of::<u32>()));
            assert!(usage.is_input_fill(TypeId::of::<String>()));
            assert!(!usage.is_input_fill(TypeId::of::<f32>()));
            
            async move {
                println!("======== Enter Async Node2 Running");
                Ok(())
            }
            .boxed()
        }
    }

    let world = World::new();
    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = RenderGraph::default();

    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);

    g.add_depend("Node1", "Node2");

    g.set_finish("Node2", true).unwrap();
    
    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        let (device, queue) = init_render().await;

        g.build(&rt, device, queue, world).await.unwrap();

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
