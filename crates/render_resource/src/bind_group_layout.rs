
pub trait AsUniformBindingBufferDynamic {
    const BINDING: u32;
    const VISIBILITY: wgpu::ShaderStages;
    const SIZE: wgpu::BufferAddress;

    const BINDING_TYPE: wgpu::BindingType = wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: true,
        min_binding_size: wgpu::BufferSize::new(Self::SIZE),
    };
    const LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: Self::BINDING,
        visibility: Self::VISIBILITY,
        ty: Self::BINDING_TYPE,
        count: None,
    };

    fn bind_group_entry<'a>(&self) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: Self::BINDING,
            resource: self.binding_resource(),
        }
    }
    fn binding_resource<'a>(&self) -> wgpu::BindingResource<'a>;

    fn offset(&self) -> wgpu::BufferAddress;
}

pub trait AsUniformBindingBufferStatic {
    const BINDING: u32;
    const VISIBILITY: wgpu::ShaderStages;
    const SIZE: wgpu::BufferAddress;
    const OFFSET: wgpu::BufferAddress;

    const BINDING_TYPE: wgpu::BindingType = wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: wgpu::BufferSize::new(Self::SIZE),
    };
    const LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: Self::BINDING,
        visibility: Self::VISIBILITY,
        ty: Self::BINDING_TYPE,
        count: None,
    };

    fn bind_group_entry<'a>(&self) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: Self::BINDING,
            resource: self.binding_resource(),
        }
    }
    fn binding_resource<'a>(&self) -> wgpu::BindingResource<'a>;

    fn offset(&self) -> wgpu::BufferAddress;
}

pub trait AsUniformBindingTextureView {
    const BINDING: u32;
    const VISIBILITY: wgpu::ShaderStages;
    const SAMPLER_TYPE: wgpu::TextureSampleType;
    const DIMENSION: wgpu::TextureViewDimension;
    const MULTISAMPLED: bool;

    const BINDING_TYPE: wgpu::BindingType = wgpu::BindingType::Texture {
        sample_type: Self::SAMPLER_TYPE,
        view_dimension: Self::DIMENSION,
        multisampled: Self::MULTISAMPLED,
    };
    const LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: Self::BINDING,
        visibility: Self::VISIBILITY,
        ty: Self::BINDING_TYPE,
        count: None,
    };

    fn bind_group_entry<'a>(&self) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: Self::BINDING,
            resource: self.binding_resource(),
        }
    }
    fn binding_resource<'a>(&self) -> wgpu::BindingResource<'a>;
}

pub trait AsUniformBindingSampler {
    const BINDING: u32;
    const VISIBILITY: wgpu::ShaderStages;
    const SAMPLER_BINDING_TYPE: wgpu::SamplerBindingType;

    const BINDING_TYPE: wgpu::BindingType = wgpu::BindingType::Sampler(
        Self::SAMPLER_BINDING_TYPE
    );
    const LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
        binding: Self::BINDING,
        visibility: Self::VISIBILITY,
        ty: Self::BINDING_TYPE,
        count: None,
    };

    fn bind_group_entry<'a>(&self) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: Self::BINDING,
            resource: self.binding_resource(),
        }
    }
    fn binding_resource<'a>(&self) -> wgpu::BindingResource<'a>;
}