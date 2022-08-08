
pub struct PositionVertexBuffer;
impl PositionVertexBuffer {
	pub fn id() -> u32 {
		0
	}
}

pub struct UvVertexBuffer;
impl UvVertexBuffer {
	pub fn id() -> u32 {
		1
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

pub struct TextMaterialGroup;
impl pi_render::rhi::dyn_uniform_buffer::Group for TextMaterialGroup {
	fn id() -> u32 {
		1
	}

	fn create_layout(
		device: &pi_render::rhi::device::RenderDevice,
		has_dynamic_offset: bool,
	) -> pi_render::rhi::bind_group_layout::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("text_material bindgroup layout"),
			entries: &[wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset,
					min_binding_size: wgpu::BufferSize::new(112),
				},
				count: None, // TODO
			}],
		})
	}
}

impl pi_render::rhi::dyn_uniform_buffer::BufferGroup for TextMaterialGroup {
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
					size: Some(std::num::NonZeroU64::new(112).unwrap()),
				}),
			}],
			label: Some("text_material bindgroup"),
		})
	}
}

pub struct SampTex2DGroup;
impl pi_render::rhi::dyn_uniform_buffer::Group for SampTex2DGroup {
	fn id() -> u32 {
		2
	}

	fn create_layout(
		device: &pi_render::rhi::device::RenderDevice,
		has_dynamic_offset: bool,
	) -> pi_render::rhi::bind_group_layout::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("samp_tex_2d bindgroup layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
					},
					count: None, // TODO
				},
			],
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

pub struct TextMaterialBind;
impl pi_render::rhi::dyn_uniform_buffer::Bind for TextMaterialBind {
	#[inline]
	fn min_size() -> usize {
		112
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

pub struct TextureSizeUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for TextureSizeUniform<'a> {
	fn write_into(&self, index: u32, buffer: &mut [u8]) {
		unsafe {
			std::ptr::copy_nonoverlapping(
				self.0.as_ptr() as usize as *const u8,
				buffer.as_mut_ptr().add(index as usize + 72),
				8,
			)
		};
	}
}

pub struct UcolorUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for UcolorUniform<'a> {
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

pub struct StrokeColorUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for StrokeColorUniform<'a> {
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

pub struct TextShader;

impl TextShader {
	pub fn create_bind_group_samp_tex_2d(
		device: &pi_render::rhi::device::RenderDevice,
		layout: &pi_render::rhi::bind_group_layout::BindGroupLayout,
		samp: &wgpu::Sampler,
		tex_2d: &wgpu::TextureView,
	) -> pi_render::rhi::bind_group::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Sampler(samp),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(tex_2d),
				},
			],
			label: Some("samp_tex_2d bindgroup"),
		})
	}
}
