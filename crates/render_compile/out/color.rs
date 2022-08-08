pub struct PositionVertexBuffer;
impl PositionVertexBuffer {
	pub fn id() -> u32 {
		0
	}
}

pub struct CameraMatrixGroup;
impl pi_render::rhi::dyn_uniform_buffer::Group for CameraMatrixGroup {
	fn id() -> u32 {
		0
	}

	fn create_layout(
		device: &pi_render::rhi::device::RenderDevice,
		has_dynamic_offset: bool,
	) -> pi_render::rhi::bind_group_layout::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("camera_matrix bindgroup layout"),
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset,
					min_binding_size: wgpu::BufferSize::new(128),
				},
				count: None, // TODO
			}],
		})
	}
}

impl pi_render::rhi::dyn_uniform_buffer::BufferGroup for CameraMatrixGroup {
	fn create_bind_group(
		device: &pi_render::rhi::device::RenderDevice,
		layout: &pi_render::rhi::bind_group_layout::BindGroupLayout,
		buffer: &pi_render::rhi::buffer::Buffer,
	) -> pi_render::rhi::bind_group::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
					buffer,
					offset: 0,
					size: Some(std::num::NonZeroU64::new(128).unwrap()),
				}),
			}],
			label: Some("camera_matrix bindgroup"),
		})
	}
}

pub struct ColorMaterialGroup;
impl pi_render::rhi::dyn_uniform_buffer::Group for ColorMaterialGroup {
	fn id() -> u32 {
		1
	}

	fn create_layout(
		device: &pi_render::rhi::device::RenderDevice,
		has_dynamic_offset: bool,
	) -> pi_render::rhi::bind_group_layout::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("color_material bindgroup layout"),
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset,
					min_binding_size: wgpu::BufferSize::new(128),
				},
				count: None, // TODO
			}],
		})
	}
}

impl pi_render::rhi::dyn_uniform_buffer::BufferGroup for ColorMaterialGroup {
	fn create_bind_group(
		device: &pi_render::rhi::device::RenderDevice,
		layout: &pi_render::rhi::bind_group_layout::BindGroupLayout,
		buffer: &pi_render::rhi::buffer::Buffer,
	) -> pi_render::rhi::bind_group::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
					buffer,
					offset: 0,
					size: Some(std::num::NonZeroU64::new(128).unwrap()),
				}),
			}],
			label: Some("color_material bindgroup"),
		})
	}
}

pub struct CameraMatrixBind;
impl pi_render::rhi::dyn_uniform_buffer::Bind for CameraMatrixBind {
	#[inline]
	fn min_size() -> usize {
		128
	}

	fn index() -> pi_render::rhi::dyn_uniform_buffer::BindIndex {
		pi_render::rhi::dyn_uniform_buffer::BindIndex::new(0)
	}
}

pub struct ColorMaterialBind;
impl pi_render::rhi::dyn_uniform_buffer::Bind for ColorMaterialBind {
	#[inline]
	fn min_size() -> usize {
		128
	}

	fn index() -> pi_render::rhi::dyn_uniform_buffer::BindIndex {
		pi_render::rhi::dyn_uniform_buffer::BindIndex::new(0)
	}
}

pub struct ProjectUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for ProjectUniform<'a> {
	fn write_into(&self, index: u32, buffer: &mut [u8]) {
		unsafe {
			std::ptr::copy_nonoverlapping(
				self.0.as_ptr() as usize as *const u8,
				buffer.as_mut_ptr().add(index as usize + 0),
				64,
			)
		};
	}
}

pub struct ViewUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for ViewUniform<'a> {
	fn write_into(&self, index: u32, buffer: &mut [u8]) {
		unsafe {
			std::ptr::copy_nonoverlapping(
				self.0.as_ptr() as usize as *const u8,
				buffer.as_mut_ptr().add(index as usize + 64),
				64,
			)
		};
	}
}

pub struct WorldUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for WorldUniform<'a> {
	fn write_into(&self, index: u32, buffer: &mut [u8]) {
		unsafe {
			std::ptr::copy_nonoverlapping(
				self.0.as_ptr() as usize as *const u8,
				buffer.as_mut_ptr().add(index as usize + 0),
				64,
			)
		};
	}
}

pub struct DepthUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for DepthUniform<'a> {
	fn write_into(&self, index: u32, buffer: &mut [u8]) {
		unsafe {
			std::ptr::copy_nonoverlapping(
				self.0.as_ptr() as usize as *const u8,
				buffer.as_mut_ptr().add(index as usize + 64),
				4,
			)
		};
	}
}

pub struct ColorUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for ColorUniform<'a> {
	fn write_into(&self, index: u32, buffer: &mut [u8]) {
		unsafe {
			std::ptr::copy_nonoverlapping(
				self.0.as_ptr() as usize as *const u8,
				buffer.as_mut_ptr().add(index as usize + 80),
				16,
			)
		};
	}
}

pub struct UrectUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for UrectUniform<'a> {
	fn write_into(&self, index: u32, buffer: &mut [u8]) {
		unsafe {
			std::ptr::copy_nonoverlapping(
				self.0.as_ptr() as usize as *const u8,
				buffer.as_mut_ptr().add(index as usize + 96),
				16,
			)
		};
	}
}

pub struct BlurUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for BlurUniform<'a> {
	fn write_into(&self, index: u32, buffer: &mut [u8]) {
		unsafe {
			std::ptr::copy_nonoverlapping(
				self.0.as_ptr() as usize as *const u8,
				buffer.as_mut_ptr().add(index as usize + 112),
				4,
			)
		};
	}
}

pub struct ColorShader;

impl ColorShader {}
