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
use pi_share::{cell::TrustCell, Share};
use render_derive::NodeParam;
use std::{any::TypeId, sync::Arc, time::Duration};

// 构建 & 修改 渲染图 测试
#[test]
fn change_graph() {
    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let rt = runtime.clone();

    let mut g = RenderGraph::new(runtime);

    let n1 = Node1::default();
    let n2 = Node2::default();
    let n3 = Node3::default();
    let n4 = Node4::default();

    g.add_node("Node1", n1.clone());
    g.add_node("Node2", n2.clone());
    g.add_node("Node3", n3.clone());
    g.add_node("Node4", n4.clone());

    g.add_depend("Node1", "Node3");
    g.add_depend("Node2", "Node3");

    g.set_finish("Node3", true).unwrap();

    let _ = rt.spawn(rt.alloc(), async move {
        let (device, queue) = init_render().await;

        println!("======================== should build call ");

        g.build(device.clone(), queue.clone()).await.unwrap();
        g.run().await.unwrap();

        println!("======================== shouldn't build call ");
        g.build(device.clone(), queue.clone()).await.unwrap();
        g.run().await.unwrap();

        *n1.0.as_ref().borrow_mut() = false;
        *n2.0.as_ref().borrow_mut() = false;
        *n3.0.as_ref().borrow_mut() = false;
        *n4.0.as_ref().borrow_mut() = false;

        g.remove_depend("Node1", "Node3");
        g.remove_depend("Node2", "Node3");

        g.add_depend("Node1", "Node4");
        g.add_depend("Node2", "Node4");

        g.set_finish("Node3", false).unwrap();
        g.set_finish("Node4", true).unwrap();

        println!("======================== should build call ");
        g.build(device.clone(), queue.clone()).await.unwrap();
        g.run().await.unwrap();

        println!("======================== shouldn't build call ");
        g.build(device, queue).await.unwrap();
        g.run().await.unwrap();
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

#[derive(NodeParam, Debug, Default, Clone)]
#[field_slots]
pub struct Output1 {
    pub a: A,
    pub b: B,
}

impl Drop for Output1 {
    fn drop(&mut self) {
        println!("======== Output1 drop = {:?}", self);
    }
}

#[derive(NodeParam, Default, Clone, Debug)]
#[field_slots]
pub struct Output2 {
    pub c: C,
    pub d: D,
}

impl Drop for Output2 {
    fn drop(&mut self) {
        println!("======== Output2 drop = {:?}", self);
    }
}

#[derive(NodeParam, Default, Clone)]
#[field_slots]
pub struct Input3 {
    pub a: A,
    pub d: D,
}

#[derive(Clone, Debug)]
struct Node1(Arc<TrustCell<bool>>);

impl Default for Node1 {
    fn default() -> Self {
        Self(Arc::new(TrustCell::new(true)))
    }
}

impl Node for Node1 {
    type Input = ();
    type Output = Output1;

    fn build<'a>(
        &'a self,
        context: RenderContext,
        usage: &'a ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        println!("++++++++++++++++++++++++ Node1 Build");
        None
    }

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        let is_first_build = *self.0.as_ref().borrow();
        println!(
            "======== Enter Node1 Running, is_first_build = {}",
            is_first_build
        );

        if is_first_build {
            assert!(usage.is_output_usage(TypeId::of::<A>()));
            assert!(!usage.is_output_usage(TypeId::of::<B>()));
        } else {
            assert!(!usage.is_output_usage(TypeId::of::<A>()));
            assert!(usage.is_output_usage(TypeId::of::<B>()));
        }

        async move {
            println!("======== Enter Async Node1 Running");

            Ok(Output1 { a: A(1), b: B(2) })
        }
        .boxed()
    }
}

#[derive(Clone, Debug)]
struct Node2(Arc<TrustCell<bool>>);

impl Default for Node2 {
    fn default() -> Self {
        Self(Arc::new(TrustCell::new(true)))
    }
}

impl Node for Node2 {
    type Input = ();
    type Output = Output2;

    fn build<'a>(
        &'a self,
        context: RenderContext,
        usage: &'a ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        println!("++++++++++++++++++++++++ Node2 Build");
        None
    }

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        let is_first_build = *self.0.as_ref().borrow();
        println!(
            "======== Enter Node2 Running, is_first_build = {}",
            is_first_build
        );

        if is_first_build {
            assert!(!usage.is_output_usage(TypeId::of::<C>()));
            assert!(usage.is_output_usage(TypeId::of::<D>()));
        } else {
            assert!(!usage.is_output_usage(TypeId::of::<C>()));
            assert!(!usage.is_output_usage(TypeId::of::<D>()));
        }

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

#[derive(Clone, Debug)]
struct Node3(Arc<TrustCell<bool>>);

impl Default for Node3 {
    fn default() -> Self {
        Self(Arc::new(TrustCell::new(true)))
    }
}

impl Node for Node3 {
    type Input = Input3;
    type Output = ();

    fn build<'a>(
        &'a self,
        context: RenderContext,
        usage: &'a ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        println!("++++++++++++++++++++++++ Node3 Build");
        None
    }

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node3 Running");
        let is_first_build = *self.0.as_ref().borrow();
        assert!(is_first_build);

        assert!(usage.is_input_fill(TypeId::of::<A>()));
        assert!(usage.is_input_fill(TypeId::of::<D>()));

        async move {
            println!("======== Enter Async Node3 Running");

            Ok(())
        }
        .boxed()
    }
}

#[derive(Clone, Debug)]
struct Node4(Arc<TrustCell<bool>>);

impl Default for Node4 {
    fn default() -> Self {
        Self(Arc::new(TrustCell::new(true)))
    }
}

impl Node for Node4 {
    type Input = B;
    type Output = ();

    fn build<'a>(
        &'a self,
        context: RenderContext,
        usage: &'a ParamUsage,
    ) -> Option<BoxFuture<'a, Result<(), String>>> {
        println!("++++++++++++++++++++++++ Node4 Build");
        None
    }

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: pi_share::ShareRefCell<wgpu::CommandEncoder>,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> futures::future::BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node4 Running");

        let is_first_build = *self.0.as_ref().borrow();
        assert!(!is_first_build);

        assert!(usage.is_input_fill(TypeId::of::<B>()));

        assert_eq!(*input, B(2));

        async move {
            println!("======== Enter Async Node4 Running");

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
