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
use render_derive::NodeParam;
use std::{any::TypeId, time::Duration};

// 多个 节点的 输入输出测试
#[test]
fn multi_node() {

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = RenderGraph::default();
    
    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);
    g.add_node("Node3", Node3);
    g.add_node("Node4", Node4);
    g.add_node("Node5", Node5);
    g.add_node("Node6", Node6);
    
    g.add_depend("Node1", "Node4");
    g.add_depend("Node2", "Node4");
    
    g.add_depend("Node2", "Node5");
    g.add_depend("Node3", "Node5");
    
    g.add_depend("Node4", "Node6");
    g.add_depend("Node5", "Node6");

    g.set_finish("Node6", true).unwrap();
    
    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        let rt = rt.clone();
        let (device, queue) = init_render().await;

        g.build(&rt, device, queue).await.unwrap();

        println!("======== 1 run graph");
        g.run(&rt).await.unwrap();

        println!("======== 2 run graph");
        g.run(&rt).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(5));
}

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct A(u32);

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct B(u32);

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct C(String);

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct D(u32);

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct E(u32);

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct F(u32);

#[derive(NodeParam, Debug, Default, Clone)]
#[field_slots]
pub struct Output1 {
    pub a: A,
    pub b: B,
}

impl Drop for Output1 {
    fn drop(&mut self) {
        println!("======== Output1 drop !");
    }
}

#[derive(NodeParam, Default, Clone)]
#[field_slots]
pub struct Output2 {
    pub c: C,
    pub d: D,
}

impl Drop for Output2 {
    fn drop(&mut self) {
        println!("======== Output2 drop !");
    }
}

#[derive(NodeParam, Default, Clone)]
#[field_slots]
pub struct Output3 {
    pub a: A,
    pub e: E,
}

impl Drop for Output3 {
    fn drop(&mut self) {
        println!("======== Output3 drop !");
    }
}

#[derive(NodeParam, Default, Clone)]
#[field_slots]
pub struct Input4 {
    pub a: A,
    pub c: C,
}

#[derive(NodeParam, Default, Clone)]
#[field_slots]
pub struct Input5 {
    pub c: C,
    pub e: E,
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

        assert_eq!(input, &());

        assert!(!usage.is_input_fill(TypeId::of::<()>()));

        assert!(usage.is_output_usage(TypeId::of::<A>()));
        assert!(!usage.is_output_usage(TypeId::of::<B>()));

        async move {
            println!("======== Enter Async Node1 Running");

            Ok(Output1 { a: A(1), b: B(2) })
        }
        .boxed()
    }
}

struct Node2;
impl Node for Node2 {
    type Input = ();
    type Output = Output2;

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node2 Running");

        assert!(!usage.is_input_fill(TypeId::of::<()>()));
        
        assert!(usage.is_output_usage(TypeId::of::<C>()));
        assert!(!usage.is_output_usage(TypeId::of::<D>()));

        async move {
            println!("======== Enter Async Node2 Running");

            Ok(Output2 {
                c: C("3".to_string()),
                d: D(44),
            })
        }
        .boxed()
    }
}

struct Node3;
impl Node for Node3 {
    type Input = ();
    type Output = Output3;

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node3 Running");

        assert!(!usage.is_input_fill(TypeId::of::<()>()));
        
        assert!(!usage.is_output_usage(TypeId::of::<A>()));
        assert!(usage.is_output_usage(TypeId::of::<E>()));

        async move {
            println!("======== Enter Async Node3 Running");

            Ok(Output3 {
                a: A(36),
                e: E(45),
            })
        }
        .boxed()
    }
}

struct Node4;
impl Node for Node4 {
    type Input = Input4;
    type Output = A;

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node4 Running");

        assert!(usage.is_input_fill(TypeId::of::<A>()));
        assert!(usage.is_input_fill(TypeId::of::<C>()));
       
        assert!(usage.is_output_usage(TypeId::of::<A>()));

        assert_eq!(input.a, A(1));
        assert_eq!(input.c, C("3".to_string()));

        async move {
            println!("======== Enter Async Node4 Running");

            Ok(A(44))
        }
        .boxed()
    }
}

struct Node5;
impl Node for Node5 {
    type Input = Input5;
    type Output = F;

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node5 Running");

        assert!(usage.is_input_fill(TypeId::of::<C>()));
        assert!(usage.is_input_fill(TypeId::of::<E>()));
       
        assert!(!usage.is_output_usage(TypeId::of::<F>()));

        assert_eq!(input.c, C("3".to_string()));
        assert_eq!(input.e, E(45));

        async move {
            println!("======== Enter Async Node5 Running");

            Ok(F(55))
        }
        .boxed()
    }
}

struct Node6;
impl Node for Node6 {
    type Input = A;
    type Output = ();

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node6 Running");

        assert!(usage.is_input_fill(TypeId::of::<A>()));
       
        assert!(!usage.is_output_usage(TypeId::of::<()>()));

        assert_eq!(*input, A(44));
        
        async move {
            println!("======== Enter Async Node6 Running");

            Ok(())
        }
        .boxed()
    }
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
