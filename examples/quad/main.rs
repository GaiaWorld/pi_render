mod quad;
mod scene_pass;

use log::{debug, info};
use pi_async::rt::{
    single_thread::{SingleTaskPool, SingleTaskRunner},
    AsyncRuntime,
};
use pi_ecs::prelude::{Dispatcher, SingleDispatcher, World};
use pi_render::{
    components::{
        camera::render_target::{RenderTarget, RenderTargets, TextureViews},
        view::render_window::{RenderWindow, RenderWindows},
    },
    graph::graph::RenderGraph,
    init_render,
    pass::clear_pass::{ClearOption, ClearOptions, ClearPassNode},
    phase::DrawFunctions,
    rhi::{options::RenderOptions, PresentMode},
    RenderStage,
};
use pi_share::ShareRefCell;
use quad::RenderItem;
use std::{default::Default, sync::Arc};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

// 渲染 环境
#[derive(Default)]
struct RenderExample {
    dispatcher: Option<SingleDispatcher<SingleTaskPool<()>>>,
}

impl RenderExample {
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

        let stages = vec![
            Arc::new(extract_stage.build()),
            Arc::new(prepare_stage.build()),
            Arc::new(render_stage.build()),
        ];

        self.dispatcher = Some(SingleDispatcher::new(stages, &world, rt));

        // Render Graph
        let rg = world.get_resource_mut::<RenderGraph>().unwrap();
        let clear_node = rg.add_node("clean", ClearPassNode);
        rg.set_node_finish(clear_node, true).unwrap();
    }

    // 窗口大小改变 时 调用一次
    pub fn resize(&self, w: u32, h: u32) {
        info!("RenderExample::resize({}, {})", w, h);
    }

    // 执行 窗口渲染，每帧调用一次
    pub fn render(&self) {}

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
                let w = size.width;
                let h = size.height;
                let e = example.clone();
                let _ = rt.spawn(rt.alloc(), async move {
                    info!("RenderExample::resize, size = {:?}", size);
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let event_loop = EventLoop::new();
    let window = ShareRefCell::new(winit::window::Window::new(&event_loop).unwrap());

    let world = World::new();

    let runner = SingleTaskRunner::<()>::default();
    let runtime = AsyncRuntime::Local(runner.startup().unwrap());

    let rt = runtime.clone();
    let example = ShareRefCell::new(RenderExample::default());
    let mut e = example.clone();

    let win = window.clone();
    std::thread::spawn(move || {
        let example = e.clone();

        let runtime = runtime.clone();

        let rt = runtime.clone();
        let _ = runtime.spawn(runtime.alloc(), async move {
            let options = RenderOptions::default();

            e.init(world.clone(), options, win.clone(), rt).await;

            // 取 TextureView
            let texture_views = world.get_resource_mut::<TextureViews>().unwrap();
            let view = texture_views.insert(None);

            // 创建 RenderWindow
            let render_window = RenderWindow::new(win, PresentMode::Mailbox, view);
            let render_windows = world.get_resource_mut::<RenderWindows>().unwrap();
            render_windows.insert(render_window);

            // 创建 RenderTarget
            let mut rt = RenderTarget::default();
            rt.add_color(view);
            let render_targets = world.get_resource_mut::<RenderTargets>().unwrap();
            let rt_key = render_targets.insert(rt);

            // 装配 清屏
            let mut clear_option = ClearOption::default();
            clear_option.set_color(1.0, 0.0, 0.0, 1.0);
            clear_option.set_target(rt_key);
            let clear_options = world.get_resource_mut::<ClearOptions>().unwrap();
            clear_options.insert(clear_option);
        });

        let mut frame = 0;
        loop {
            frame += 1;
            debug!("=================== frame = {}", frame);

            match &example.dispatcher {
                Some(d) => {
                    d.run();
                }
                None => {}
            }

            loop {
                let count = runner.run().unwrap();
                if count == 0 {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    });

    run_window_loop(window, event_loop, example, rt);
}

fn insert_resource(world: World) {
    world.insert_resource(DrawFunctions::<RenderItem>::default());
}
