
pub struct ColorShader {
	pub context: Share<ShareMutex<BindGroupContext>>,
}

impl ColorShader {
	pub const GROUP_TEX_2D_SAMP: u32 = 2;

	pub const GROUP_OTHER: u32 = 1;

	pub const GROUP_PROJECTMATRIX: u32 = 0;

	pub fn create_bind_group_layout_tex_2d_samp(device: &RenderDevice) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("tex_2d_samp bindgroup layout"),
			entries: &[
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
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
		})
	}

	pub fn create_bind_group_tex_2d_samp(
		device: &RenderDevice,
		layout: &BindGroupLayout,
		tex_2d: &wgpu::TextureView,
		samp: &wgpu::Sampler,
	) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(tex_2d),
				},
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Sampler(samp),
				},
			],
			label: Some("tex_2d_samp bindgroup"),
		})
	}

	pub fn set_world_matrix(&mut self, index: &BlockIndex, world_matrix: &[f32]) {
		self.context.lock().unwrap().set_uniform(index, 0, color);
	}

	pub fn set_color(&mut self, index: &BlockIndex, color: &[f32]) {
		self.context.lock().unwrap().set_uniform(index, 64, color);
	}

	pub fn set_depth(&mut self, index: &BlockIndex, depth: &[f32]) {
		self.context.lock().unwrap().set_uniform(index, 80, color);
	}

	pub fn create_bind_group_layout_other(device: &RenderDevice) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("other bindgroup layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset,
						min_binding_size: wgpu::BufferSize::new(64),
					},
					count: None, // TODO
				},
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset,
						min_binding_size: wgpu::BufferSize::new(16),
					},
					count: None, // TODO
				},
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset,
						min_binding_size: wgpu::BufferSize::new(4),
					},
					count: None, // TODO
				},
			],
		})
	}

	pub fn create_bind_group_other(
		device: &RenderDevice,
		layout: &BindGroupLayout,
		has_dynamic_offset: bool,
		has_dynamic_offset: bool,
		has_dynamic_offset: bool,
	) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 0,
						size: Some(64),
					}),
				},
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 64,
						size: Some(16),
					}),
				},
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 80,
						size: Some(4),
					}),
				},
			],
			label: Some("other bindgroup"),
		})
	}

	pub fn set_project_matrix(&mut self, index: &BlockIndex, project_matrix: &[f32]) {
		self.context.lock().unwrap().set_uniform(index, 0, color);
	}

	pub fn set_view_matrix(&mut self, index: &BlockIndex, view_matrix: &[f32]) {
		self.context.lock().unwrap().set_uniform(index, 64, color);
	}

	pub fn create_bind_group_layout_projectmatrix(device: &RenderDevice) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("projectmatrix bindgroup layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset,
						min_binding_size: wgpu::BufferSize::new(64),
					},
					count: None, // TODO
				},
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset,
						min_binding_size: wgpu::BufferSize::new(64),
					},
					count: None, // TODO
				},
			],
		})
	}

	pub fn create_bind_group_projectmatrix(
		device: &RenderDevice,
		layout: &BindGroupLayout,
		has_dynamic_offset: bool,
		has_dynamic_offset: bool,
	) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 0,
						size: Some(64),
					}),
				},
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 64,
						size: Some(64),
					}),
				},
			],
			label: Some("projectmatrix bindgroup"),
		})
	}
}
