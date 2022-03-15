use log::{debug, info};
use pi_async::rt::{
    single_thread::{SingleTaskPool, SingleTaskRunner},
    AsyncRuntime,
};
use pi_ecs::prelude::{Dispatcher, SingleDispatcher, World};
use pi_render::{
    init_render,
    render_graph::graph::RenderGraph,
    render_nodes::clear_pass::ClearPassNode,
    rhi::{options::RenderOptions, PresentMode},
    view::render_window::RenderWindow,
    RenderArchetype, RenderStage,
};
use pi_share::ShareRefCell;
use std::sync::Arc;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

// 渲染 环境
struct RenderExample {
    dispatcher: Option<SingleDispatcher<SingleTaskPool<()>>>,
}

impl RenderExample {
    pub fn new() -> Self {
        Self { dispatcher: None }
    }

    pub async fn init(
        &mut self,
        mut world: World,
        options: RenderOptions,
        window: ShareRefCell<winit::window::Window>,
        rt: AsyncRuntime<(), SingleTaskPool<()>>,
    ) {
        info!("RenderExample::init");

        // Render: stage && system
        let RenderStage {
            extract_stage,
            prepare_stage,
            render_stage,
        } = init_render(&mut world, options, window, rt.clone()).await;

        let mut stages = vec![];
        stages.push(Arc::new(extract_stage.build()));
        stages.push(Arc::new(prepare_stage.build()));
        stages.push(Arc::new(render_stage.build()));

        self.dispatcher = Some(SingleDispatcher::new(stages, &world, rt));

        // Render Graph
        let rg = world.get_resource_mut::<RenderGraph>().unwrap();
        let clear_node = rg.add_node("clean", ClearPassNode::new(&mut world.clone()));
        rg.set_node_finish(clear_node, true).unwrap();
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let event_loop = EventLoop::new();
    let window = ShareRefCell::new(winit::window::Window::new(&event_loop).unwrap());

    let mut world = World::new();
    
    world
        .spawn::<RenderArchetype>()
        .insert(RenderWindow::new(window.clone(), PresentMode::Mailbox));
        
    // TODO 准备 Entity: ClearOption, RenderTarget, Option<Viewport>, Option<Scissor>,
    000000000000000

    let runner = SingleTaskRunner::<()>::default();
    let runtime = AsyncRuntime::Local(runner.startup().unwrap());

    let rt = runtime.clone();
    let example = ShareRefCell::new(RenderExample::new());
    let mut e = example.clone();

    let win = window.clone();
    std::thread::spawn(move || {
        let runtime = runtime.clone();

        let rt = runtime.clone();
        let _ = runtime.spawn(runtime.alloc(), async move {
            let options = RenderOptions::default();
            e.init(world, options, win, rt).await;
        });

        loop {
            let _ = runner.run();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    run_window_loop(window, event_loop, example, rt);
}
