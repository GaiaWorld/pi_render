use pi_async::rt::{AsyncRuntime, AsyncRuntimeBuilder};
use pi_futures::BoxFuture;
use pi_render::depend_graph::{
    graph::DependGraph,
    node::{DependNode, ParamUsage},
};
use pi_share::Share;
use render_derive::NodeParam;
use std::{any::TypeId, time::Duration};

// 多个 节点的 输入输出测试
#[test]
fn multi_node() {
    let runtime = AsyncRuntimeBuilder::default_worker_thread(None, None, None, None);

    let mut g = DependGraph::default();

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

        g.build().unwrap();

        println!("======== 1 run graph");
        g.run(&rt).await.unwrap();

        println!("======== 2 run graph");
        g.run(&rt).await.unwrap();
    });

    std::thread::sleep(Duration::from_secs(3));
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
impl DependNode for Node1 {
    type Input = ();
    type Output = Output1;

    fn run<'a>(
        &'a mut self,
        input: &Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node1 Running");

        assert_eq!(input, &());

        assert!(!usage.is_input_fill(TypeId::of::<()>()));

        assert!(usage.is_output_usage(TypeId::of::<A>()));
        assert!(!usage.is_output_usage(TypeId::of::<B>()));

        Box::pin(async move {
            println!("======== Enter Async Node1 Running");

            Ok(Output1 { a: A(1), b: B(2) })
        })
    }
}

struct Node2;
impl DependNode for Node2 {
    type Input = ();
    type Output = Output2;

    fn run<'a>(
        &'a mut self,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node2 Running");

        assert!(!usage.is_input_fill(TypeId::of::<()>()));

        assert!(usage.is_output_usage(TypeId::of::<C>()));
        assert!(!usage.is_output_usage(TypeId::of::<D>()));

        Box::pin(async move {
            println!("======== Enter Async Node2 Running");

            Ok(Output2 {
                c: C("3".to_string()),
                d: D(44),
            })
        })
    }
}

struct Node3;
impl DependNode for Node3 {
    type Input = ();
    type Output = Output3;

    fn run<'a>(
        &'a mut self,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node3 Running");

        assert!(!usage.is_input_fill(TypeId::of::<()>()));

        assert!(!usage.is_output_usage(TypeId::of::<A>()));
        assert!(usage.is_output_usage(TypeId::of::<E>()));

        Box::pin(async move {
            println!("======== Enter Async Node3 Running");

            Ok(Output3 { a: A(36), e: E(45) })
        })
    }
}

struct Node4;
impl DependNode for Node4 {
    type Input = Input4;
    type Output = A;

    fn run<'a>(
        &'a mut self,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node4 Running");

        assert!(usage.is_input_fill(TypeId::of::<A>()));
        assert!(usage.is_input_fill(TypeId::of::<C>()));

        assert!(usage.is_output_usage(TypeId::of::<A>()));

        assert_eq!(input.a, A(1));
        assert_eq!(input.c, C("3".to_string()));

        Box::pin(async move {
            println!("======== Enter Async Node4 Running");

            Ok(A(44))
        })
    }
}

struct Node5;
impl DependNode for Node5 {
    type Input = Input5;
    type Output = F;

    fn run<'a>(
        &'a mut self,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node5 Running");

        assert!(usage.is_input_fill(TypeId::of::<C>()));
        assert!(usage.is_input_fill(TypeId::of::<E>()));

        assert!(!usage.is_output_usage(TypeId::of::<F>()));

        assert_eq!(input.c, C("3".to_string()));
        assert_eq!(input.e, E(45));

        Box::pin(async move {
            println!("======== Enter Async Node5 Running");

            Ok(F(55))
        })
    }
}

struct Node6;
impl DependNode for Node6 {
    type Input = A;
    type Output = ();

    fn run<'a>(
        &'a mut self,
        input: &'a Self::Input,
        usage: &'a ParamUsage,
    ) -> BoxFuture<'a, Result<Self::Output, String>> {
        println!("======== Enter Node6 Running");

        assert!(usage.is_input_fill(TypeId::of::<A>()));

        assert!(!usage.is_output_usage(TypeId::of::<()>()));

        assert_eq!(*input, A(44));

        Box::pin(async move {
            println!("======== Enter Async Node6 Running");

            Ok(())
        })
    }
}
