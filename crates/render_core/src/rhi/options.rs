use std::borrow::Cow;

#[derive(Clone, Debug)]
pub enum RenderPriority {
    Compatibility,
    Functionality,
    WebGL2,
}

#[derive(Clone, Debug)]
pub struct RenderOptions {
    pub present_mode: wgpu::PresentMode,
    pub device_label: Option<Cow<'static, str>>,
    pub backends: wgpu::Backends,
    pub power_preference: wgpu::PowerPreference,
    pub priority: RenderPriority,
    /// The features to ensure are enabled regardless of what the adapter/backend supports.
    /// Setting these explicitly may cause renderer initialization to fail.
    pub features: wgpu::Features,
    /// The features to ensure are disabled regardless of what the adapter/backend supports
    pub disabled_features: Option<wgpu::Features>,
    /// The imposed limits.
    pub limits: wgpu::Limits,
    /// The constraints on limits allowed regardless of what the adapter/backend supports
    pub constrained_limits: Option<wgpu::Limits>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        let default_backends = if cfg!(feature = "webgl") {
            wgpu::Backends::GL
        } else {
            wgpu::Backends::PRIMARY
        };

        let backends = wgpu::util::backend_bits_from_env().unwrap_or(default_backends);

        let priority = options_priority_from_env().unwrap_or(RenderPriority::Functionality);

        let limits = if cfg!(feature = "webgl") || matches!(priority, RenderPriority::WebGL2) {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            #[allow(unused_mut)]
            let mut limits = wgpu::Limits::default();
            #[cfg(feature = "ci_limits")]
            {
                limits.max_storage_textures_per_shader_stage = 4;
                limits.max_texture_dimension_3d = 1024;
            }
            limits
        };

        Self {
            present_mode: wgpu::PresentMode::Fifo,
            device_label: Default::default(),
            backends,
            power_preference: wgpu::PowerPreference::HighPerformance,
            priority,
            features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            disabled_features: None,
            limits,
            constrained_limits: None,
        }
    }
}

/// Get a features/limits priority from the environment variable `WGPU_OPTIONS_PRIO`
pub fn options_priority_from_env() -> Option<RenderPriority> {
    Some(
        match std::env::var("WGPU_OPTIONS_PRIO")
            .as_deref()
            .map(str::to_lowercase)
            .as_deref()
        {
            Ok("compatibility") => RenderPriority::Compatibility,
            Ok("functionality") => RenderPriority::Functionality,
            Ok("webgl2") => RenderPriority::WebGL2,
            _ => return None,
        },
    )
}
