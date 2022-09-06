use futures::FutureExt;
use pi_async::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_render::{
    graph_new::{graph::RenderGraph, node::Node, RenderContext},
    rhi::{device::RenderDevice, RenderQueue},
};
use pi_share::Share;
use render_derive::NodeParam;
use std::time::Duration;

#[derive(NodeParam, Default, Clone)]
#[field_slots]
pub struct A {
    pub a: f32,
    pub b: u64,
    pub c: String,
}

#[derive(NodeParam, Default, Clone)]
#[field_slots]
pub struct B {
    pub a: f32,
    pub c: String,
}

#[test]
fn simple_graph() {
    
    struct Node1;

    impl Node for Node1 {
        type Input = ();
        type Output = A;

        fn run<'a>(
            &'a self,
            context: RenderContext,
            commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
            input: &Self::Input,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            assert_eq!(input, &());

            async move {
                println!("Node 1 run");

                Ok(A {
                    a: 1.2,
                    b: 235,
                    c: "abcdefg".to_string(),
                })
            }
            .boxed()
        }
    }

    struct Node2;

    impl Node for Node2 {
        type Input = B;
        type Output = ();

        fn run<'a>(
            &'a self,
            context: RenderContext,
            commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
            input: &'a Self::Input,
        ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
            async move {
                println!("Begin Node 2 run");

                assert_eq!(input.a, 1.2);
                assert_eq!(input.c, "abcdefg".to_string());

                Ok(())
            }
            .boxed()
        }
    }
    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let rt = runtime.clone();

    let mut g = RenderGraph::new(runtime);
    g.add_node("Node1", Node1);
    g.add_node("Node2", Node2);
    g.add_depend("Node1", "Node2");
    g.set_finish("Node2", true);

    rt.spawn(rt.alloc(), async move {
        let (device, queue) = init_render().await;

        g.build(device, queue).unwrap();

        g.run().await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(2 * 60 * 60));
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
