use futures::{future::BoxFuture, FutureExt};
use pi_async::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_render::{
    graph::{
        graph::RenderGraph,
        node::{Node, ParamUsage},
        RenderContext, param::InParamCollector,
    },
    rhi::{device::RenderDevice, RenderQueue},
};
use pi_share::Share;
use render_derive::NodeParam;
use std::{any::TypeId, time::Duration};

// 多个 节点的 输入输出测试
#[test]
fn input_collector() {

    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = RenderGraph::default();
    
    let n11 = g.add_node("Node11", Node1(11)).unwrap();
    let n12 = g.add_node("Node12", Node1(12)).unwrap();
    let n13 = g.add_node("Node13", Node1(13)).unwrap();
    let n14 = g.add_node("Node14", Node1(14)).unwrap();
    let n15 = g.add_node("Node15", Node1(15)).unwrap();
    
    let n2 = g.add_node("Node2", Node2).unwrap();
    
    println!("node11 id = {:?}", n11);
    println!("node12 id = {:?}", n12);
    println!("node13 id = {:?}", n13);
    println!("node14 id = {:?}", n14);
    println!("node15 id = {:?}", n15);

    g.add_depend("Node11", "Node2");
    g.add_depend("Node12", "Node2");
    g.add_depend("Node13", "Node2");
    g.add_depend("Node14", "Node2");
    g.add_depend("Node15", "Node2");

    g.set_finish("Node2", true).unwrap();
    
    let rt = runtime.clone();
    let _ = runtime.spawn(runtime.alloc(), async move {
        let (device, queue) = init_render().await;

        g.build(&rt, device, queue).await.unwrap();

        println!("======== run graph");
        g.run(&rt).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(5));
}

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct A(u32);

#[derive(NodeParam, Debug, Default, Clone, Eq, PartialEq)]
pub struct B(u32);

#[derive(NodeParam, Debug, Default, Clone)]
#[field_slots]
pub struct Output1 {
    pub a: A,
    pub b: B,
}

impl Drop for Output1 {
    fn drop(&mut self) {
        println!("======== Output1 drop, A = {:?}", self.a);
    }
}

#[derive(NodeParam, Default, Clone)]
#[field_slots]
pub struct Input2 {
    pub c: InParamCollector<A>,
}

struct Node1(u32);

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

            Ok(Output1 { a: A(self.0), b: B(1) })
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
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node2 Running");

        assert!(usage.is_input_fill(TypeId::of::<A>()));
        
        let c = &input.c.0;
        assert_eq!(c.len(), 5);
        for (id, v) in c {
            println!("id = {:?}, v = {:?}", id, v);
        }

        async move {
            println!("======== Enter Async Node2 Running");

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
