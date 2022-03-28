use log::{debug, info};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

// 渲染 环境
struct RenderExample {
    window: Window,
}

impl RenderExample {
    pub fn new(window: Window) -> Self {
        Self { window }
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
        // info!("RenderExample::render");
    }

    pub fn clean(&self) {
        info!("RenderExample::clean");
    }
}

fn run(event_loop: EventLoop<()>, window: Window) {
    let example = RenderExample::new(window);

    example.init();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                example.resize(size.width, size.height);
            }
            Event::MainEventsCleared => {
                example.window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                example.render();
            }
            Event::WindowEvent {
                // 窗口 关闭，退出 循环
                event: WindowEvent::CloseRequested,
                ..
            } => {
                example.clean();

                *control_flow = ControlFlow::Exit
            }
            _ => {}
        }
    });
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();

    run(event_loop, window);
}
