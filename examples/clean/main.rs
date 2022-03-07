use log::{debug, info};
use pi_async::rt::{
    single_thread::{SingleTaskPool, SingleTaskRunner},
    AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt,
};
use pi_ecs::prelude::{Dispatcher, SingleDispatcher, World};
use pi_render::{
    create_instance_surface, init_render, render_graph::graph::RenderGraph,
    render_nodes::clear_pass::ClearPassNode, rhi::options::RenderOptions,
};
use pi_share::cell::TrustCell;
use std::{ops::DerefMut, sync::Arc};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// 渲染 环境
struct RenderExample {
    world: World,
    window: Window,
    rt: AsyncRuntime<(), SingleTaskPool<()>>,
    dispatcher: Option<SingleDispatcher<SingleTaskPool<()>>>,
}

impl RenderExample {
    pub fn new(window: Window, rt: AsyncRuntime<(), SingleTaskPool<()>>) -> Self {
        let world = World::new();

        Self {
            window,
            world,
            rt,
            dispatcher: None,
        }
    }

    pub async fn init_render<P>(
        &mut self,
        instance: wgpu::Instance,
        surface: wgpu::Surface,
        options: RenderOptions,
        rt: AsyncRuntime<(), P>,
    ) where
        P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
    {
        let render_stage = init_render(&mut self.world, instance, surface, options, rt).await;

        let rg = self.world.get_resource_mut::<RenderGraph>().unwrap();
        let clear_node = rg.add_node("clean", ClearPassNode::new(&mut self.world.clone()));
        rg.set_node_finish(clear_node, true).unwrap();

        let mut stages = vec![];
        stages.push(Arc::new(render_stage.build()));

        let rt = self.rt.clone();

        let dispatcher = SingleDispatcher::new(stages, &self.world, rt);

        self.dispatcher = Some(dispatcher);
    }

    // 初始化渲染调用
    pub fn init(&self) {
        info!("RenderExample::init");
    }

    // 窗口大小改变 时 调用一次
    pub fn resize(&self, w: u32, h: u32) {
        info!("RenderExample::resize({}, {})", w, h);
    }

    // 执行 窗口渲染，每帧调用一次
    pub fn render(&self) {
        debug!("RenderExample::render");

        match &self.dispatcher {
            Some(d) => d.run(),
            None => {}
        }
    }

    pub fn clean(&self) {
        info!("RenderExample::clean");
    }
}

fn run(event_loop: EventLoop<()>, window: Window) {
    let runner = SingleTaskRunner::<()>::default();
    let runtime = runner.startup().unwrap();
    let single = AsyncRuntime::Local(runtime.clone());

    let single_clone = single.clone();

    let options = RenderOptions::default();

    let (instance, surface) = create_instance_surface(&window, &options);

    let example = Arc::new(TrustCell::new(RenderExample::new(window, single_clone)));

    let e = example.clone();
    let s = single.clone();

    let _ = single.spawn(single.alloc(), async move {
        e.get().init();

        let mut e = e.borrow_mut();
        let e = e.deref_mut();
        e.init_render(instance, surface, options, s.clone()).await;
    });

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let w = size.width;
                let h = size.height;
                let e = example.clone();
                let _ = single.spawn(single.alloc(), async move {
                    e.get().resize(w, h);
                });
            }
            Event::MainEventsCleared => {
                let e = example.clone();
                let _ = single.spawn(single.alloc(), async move {
                    e.get().window.request_redraw();
                });
            }
            Event::RedrawRequested(_) => {
                let e = example.clone();
                let _ = single.spawn(single.alloc(), async move {
                    e.get().render();
                });

                let _ = runner.run();
            }
            Event::WindowEvent {
                // 窗口 关闭，退出 循环
                event: WindowEvent::CloseRequested,
                ..
            } => {
                let e = example.clone();
                let _ = single.spawn(single.alloc(), async move {
                    e.get().clean();
                });

                *control_flow = ControlFlow::Exit
            }
            _ => {}
        }
    });
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    run(event_loop, window);
}
