use futures::FutureExt;
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

#[test]
fn simple_graph() {
    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let rt = runtime.clone();

    let mut g = RenderGraph::new(runtime);
    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);
    g.add_node("Node3", Node3);
    g.add_depend("Node1", "Node3");
    g.add_depend("Node2", "Node3");

    g.set_finish("Node3", true).unwrap();

    let _ = rt.spawn(rt.alloc(), async move {
        let (device, queue) = init_render().await;

        g.build(device, queue).unwrap();

        println("======== 1 run graph");
        g.run().await.unwrap();

        std::thread::sleep(Duration::from_secs(1));
        
        println("======== 2 run graph");
        g.run().await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(5));
}

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct A(u32);

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct B(u32);

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct C(u32);

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct D(u32);

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
    pub d: String,
}

impl Drop for Output2 {
    fn drop(&mut self) {
        println!("======== Output2 drop !");
    }
}

#[derive(NodeParam, Default, Clone)]
#[field_slots]
pub struct Input3 {
    pub a: A,
    pub d: D,
    pub s: String,
}

struct Node1;
impl Node for Node1 {
    type Input = ();
    type Output = Output1;

    fn build<'a>(
        &'a self,
        context: RenderContext,
        usage: &'a ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        println!("======== Node1 Build");
    }

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        assert_eq!(input, &());

        async move {
            assert!(usage.is_output_usage(TypeId::of::<A>()));
            assert!(!usage.is_output_usage(TypeId::of::<B>()));

            assert!(!usage.is_input_fill(TypeId::of::<()>()));

            Ok(Output1 { a: A(1), b: B(2) })
        }
        .boxed()
    }
}

struct Node2;
impl Node for Node2 {
    type Input = ();
    type Output = Output2;

    fn build<'a>(
        &'a self,
        context: RenderContext,
        usage: &'a ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        println!("======== Node2 Build");
    }

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        async move {
            assert!(!usage.is_output_usage(TypeId::of::<C>()));
            assert!(usage.is_output_usage(TypeId::of::<String>()));

            assert!(!usage.is_input_fill(TypeId::of::<()>()));

            Ok(Output2 {
                c: C(3),
                d: "4".to_string(),
            })
        }
        .boxed()
    }
}

struct Node3;
impl Node for Node3 {
    type Input = Input3;
    type Output = ();

    fn build<'a>(
        &'a self,
        context: RenderContext,
        usage: &'a ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        println!("======== Node3 Build");
    }

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        async move {
            println!("Begin Node 3 run");

            assert!(usage.is_input_fill(TypeId::of::<A>()));
            assert!(usage.is_input_fill(TypeId::of::<String>()));
            assert!(!usage.is_input_fill(TypeId::of::<D>()));

            assert!(!usage.is_output_usage(TypeId::of::<()>()));

            assert_eq!(input.a, A(1));
            assert_eq!(input.s, "4".to_string());

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
