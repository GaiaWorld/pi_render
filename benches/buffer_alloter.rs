use criterion::{criterion_group, criterion_main, Criterion};
use pi_share::{ShareMutex, Share};
use render_core::rhi::{buffer_alloc::BufferAlloter, device::RenderDevice, RenderQueue};
use pi_async_rt::rt::AsyncRuntime;
use wgpu::{Gles3MinorVersion, InstanceFlags};
use winit::{event_loop::EventLoopBuilder, platform::windows::EventLoopBuilderExtWindows};

use render_core::rhi::{device::initialize_renderer, options::RenderOptions};

fn buffer_alloter(c: &mut Criterion) {
	let mut group = c.benchmark_group("buffer_alloter");
	let options = RenderOptions::default();
	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
		// Which `Backends` to enable.
		backends: options.backends,
		// Which DX12 shader compiler to use.
		dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
		flags: InstanceFlags::DEBUG,
		gles_minor_version: Gles3MinorVersion::Automatic,
	});
	let event_loop = EventLoopBuilder::new().with_any_thread(true).build();
	let window = winit::window::Window::new(&event_loop).unwrap();

	let surface = instance.create_surface(window).unwrap();

	let result: Share<ShareMutex<Option<(RenderDevice, RenderQueue)>>> = Share::new(ShareMutex::new(None));
	let result1 = result.clone();

	let _ = pi_hal::runtime::MULTI_MEDIA_RUNTIME.spawn( async move {
		let request_adapter_options = wgpu::RequestAdapterOptions {
			power_preference: options.power_preference,
			compatible_surface: Some(&surface),
			..Default::default()
		};
		let mut alloter = pi_assets::allocator::Allocator::new(32 * 1024 * 1024);
		let (device, queue, _adapter_info) =
		initialize_renderer(&instance, &options, &request_adapter_options, &mut alloter).await;
		let mut r = result1.lock().unwrap();
		*r = Some((device, queue));
	});

	let max_align = 1024;

	loop {
		let lock = result.lock().unwrap();
		if let Some((device, queue)) = &*lock {
			// level2， 由于max_align为128， 该buffer应该创建独立的buffer
			let mut level2_buffer = Vec::new();
			for _i in 0..max_align{
				level2_buffer.push(1)
			}

			let mut level3_buffer = Vec::new();
			for _i in 0..max_align + 4 {
				level3_buffer.push(1)
			}

			let alloter = BufferAlloter::new(device.clone(), queue.clone(), max_align, wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX);
				
			group.bench_function("alloter_align_bench", |b| {
				b.iter(|| {
					alloter.alloc(level2_buffer.as_slice());
				})
			});
			group.bench_function("alloter_not_align", |b| {
				b.iter(|| {
					alloter.alloc(level3_buffer.as_slice());
				})
			});
			group.bench_function("copy_to_nonoverlapping", |b| {
				let mut vec = Vec::with_capacity(max_align as usize); 
				b.iter(|| {
					unsafe { level2_buffer.as_ptr().copy_to_nonoverlapping(vec.as_mut_ptr(), max_align as usize) };
				})
			});
			
			break;
		}
	}
}


criterion_group!(benches, buffer_alloter);
criterion_main!(benches);