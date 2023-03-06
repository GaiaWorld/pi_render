
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EShaderStage {
    /// Binding is not visible from any shader stage.
    NONE,
    /// Binding is visible from the vertex shader of a render pipeline.
    VERTEX,
    /// Binding is visible from the fragment shader of a render pipeline.
    FRAGMENT,
    /// Binding is visible from the compute shader of a compute pipeline.
    COMPUTE,
    /// Binding is visible from the vertex and fragment shaders of a render pipeline.
    VERTEXFRAGMENT,
}
impl EShaderStage {
    pub fn mode(&self) -> wgpu::ShaderStages {
        match self {
            EShaderStage::NONE              => wgpu::ShaderStages::NONE              ,
            EShaderStage::VERTEX            => wgpu::ShaderStages::VERTEX            ,
            EShaderStage::FRAGMENT          => wgpu::ShaderStages::FRAGMENT          ,
            EShaderStage::COMPUTE           => wgpu::ShaderStages::COMPUTE           ,
            EShaderStage::VERTEXFRAGMENT    => wgpu::ShaderStages::VERTEX_FRAGMENT   ,
        }
    }
    pub fn new(stage: wgpu::ShaderStages) -> Self {
        if stage == wgpu::ShaderStages::COMPUTE {
            Self::COMPUTE
        } else if stage == wgpu::ShaderStages::VERTEX {
            Self::VERTEX
        } else if stage == wgpu::ShaderStages::FRAGMENT {
            Self::FRAGMENT
        } else if stage == wgpu::ShaderStages::VERTEX_FRAGMENT {
            Self::VERTEXFRAGMENT
        } else {
            Self::NONE
        }
    }
}