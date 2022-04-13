use futures::future::{BoxFuture, FutureExt};
use naga::{back::wgsl::WriterFlags, valid::ModuleInfo, Module};
use once_cell::sync::Lazy;
use pi_hash::{XHashMap, XHashSet};
use regex::Regex;
use std::{borrow::Cow, ops::Deref, path::PathBuf, sync::Arc};
use thiserror::Error;
use uuid::Uuid;
use wgpu::{util::make_spirv, ShaderModuleDescriptor, ShaderSource};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ShaderId(Uuid);

impl ShaderId {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        ShaderId(Uuid::new_v4())
    }
}

/// Shader 解析 错误
#[derive(Error, Debug)]
pub enum ShaderReflectError {
    #[error(transparent)]
    WgslParse(#[from] naga::front::wgsl::ParseError),

    #[error("GLSL Parse Error: {0:?}")]
    GlslParse(Vec<naga::front::glsl::Error>),

    #[error(transparent)]
    SpirVParse(#[from] naga::front::spv::Error),

    #[error(transparent)]
    Validation(#[from] naga::WithSpan<naga::valid::ValidationError>),
}

/// Shader: [`ShaderSource`] and [`ShaderStage`](naga::ShaderStage)
/// 未解析的 Shader，包含 预处理 #import 指令
#[derive(Debug, Clone)]
pub struct Shader {
    id: ShaderId,
    source: Source,
    import_path: Option<ShaderImport>,
    imports: Vec<ShaderImport>,
}

impl Shader {
    /// 从 wgsl 得到的 Shader
    /// 注：wgsl 不需要指定 是哪个 stage 的 shader，因为里面有标识符指定
    pub fn from_wgsl(source: impl Into<Cow<'static, str>>) -> Shader {
        let source = source.into();
        Shader {
            id: ShaderId::new(),
            imports: SHADER_IMPORT_PROCESSOR.get_imports_from_str(&source),
            source: Source::Wgsl(source),
            import_path: None,
        }
    }

    /// 从 glsl 得到的 Shader
    /// 需要指定 stage: vs, fs, cs
    pub fn from_glsl(source: impl Into<Cow<'static, str>>, stage: naga::ShaderStage) -> Shader {
        let source = source.into();

        Shader {
            id: ShaderId::new(),
            imports: SHADER_IMPORT_PROCESSOR.get_imports_from_str(&source),
            source: Source::Glsl(source, stage),
            import_path: None,
        }
    }

    /// 从 spirv 二进制 得到的 Shader
    pub fn from_spirv(source: impl Into<Cow<'static, [u8]>>) -> Shader {
        Shader {
            id: ShaderId::new(),
            imports: Vec::new(),
            source: Source::SpirV(source.into()),
            import_path: None,
        }
    }

    pub fn id(&self) -> ShaderId {
        self.id
    }

    /// 设置 #import
    pub fn set_import_path<P: Into<String>>(&mut self, import_path: P) {
        self.import_path = Some(ShaderImport::Custom(import_path.into()));
    }

    /// 用 import-path，返回自身
    pub fn with_import_path<P: Into<String>>(mut self, import_path: P) -> Self {
        self.set_import_path(import_path);
        self
    }

    #[inline]
    pub fn import_path(&self) -> Option<&ShaderImport> {
        self.import_path.as_ref()
    }

    /// 返回迭代器
    pub fn imports(&self) -> impl ExactSizeIterator<Item = &ShaderImport> {
        self.imports.iter()
    }
}

/// Shader 源码
#[derive(Debug, Clone)]
pub enum Source {
    Wgsl(Cow<'static, str>),
    Glsl(Cow<'static, str>, naga::ShaderStage),
    SpirV(Cow<'static, [u8]>),
}

/// 已经 经过 预处理的 [Shader]
/// 不能含：#import 或 #ifdef 之类的预处理指令
#[derive(PartialEq, Eq, Debug)]
pub enum ProcessedShader {
    Wgsl(Cow<'static, str>),
    Glsl(Cow<'static, str>, naga::ShaderStage),
    SpirV(Cow<'static, [u8]>),
}

impl ProcessedShader {
    /// 取 wgsl 源码
    pub fn get_wgsl_source(&self) -> Option<&str> {
        if let ProcessedShader::Wgsl(source) = self {
            Some(source)
        } else {
            None
        }
    }

    /// 取 glsl 源码
    pub fn get_glsl_source(&self) -> Option<&str> {
        if let ProcessedShader::Glsl(source, _stage) = self {
            Some(source)
        } else {
            None
        }
    }

    /// 反射
    pub fn reflect(&self) -> Result<ShaderReflection, ShaderReflectError> {
        let module = match &self {
            ProcessedShader::Wgsl(source) => naga::front::wgsl::parse_str(source)?,
            ProcessedShader::Glsl(source, shader_stage) => {
                let mut parser = naga::front::glsl::Parser::default();
                parser
                    .parse(&naga::front::glsl::Options::from(*shader_stage), source)
                    .map_err(ShaderReflectError::GlslParse)?
            }
            ProcessedShader::SpirV(source) => naga::front::spv::parse_u8_slice(
                source,
                &naga::front::spv::Options {
                    adjust_coordinate_space: false,
                    ..naga::front::spv::Options::default()
                },
            )?,
        };

        // 取 Module 信息
        let module_info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::default(),
            naga::valid::Capabilities::default(),
        )
        .validate(&module)?;

        Ok(ShaderReflection {
            module,
            module_info,
        })
    }

    /// 取对应的 描述符
    pub fn get_module_descriptor(&self) -> Result<ShaderModuleDescriptor, AsModuleDescriptorError> {
        Ok(ShaderModuleDescriptor {
            label: None,
            source: match self {
                ProcessedShader::Wgsl(source) => {
                    // This isn't neccessary, but catches errors early during hot reloading of invalid wgsl shaders.
                    // Eventually, wgpu will have features that will make this unneccessary like compilation info or error scopes, but until then parsing the shader twice during development the easiest solution.
                    #[cfg(debug_assertions)]
                    let _ = self.reflect()?;

                    ShaderSource::Wgsl(source.clone())
                }
                ProcessedShader::Glsl(_source, _stage) => {
                    let reflection = self.reflect()?;

                    // 通过 反射信息 转换成 wgsl
                    let wgsl = reflection.get_wgsl()?;
                    ShaderSource::Wgsl(wgsl.into())
                }
                ProcessedShader::SpirV(source) => make_spirv(source),
            },
        })
    }
}

#[derive(Error, Debug)]
pub enum AsModuleDescriptorError {
    #[error(transparent)]
    ShaderReflectError(#[from] ShaderReflectError),
    #[error(transparent)]
    WgslConversion(#[from] naga::back::wgsl::Error),
    #[error(transparent)]
    SpirVConversion(#[from] naga::back::spv::Error),
}

/// Shader 反射 信息
pub struct ShaderReflection {
    /// naga 模块
    pub module: Module,
    /// naga 模块 信息
    pub module_info: ModuleInfo,
}

impl ShaderReflection {
    /// 转 spriv 二进制
    pub fn get_spirv(&self) -> Result<Vec<u32>, naga::back::spv::Error> {
        naga::back::spv::write_vec(
            &self.module,
            &self.module_info,
            &naga::back::spv::Options {
                flags: naga::back::spv::WriterFlags::empty(),
                ..naga::back::spv::Options::default()
            },
            None,
        )
    }

    /// 转 wgsl
    pub fn get_wgsl(&self) -> Result<String, naga::back::wgsl::Error> {
        naga::back::wgsl::write_string(&self.module, &self.module_info, WriterFlags::EXPLICIT_TYPES)
    }
}

/// 加载 Shader
/// 得到 未经过预处理的 Shader
pub fn load_shader(
    path: PathBuf,
    bytes: Arc<[u8]>,
) -> BoxFuture<'static, Result<Shader, anyhow::Error>> {
    async move {
        // 根据后缀名判断 是 那种类型
        let ext = path.extension().unwrap().to_str().unwrap();

        let mut shader = match ext {
            "spv" => Shader::from_spirv(bytes.to_vec()),
            "wgsl" => Shader::from_wgsl(String::from_utf8(bytes.to_vec()).unwrap()),
            "vert" => Shader::from_glsl(
                String::from_utf8(bytes.to_vec()).unwrap(),
                naga::ShaderStage::Vertex,
            ),
            "frag" => Shader::from_glsl(
                String::from_utf8(bytes.to_vec()).unwrap(),
                naga::ShaderStage::Fragment,
            ),
            _ => panic!("unhandled extension: {}", ext),
        };

        shader.import_path = Some(ShaderImport::Path(path.to_string_lossy().to_string()));

        Ok(shader)
    }
    .boxed()
}

/// 预处理 Shader 遇到的 错误
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProcessShaderError {
    #[error("Too many '# endif' lines. Each endif should be preceded by an if statement.")]
    TooManyEndIfs,

    #[error(
        "Not enough '# endif' lines. Each if statement should be followed by an endif statement."
    )]
    NotEnoughEndIfs,

    #[error("This Shader's format does not support processing shader defs.")]
    ShaderFormatDoesNotSupportShaderDefs,

    #[error("This Shader's formatdoes not support imports.")]
    ShaderFormatDoesNotSupportImports,

    #[error("Unresolved import: {0:?}.")]
    UnresolvedImport(ShaderImport),

    #[error("The shader import {0:?} does not match the source file type. Support for this might be added in the future.")]
    MismatchedImportFormat(ShaderImport),
}

/// 预处理器：#import
pub struct ShaderImportProcessor {
    /// 路径 识别 #import "..."
    import_asset_path_regex: Regex,
    /// 定制 识别 #import ...
    import_custom_path_regex: Regex,
}

/// 每一条 #import 信息
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ShaderImport {
    /// 路径 识别
    Path(String),
    /// 定制，具体语义由 高层解析
    Custom(String),
}

impl Default for ShaderImportProcessor {
    fn default() -> Self {
        Self {
            // #import "..."
            import_asset_path_regex: Regex::new(r#"^\s*#\s*import\s*"(.+)""#).unwrap(),
            // #import ...
            import_custom_path_regex: Regex::new(r"^\s*#\s*import\s*(.+)").unwrap(),
        }
    }
}

impl ShaderImportProcessor {
    /// 取一个Shader所有的 #import 后面的部分
    pub fn get_imports(&self, shader: &Shader) -> Vec<ShaderImport> {
        match &shader.source {
            Source::Wgsl(source) => self.get_imports_from_str(source),
            Source::Glsl(source, _stage) => self.get_imports_from_str(source),
            Source::SpirV(_source) => Vec::new(), // 二进制 文件 无法做预处理
        }
    }

    /// 从 string 取 #import 后面的部分
    pub fn get_imports_from_str(&self, shader: &str) -> Vec<ShaderImport> {
        let mut imports = Vec::new();

        // 扫描 每一行 匹配 #import ***，将 *** 构造 ShaderImport 扔到 Vec
        for line in shader.lines() {
            if let Some(cap) = self.import_asset_path_regex.captures(line) {
                let import = cap.get(1).unwrap();
                imports.push(ShaderImport::Path(import.as_str().to_string()));
            } else if let Some(cap) = self.import_custom_path_regex.captures(line) {
                let import = cap.get(1).unwrap();
                imports.push(ShaderImport::Custom(import.as_str().to_string()));
            }
        }

        imports
    }
}

pub static SHADER_IMPORT_PROCESSOR: Lazy<ShaderImportProcessor> =
    Lazy::new(ShaderImportProcessor::default);

/// 预处理器：#ifdef, #ifndef, #else #endif
pub struct ShaderProcessor {
    ifdef_regex: Regex,
    ifndef_regex: Regex,
    else_regex: Regex,
    endif_regex: Regex,
}

impl Default for ShaderProcessor {
    fn default() -> Self {
        Self {
            ifdef_regex: Regex::new(r"^\s*#\s*ifdef\s*([\w|\d|_]+)").unwrap(),
            ifndef_regex: Regex::new(r"^\s*#\s*ifndef\s*([\w|\d|_]+)").unwrap(),
            else_regex: Regex::new(r"^\s*#\s*else").unwrap(),
            endif_regex: Regex::new(r"^\s*#\s*endif").unwrap(),
        }
    }
}

impl ShaderProcessor {
    /// 执行 替换 文本的 预处理
    /// shader: 文本
    /// shader_defs: 预处理 宏
    /// import_shaders: 提供给 该shader找的 import 的 其他 Shader
    pub fn process(
        &self,
        id: &ShaderId,
        shader_defs: &XHashSet<String>,
        shaders: &XHashMap<ShaderId, Shader>,
        import_shaders: &XHashMap<ShaderImport, ShaderId>,
    ) -> Result<ProcessedShader, ProcessShaderError> {
        // 拿到 源码
        let shader = shaders.get(id).unwrap();
        let shader_str = match &shader.source {
            Source::Wgsl(source) => source.deref(),
            Source::Glsl(source, _stage) => source.deref(),
            Source::SpirV(source) => {
                if shader_defs.is_empty() {
                    return Ok(ProcessedShader::SpirV(source.clone()));
                } else {
                    return Err(ProcessShaderError::ShaderFormatDoesNotSupportShaderDefs);
                }
            }
        };

        let mut scopes = vec![true];
        let mut final_string = String::new();

        // 逐行处理
        for line in shader_str.lines() {
            // 遇到 #ifdef
            if let Some(cap) = self.ifdef_regex.captures(line) {
                // 取对应的 def
                let def = cap.get(1).unwrap();
                // 将 shader_defs 是否 含 该 def 的结果 加到 scopes 中
                scopes.push(*scopes.last().unwrap() && shader_defs.contains(def.as_str()));
            } else if let Some(cap) = self.ifndef_regex.captures(line) {
                // #ifndef 就将结果 取反，然后加到 scopes中
                let def = cap.get(1).unwrap();
                scopes.push(*scopes.last().unwrap() && !shader_defs.contains(def.as_str()));
            } else if self.else_regex.is_match(line) {
                // 遇到 #else
                let mut is_parent_scope_truthy = true;
                if scopes.len() > 1 {
                    is_parent_scope_truthy = scopes[scopes.len() - 2];
                }
                if let Some(last) = scopes.last_mut() {
                    *last = is_parent_scope_truthy && !*last;
                }
            } else if self.endif_regex.is_match(line) {
                // 遇到 #endif，scopes 结束
                scopes.pop();

                if scopes.is_empty() {
                    return Err(ProcessShaderError::TooManyEndIfs);
                }
            } else if let Some(cap) = SHADER_IMPORT_PROCESSOR
                .import_asset_path_regex
                .captures(line)
            {
                // 遇到 #import "..." 语句
                let import = ShaderImport::Path(cap.get(1).unwrap().as_str().to_string());

                self.apply_import(
                    id,
                    shader_defs,
                    shaders,
                    import_shaders,
                    &import,
                    &mut final_string,
                )?;
            } else if let Some(cap) = SHADER_IMPORT_PROCESSOR
                .import_custom_path_regex
                .captures(line)
            {
                // 遇到 #import ... 语句
                let import = ShaderImport::Custom(cap.get(1).unwrap().as_str().to_string());
                self.apply_import(
                    id,
                    shader_defs,
                    shaders,
                    import_shaders,
                    &import,
                    &mut final_string,
                )?;
            } else if *scopes.last().unwrap() {
                final_string.push_str(line);
                final_string.push('\n');
            }
        }

        if scopes.len() != 1 {
            return Err(ProcessShaderError::NotEnoughEndIfs);
        }

        let processed_source = Cow::from(final_string);

        match &shader.source {
            Source::Wgsl(_source) => Ok(ProcessedShader::Wgsl(processed_source)),
            Source::Glsl(_source, stage) => Ok(ProcessedShader::Glsl(processed_source, *stage)),
            Source::SpirV(_source) => {
                unreachable!("SpirV has early return");
            }
        }
    }

    // 处理 #import
    fn apply_import(
        &self,
        id: &ShaderId,
        shader_defs: &XHashSet<String>,
        shaders: &XHashMap<ShaderId, Shader>,
        import_shaders: &XHashMap<ShaderImport, ShaderId>,
        import: &ShaderImport,
        final_string: &mut String,
    ) -> Result<(), ProcessShaderError> {
        let imported_shader = import_shaders
            .get(import)
            .ok_or_else(|| ProcessShaderError::UnresolvedImport(import.clone()))?;

        let imported_processed =
            self.process(imported_shader, shader_defs, shaders, import_shaders)?;

        let shader = shaders.get(id).unwrap();

        match &shader.source {
            Source::Wgsl(_) => {
                if let ProcessedShader::Wgsl(import_source) = &imported_processed {
                    final_string.push_str(import_source);
                } else {
                    return Err(ProcessShaderError::MismatchedImportFormat(import.clone()));
                }
            }
            Source::Glsl(_, _) => {
                if let ProcessedShader::Glsl(import_source, _) = &imported_processed {
                    final_string.push_str(import_source);
                } else {
                    return Err(ProcessShaderError::MismatchedImportFormat(import.clone()));
                }
            }
            Source::SpirV(_) => {
                return Err(ProcessShaderError::ShaderFormatDoesNotSupportImports);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::rhi::shader::{ProcessShaderError, Shader, ShaderImport, ShaderProcessor};
    use naga::ShaderStage;
    use pi_hash::{XHashMap, XHashSet};

    #[rustfmt::skip]
const WGSL: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

#ifdef TEXTURE
[[group(1), binding(0)]]
var sprite_texture: texture_2d<f32>;
#endif

struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";

    const WGSL_ELSE: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

#ifdef TEXTURE
[[group(1), binding(0)]]
var sprite_texture: texture_2d<f32>;
#else
[[group(1), binding(0)]]
var sprite_texture: texture_2d_array<f32>;
#endif

struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";

    const WGSL_NESTED_IFDEF: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

# ifdef TEXTURE
# ifdef ATTRIBUTE
[[group(1), binding(0)]]
var sprite_texture: texture_2d<f32>;
# endif
# endif

struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";

    const WGSL_NESTED_IFDEF_ELSE: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

# ifdef TEXTURE
# ifdef ATTRIBUTE
[[group(1), binding(0)]]
var sprite_texture: texture_2d<f32>;
#else
[[group(1), binding(0)]]
var sprite_texture: texture_2d_array<f32>;
# endif
# endif

struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";

    #[test]
    fn process_shader_def_defined() {
        #[rustfmt::skip]
    const EXPECTED: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

[[group(1), binding(0)]]
var sprite_texture: texture_2d<f32>;

struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";
        let shader = Shader::from_wgsl(WGSL);
        let id = shader.id();
        let mut shaders = XHashMap::default();
        shaders.insert(shader.id(), shader);

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("TEXTURE".to_string());

        let processor = ShaderProcessor::default();
        let result = processor
            .process(&id, &shader_defs, &shaders, &XHashMap::default())
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_shader_def_not_defined() {
        #[rustfmt::skip]
        const EXPECTED: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;


struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";
        let shader = Shader::from_wgsl(WGSL);
        let id = shader.id();
        let mut shaders = XHashMap::default();
        shaders.insert(shader.id(), shader);

        let processor = ShaderProcessor::default();
        let result = processor
            .process(&id, &XHashSet::default(), &shaders, &XHashMap::default())
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_shader_def_else() {
        #[rustfmt::skip]
    const EXPECTED: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

[[group(1), binding(0)]]
var sprite_texture: texture_2d_array<f32>;

struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";

        let shader = Shader::from_wgsl(WGSL_ELSE);
        let id = shader.id();
        let mut shaders = XHashMap::default();
        shaders.insert(shader.id(), shader);

        let processor = ShaderProcessor::default();
        let result = processor
            .process(&id, &XHashSet::default(), &shaders, &XHashMap::default())
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_shader_def_unclosed() {
        #[rustfmt::skip]
        const INPUT: &str = r"
#ifdef FOO
";

        let shader = Shader::from_wgsl(INPUT);
        let id = shader.id();
        let mut shaders = XHashMap::default();
        shaders.insert(shader.id(), shader);

        let processor = ShaderProcessor::default();
        let result = processor.process(&id, &XHashSet::default(), &shaders, &XHashMap::default());
        assert_eq!(result, Err(ProcessShaderError::NotEnoughEndIfs));
    }

    #[test]
    fn process_shader_def_too_closed() {
        #[rustfmt::skip]
        const INPUT: &str = r"
#endif
";

        let shader = Shader::from_wgsl(INPUT);
        let id = shader.id();
        let mut shaders = XHashMap::default();
        shaders.insert(shader.id(), shader);

        let processor = ShaderProcessor::default();
        let result = processor.process(&id, &XHashSet::default(), &shaders, &XHashMap::default());
        assert_eq!(result, Err(ProcessShaderError::TooManyEndIfs));
    }

    #[test]
    fn process_shader_def_commented() {
        #[rustfmt::skip]
        const INPUT: &str = r"
// #ifdef FOO
fn foo() { }
";

        let shader = Shader::from_wgsl(INPUT);
        let id = shader.id();
        let mut shaders = XHashMap::default();
        shaders.insert(shader.id(), shader);

        let processor = ShaderProcessor::default();
        let result = processor
            .process(&id, &XHashSet::default(), &shaders, &XHashMap::default())
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), INPUT);
    }

    #[test]
    fn process_import_wgsl() {
        #[rustfmt::skip]
        const FOO: &str = r"
fn foo() { }
";
        #[rustfmt::skip]
        const INPUT: &str = r"
#import FOO
fn bar() { }
";
        #[rustfmt::skip]
        const EXPECTED: &str = r"

fn foo() { }
fn bar() { }
";

        let processor = ShaderProcessor::default();
        let mut import_shaders = XHashMap::default();
        let mut shaders = XHashMap::default();

        let import_shader = Shader::from_wgsl(FOO);
        import_shaders.insert(ShaderImport::Custom("FOO".to_string()), import_shader.id());
        shaders.insert(import_shader.id(), import_shader);

        let shader = Shader::from_wgsl(INPUT);
        let id = shader.id();
        shaders.insert(shader.id(), shader);

        let shader_defs = XHashSet::default();
        let result = processor
            .process(&id, &shader_defs, &shaders, &import_shaders)
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_import_glsl() {
        #[rustfmt::skip]
        const FOO: &str = r"
void foo() { }
";
        #[rustfmt::skip]
        const INPUT: &str = r"
#import FOO
void bar() { }
";
        #[rustfmt::skip]
        const EXPECTED: &str = r"

void foo() { }
void bar() { }
";
        let processor = ShaderProcessor::default();
        let mut import_shaders = XHashMap::default();
        let mut shaders = XHashMap::default();

        let foo = Shader::from_glsl(FOO, ShaderStage::Vertex);
        import_shaders.insert(ShaderImport::Custom("FOO".to_string()), foo.id());
        shaders.insert(foo.id(), foo);

        let shader = Shader::from_glsl(INPUT, ShaderStage::Vertex);
        let id = shader.id();
        shaders.insert(shader.id(), shader);

        let result = processor
            .process(&id, &XHashSet::default(), &shaders, &import_shaders)
            .unwrap();
        assert_eq!(result.get_glsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_nested_shader_def_outer_defined_inner_not() {
        #[rustfmt::skip]
    const EXPECTED: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;


struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";
        let mut shaders = XHashMap::default();

        let shader = Shader::from_wgsl(WGSL_NESTED_IFDEF);
        let id = shader.id();
        shaders.insert(shader.id(), shader);

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("TEXTURE".to_string());

        let processor = ShaderProcessor::default();
        let result = processor
            .process(&id, &shader_defs, &shaders, &XHashMap::default())
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_nested_shader_def_outer_defined_inner_else() {
        #[rustfmt::skip]
    const EXPECTED: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

[[group(1), binding(0)]]
var sprite_texture: texture_2d_array<f32>;

struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";
        let mut shaders = XHashMap::default();

        let shader = Shader::from_wgsl(WGSL_NESTED_IFDEF_ELSE);
        let id = shader.id();
        shaders.insert(shader.id(), shader);

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("TEXTURE".to_string());

        let processor = ShaderProcessor::default();
        let result = processor
            .process(&id, &shader_defs, &shaders, &XHashMap::default())
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_nested_shader_def_neither_defined() {
        #[rustfmt::skip]
    const EXPECTED: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;


struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";
        let mut shaders = XHashMap::default();

        let shader = Shader::from_wgsl(WGSL_NESTED_IFDEF);
        let id = shader.id();
        shaders.insert(shader.id(), shader);

        let processor = ShaderProcessor::default();
        let result = processor
            .process(&id, &XHashSet::default(), &shaders, &XHashMap::default())
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_nested_shader_def_neither_defined_else() {
        #[rustfmt::skip]
    const EXPECTED: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;


struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";
        let mut shaders = XHashMap::default();

        let shader = Shader::from_wgsl(WGSL_NESTED_IFDEF_ELSE);
        let id = shader.id();
        shaders.insert(shader.id(), shader);

        let processor = ShaderProcessor::default();
        let result = processor
            .process(&id, &XHashSet::default(), &shaders, &XHashMap::default())
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_nested_shader_def_inner_defined_outer_not() {
        #[rustfmt::skip]
    const EXPECTED: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;


struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";
        let mut shaders = XHashMap::default();

        let shader = Shader::from_wgsl(WGSL_NESTED_IFDEF);
        let id = shader.id();
        shaders.insert(shader.id(), shader);

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("ATTRIBUTE".to_string());

        let processor = ShaderProcessor::default();
        let result = processor
            .process(&id, &shader_defs, &shaders, &XHashMap::default())
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_nested_shader_def_both_defined() {
        #[rustfmt::skip]
    const EXPECTED: &str = r"
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

[[group(1), binding(0)]]
var sprite_texture: texture_2d<f32>;

struct VertexOutput {
    [[location(0)]] uv: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}
";
        let mut shader_defs = XHashSet::default();
        shader_defs.insert("TEXTURE".to_string());
        shader_defs.insert("ATTRIBUTE".to_string());

        let mut shaders = XHashMap::default();

        let shader = Shader::from_wgsl(WGSL_NESTED_IFDEF);
        let id = shader.id();
        shaders.insert(shader.id(), shader);

        let processor = ShaderProcessor::default();
        let result = processor
            .process(&id, &shader_defs, &shaders, &XHashMap::default())
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_import_ifdef() {
        #[rustfmt::skip]
        const FOO: &str = r"
#ifdef IMPORT_MISSING
fn in_import_missing() { }
#endif
#ifdef IMPORT_PRESENT
fn in_import_present() { }
#endif
";
        #[rustfmt::skip]
        const INPUT: &str = "
#import \"libs/foo\"
#ifdef MAIN_MISSING
fn in_main_missing() { }
#endif
#ifdef MAIN_PRESENT
fn in_main_present() { }
#endif
";
        #[rustfmt::skip]
        const EXPECTED: &str = r"

fn in_import_present() { }
fn in_main_present() { }
";

        let processor = ShaderProcessor::default();

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("MAIN_PRESENT".to_string());
        shader_defs.insert("IMPORT_PRESENT".to_string());

        let mut shaders = XHashMap::default();
        let shader = Shader::from_wgsl(INPUT);
        let id = shader.id();
        shaders.insert(shader.id(), shader);

        let foo = Shader::from_wgsl(FOO);
        let mut import_shaders = XHashMap::default();
        import_shaders.insert(
            ShaderImport::Path("libs/foo".to_string()),
            foo.id(),
        );
        shaders.insert(foo.id(), foo);

        let result = processor
            .process(&id, &shader_defs, &shaders, &import_shaders)
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }

    #[test]
    fn process_import_in_import() {
        #[rustfmt::skip]
        const BAR: &str = r"
#ifdef DEEP
fn inner_import() { }
#endif
";
        const FOO: &str = r"
#import BAR
fn import() { }
";
        #[rustfmt::skip]
        const INPUT: &str = r"
#import FOO
fn in_main() { }
";
        #[rustfmt::skip]
        const EXPECTED: &str = r"


fn inner_import() { }
fn import() { }
fn in_main() { }
";
        let processor = ShaderProcessor::default();

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("MAIN_PRESENT".to_string());
        shader_defs.insert("IMPORT_PRESENT".to_string());

        let mut shaders = XHashMap::default();
        let shader = Shader::from_wgsl(INPUT);
        let id = shader.id();
        shaders.insert(shader.id(), shader);

        let mut import_shaders = XHashMap::default();

        let bar = Shader::from_wgsl(BAR);
        import_shaders.insert(ShaderImport::Custom("BAR".to_string()), bar.id());
        shaders.insert(bar.id(), bar);

        let foo = Shader::from_wgsl(FOO);
        import_shaders.insert(ShaderImport::Custom("FOO".to_string()), foo.id());
        shaders.insert(foo.id(), foo);

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("DEEP".to_string());

        let result = processor
            .process(&id, &shader_defs, &shaders, &import_shaders)
            .unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }
}
