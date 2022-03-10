use log::{debug, info};
use pi_async::rt::{
    single_thread::{SingleTaskPool, SingleTaskRunner},
    AsyncRuntime, AsyncTaskPool, AsyncTaskPoolExt,
};
use pi_ecs::prelude::{Dispatcher, SingleDispatcher, World};
use pi_render::{
    init_render, render_graph::graph::RenderGraph, render_nodes::clear_pass::ClearPassNode,
    rhi::options::RenderOptions, window::{windows::Windows, window::PiWindow}, RenderStage,
};
use pi_share::ShareRefCell;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

// 渲染 环境
struct RenderExample {
    rt: AsyncRuntime<(), SingleTaskPool<()>>,
    dispatcher: Option<SingleDispatcher<SingleTaskPool<()>>>,
}

impl RenderExample {
    pub fn new(rt: AsyncRuntime<(), SingleTaskPool<()>>) -> Self {
        Self {
            rt,
            dispatcher: None,
        }
    }

    pub async fn init<P>(
        &mut self,
        mut world: World,
        options: RenderOptions,
        rt: AsyncRuntime<(), P>,
    ) where
        P: AsyncTaskPoolExt<()> + AsyncTaskPool<(), Pool = P>,
    {
        info!("RenderExample::init");

        let RenderStage {
            extract_stage,
            prepare_stage,
            render_stage,
        } = init_render(&mut world, options, rt).await;

        let rg = world.get_resource_mut::<RenderGraph>().unwrap();
        let clear_node = rg.add_node("clean", ClearPassNode::new(&mut world.clone()));
        rg.set_node_finish(clear_node, true).unwrap();

        let mut stages = vec![];
        stages.push(Arc::new(extract_stage.build()));
        stages.push(Arc::new(prepare_stage.build()));
        stages.push(Arc::new(render_stage.build()));

        let rt = self.rt.clone();

        let dispatcher = SingleDispatcher::new(stages, &world, rt);

        self.dispatcher = Some(dispatcher);
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

fn run_window_loop(
    window: ShareRefCell<winit::window::Window>,
    event_loop: EventLoop<()>,
    example: ShareRefCell<RenderExample>,
    rt: AsyncRuntime<(), SingleTaskPool<()>>,
) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                info!("RenderExample::resize, size = {:?}", size);
                let w = size.width;
                let h = size.height;
                let e = example.clone();
                let _ = rt.spawn(rt.alloc(), async move {
                    e.resize(w, h);
                });
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                debug!("RenderExample::run");

                let e = example.clone();
                let _ = rt.spawn(rt.alloc(), async move {
                    e.render();
                });
            }
            Event::WindowEvent {
                // 窗口 关闭，退出 循环
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("RenderExample::clean");
                let e = example.clone();
                let _ = rt.spawn(rt.alloc(), async move {
                    e.clean();
                });

                *control_flow = ControlFlow::Exit
            }
            _ => {}
        }
    });
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();


    let event_loop = EventLoop::new();
    let window = ShareRefCell::new(winit::window::Window::new(&event_loop).unwrap());
    
    let world = World::new();    
    // Res Windows
    let windows = Windows::default();
    windows.add(PiWindow::new(window.clone()));
    world.insert_resource(windows);

    let runner = SingleTaskRunner::<()>::default();
    let runtime = AsyncRuntime::Local(runner.startup().unwrap());

    let rt = runtime.clone();
    let example = ShareRefCell::new(RenderExample::new(rt.clone()));
    let mut e = example.clone();
    std::thread::spawn(move || {
        let runtime = runtime.clone();

        let rt = runtime.clone();
        let _ = runtime.spawn(runtime.alloc(), async move {
            
            let options = RenderOptions::default();
            e.init(world, options, rt).await;
        });

        loop {
            let _ = runner.run();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    run_window_loop(window, event_loop, example, rt);
}