use pi_async_rt::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_futures::BoxFuture;
use pi_render::depend_graph::{
    graph::DependGraph,
    node::{DependNode, ParamUsage},
    param::InParamCollector,
};
use pi_share::Share;
use render_derive::NodeParam;
use std::{any::TypeId, time::Duration};

// 多个 节点的 输入输出测试
#[test]
fn input_collector() {
    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = DependGraph::default();

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
        g.build().unwrap();

        println!("======== run graph");
        g.run(&rt, &()).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
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

        assert_eq!(input, &());

        assert!(!usage.is_input_fill(TypeId::of::<()>()));

        assert!(usage.is_output_usage(TypeId::of::<A>()));
        assert!(!usage.is_output_usage(TypeId::of::<B>()));

        Box::pin(async move {
            println!("======== Enter Async Node1 Running");

            Ok(Output1 {
                a: A(self.0),
                b: B(1),
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

        assert!(usage.is_input_fill(TypeId::of::<A>()));

        let c = &input.c.0;
        assert_eq!(c.len(), 5);
        for (id, v) in c {
            println!("id = {:?}, v = {:?}", id, v);
        }

        Box::pin(async move {
            println!("======== Enter Async Node2 Running");

            Ok(())
        })
    }
}
