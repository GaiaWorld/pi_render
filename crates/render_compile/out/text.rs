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
				visibility: wgpu::ShaderStages::VERTEX,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset,
					min_binding_size: wgpu::BufferSize::new(96),
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
					size: Some(std::num::NonZeroU64::new(96).unwrap()),
				}),
			}],
			label: Some("color_material bindgroup"),
		})
	}
}

pub struct ColorMaterialBind;
impl pi_render::rhi::dyn_uniform_buffer::Bind for ColorMaterialBind {
	#[inline]
	fn min_size() -> usize {
		96
	}

	fn index() -> pi_render::rhi::dyn_uniform_buffer::BindIndex {
		pi_render::rhi::dyn_uniform_buffer::BindIndex::new(0)
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

pub struct ZzUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for ZzUniform<'a> {
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

pub struct XxxxUniform<'a>(pub &'a [f32]);
impl<'a> pi_render::rhi::dyn_uniform_buffer::Uniform for XxxxUniform<'a> {
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

pub struct TextShader;

impl TextShader {}
