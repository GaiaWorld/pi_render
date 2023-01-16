use bytemuck::NoUninit;
use derive_deref_rs::Deref;
use naga::{back::wgsl::WriterFlags, valid::ModuleInfo, Module};
use once_cell::sync::Lazy;
use pi_atom::Atom;
use pi_futures::BoxFuture;
use pi_hash::{XHashMap, XHashSet};
use pi_map::vecmap::VecMap;
use pi_share::Share;
use regex::Regex;
use std::{
    borrow::Cow,
    num::NonZeroU32,
    ops::Deref,
    path::{Path, PathBuf},
};
use thiserror::Error;
use uuid::Uuid;
use wgpu::{util::make_spirv, ShaderModuleDescriptor, ShaderSource};

/// 绑定类型，Binding类型应该实现该trait
/// 之所以没与BindLayout合并为位一个trait， 是考虑BindLayout可能手动实现，当可能不需要BindingType（这个需求不确实是否存在，如果不存在，考虑合并，TODO）
pub trait BindingType: BindLayout {
    fn binding_type() -> wgpu::BindingType;
}

/// 定义AsLayoutEntry，可以创建wgpu::BindGroupLayoutEntry
pub trait AsLayoutEntry {
    fn as_layout_entry(visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry;
}

/// 实现了BindingType的的类型，默认实现AsLayoutEntry
impl<T: BindingType> AsLayoutEntry for T {
    fn as_layout_entry(visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: Self::binding(),
            visibility,
            ty: Self::binding_type(),
            count: Self::count(), // TODO
        }
    }
}

/// BufferSize, 每个buffer类型的Binding应该实现该trait
pub trait BufferSize {
    fn min_size() -> usize;
}

/// 每个unifrom（不是指binding， 而是binding中的某个属性， 特值buffer类型的属性）
pub trait Uniform: WriteBuffer {
    type Binding: BindingType;
}

/// 每个unifrom（不是指binding， 而是binding中的某个属性， 特值buffer类型的属性）
pub trait WriteBuffer {
    // 将自身写入buffer缓冲区，假定buffer的容量足够，否则崩溃
    fn write_into(&self, index: u32, buffer: &mut [u8]);
    fn byte_len(&self) -> u32;
    fn offset(&self) -> u32;
}

/// shader的元信息描述
/// 根据该结构体，可还原出shader代码
#[derive(Debug, Clone, Default)]
pub struct ShaderMeta {
    /// binding描述
    pub bindings: ShaderBinding,
    /// 定义了Varying变量
    pub varyings: ShaderVarying,
    /// 定义了输入变量
    pub ins: ShaderInput,
    /// 定义了输出变量
    pub outs: ShaderOutput,
    /// 顶点代码片段
    pub vs: BlockCodeAtom,
    /// 像素代码片段
    pub fs: BlockCodeAtom,
}

impl ShaderMeta {
    pub fn to_code(&self, defines: &XHashSet<Atom>, visibility: wgpu::ShaderStages) -> String {
        let mut code = String::new();
        self.bindings.to_code(&mut code, defines, visibility);
        if visibility & wgpu::ShaderStages::VERTEX == wgpu::ShaderStages::VERTEX {
            self.ins.to_code(&mut code, defines);
            self.varyings.to_code(&mut code, defines, "out");
            self.vs.to_code(&mut code, defines);
        } else {
            self.varyings.to_code(&mut code, defines, "in");
            self.outs.to_code(&mut code, defines);
            self.fs.to_code(&mut code, defines);
        }

        code
    }

    pub fn add_binding_entry(
        &mut self,
        group: usize,
        value: (wgpu::BindGroupLayoutEntry, BindingExpandDescList),
    ) {
        let bindings = &mut self.bindings;
        let (bind_group_entry, buffer_uniform_expand, bingding_offset) =
            match bindings.bind_group_entrys.get_mut(group) {
                Some(r) => (
                    r,
                    &mut bindings.buffer_uniform_expands[group],
                    &mut bindings.bingding_offsets[group],
                ),
                None => {
                    bindings.bind_group_entrys.insert(group, Vec::new());
                    bindings.buffer_uniform_expands.insert(group, Vec::new());
                    bindings.bingding_offsets.insert(group, VecMap::new());
                    (
                        &mut bindings.bind_group_entrys[group],
                        &mut bindings.buffer_uniform_expands[group],
                        &mut bindings.bingding_offsets[group],
                    )
                }
            };
        let binding = value.0.binding;
        bind_group_entry.push(value.0);
        buffer_uniform_expand.push(value.1);

        bingding_offset.insert(binding as usize, bind_group_entry.len() - 1);
    }
}

#[derive(Debug, Clone, Default)]
pub struct ShaderBinding {
    /// layout entry 描述
    pub bind_group_entrys: VecMap<Vec<wgpu::BindGroupLayoutEntry>>,
    /// 除了glsl本省能描述的binding的属性外，pi_render扩展了一些其他的属性
    /// 包括： bingding名称，如果是buffer类型，还包含buffer的默认值、buffer的类型
    pub buffer_uniform_expands: VecMap<Vec<BindingExpandDescList>>,
    /// 用于索引Binding在goup的binding数组中的位置
    pub bingding_offsets: VecMap<VecMap<usize>>,
}

impl ShaderBinding {
    /// 转换为shader代码（数组， TODO）
    pub fn to_code(
        &self,
        code: &mut String,
        defines: &XHashSet<Atom>,
        visibility: wgpu::ShaderStages,
    ) {
        for (set, entrys) in self.bind_group_entrys.iter().enumerate() {
            if let Some(entrys) = entrys {
                for (binding_index, entry) in entrys.iter().enumerate() {
                    if entry.visibility & visibility == visibility {
                        let expand = &self.buffer_uniform_expands[set][binding_index];
                        if !check_defined(&expand.defines, defines) || expand.list.len() == 0 {
                            continue;
                        }

                        let set = set.to_string();
                        let binding = entry.binding.to_string();
                        code.push_str("layout(set=");
                        code.push_str(set.as_str());
                        code.push_str(",binding=");
                        code.push_str(binding.as_str());

                        match entry.ty {
                            wgpu::BindingType::Buffer { .. } => {
                                code.push_str(") uniform M_");
                                code.push_str(set.as_str());
                                code.push_str("_");
                                code.push_str(binding.as_str());
                                code.push_str("{\n");
                                for desc in expand.list.iter() {
                                    if let Some(r) = desc.buffer_expand.as_ref() {
                                        r.ty.to_code(code);
                                        code.push_str(" ");
                                        code.push_str(&desc.name);
                                        code.push_str(";\n");
                                    }
                                }
                                code.push_str("};\n");
                            }
                            wgpu::BindingType::Sampler(_) => {
                                let desc = &expand.list[0];
                                code.push_str(") sampler ");
                                code.push_str(&desc.name);
                                code.push_str(";\n");
                            }
                            wgpu::BindingType::Texture {
                                sample_type,
                                view_dimension,
                                multisampled,
                            } => {
                                let dimension_to_string = || match view_dimension {
                                    wgpu::TextureViewDimension::D1 => "1D",
                                    wgpu::TextureViewDimension::D2 => "2D",
                                    wgpu::TextureViewDimension::D2Array => "2DArray",
                                    wgpu::TextureViewDimension::Cube => "Cube",
                                    wgpu::TextureViewDimension::CubeArray => "CubeArray",
                                    wgpu::TextureViewDimension::D3 => "3D",
                                };
                                let sample_type_to_string = || match sample_type {
                                    wgpu::TextureSampleType::Float { .. } => "",
                                    wgpu::TextureSampleType::Depth => "", // TODO
                                    wgpu::TextureSampleType::Sint => "i",
                                    wgpu::TextureSampleType::Uint => "u",
                                };
                                let desc = &expand.list[0];
                                code.push_str(")");
                                code.push_str(sample_type_to_string());
                                code.push_str("texture");
                                code.push_str(dimension_to_string());
                                if multisampled {
                                    code.push_str("MS ");
                                } else {
                                    code.push_str(" ");
                                }
                                code.push_str(&desc.name);
                                // 数组，TODO
                                code.push_str(";\n");
                            }
                            // wgpu::BindingType::StorageTexture {
                            //     access,
                            //     format,
                            //     view_dimension,
                            // }
                            _ => todo!(),
                        }
                    }
                }
            }
        }
    }
}

/// shader输入
#[derive(Debug, Clone, Default)]
pub struct ShaderInput(pub Vec<InOut>);
impl ShaderInput {
    pub fn to_code(&self, code: &mut String, defines: &XHashSet<Atom>) {
        inout_to_code(code, defines, "in", &self.0);
    }
}

/// shader输出
#[derive(Debug, Clone, Default)]
pub struct ShaderOutput(pub Vec<InOut>);
impl ShaderOutput {
    pub fn to_code(&self, code: &mut String, defines: &XHashSet<Atom>) {
        inout_to_code(code, defines, "out", &self.0);
    }
}

/// shaderVarying
#[derive(Debug, Clone, Default, Deref)]
pub struct ShaderVarying(pub Vec<InOut>);

impl ShaderVarying {
    pub fn to_code(&self, code: &mut String, defines: &XHashSet<Atom>, ty: &str) {
        inout_to_code(code, defines, ty, &self.0);
    }
}

#[inline]
fn inout_to_code(code: &mut String, defines: &XHashSet<Atom>, ty: &str, list: &Vec<InOut>) {
    for in_out in list.iter() {
        in_out.to_code(code, &defines, ty);
    }
}

/// 每个binding应该实现该trait
pub trait BindLayout {
    // 在bindings的索引
    fn binding() -> u32;
    fn set() -> u32;
    // 该值为Some, 一定是数组
    fn count() -> Option<NonZeroU32>;
}

/// 描述binding的其他信息
/// 除了wgpu在创建布局时需要wgpu::BindGroupLayoutEntry信息外，shader中的binding还包含其他信息：
/// * bingding的宏开关
/// * bingding内每个属性的名称，如果是buffer类型的属性，还包含buffer的默认值、类型、数组长度（可选）
#[derive(Debug, Clone)]
pub struct BindingExpandDescList {
    pub list: Vec<BindingExpandDesc>,
    pub defines: Vec<Define>, //通过defines开关bingding
                              // storage, TODO
}

impl BindingExpandDescList {
    pub fn new(list: Vec<BindingExpandDesc>, defines: Vec<Define>) -> Self {
        Self { list, defines }
    }
}

/// binding中每个属性的描述
#[derive(Debug, Clone)]
pub struct BindingExpandDesc {
    /// 不是buffer类型的binding，该值为None
    pub buffer_expand: Option<BufferBindingExpandDesc>,
    /// bingding的名称
    pub name: Atom,
}

impl BindingExpandDesc {
    /// 创建一个buffer类型的描述
    #[inline]
    pub fn new_buffer<T: NoUninit>(
        name: &str,
        d: &[T],
        ty: TypeKind,
        size: TypeSize,
        len: ArrayLen,
    ) -> Self {
        BindingExpandDesc {
            buffer_expand: Some(BufferBindingExpandDesc {
                default_value: Vec::from(bytemuck::cast_slice::<_, u8>(d)),
                ty: BufferType { ty, size, len },
            }),
            name: Atom::from(name),
        }
    }

    /// 创建纹理类型的描述
    #[inline]
    pub fn new_texture(name: &str) -> Self {
        BindingExpandDesc {
            buffer_expand: None,
            name: Atom::from(name),
        }
    }

    /// 创建Sampler类型的描述
    #[inline]
    pub fn new_sampler(name: &str) -> Self {
        BindingExpandDesc {
            buffer_expand: None,
            name: Atom::from(name),
        }
    }
}

/// bingding中， buffer类型的属性描述
#[derive(Debug, Clone)]
pub struct BufferBindingExpandDesc {
    pub default_value: Vec<u8>,
    pub ty: BufferType,
}

/// Buffer的类型
#[derive(Debug, Clone)]
pub struct BufferType {
    pub ty: TypeKind,
    pub size: TypeSize,
    pub len: ArrayLen,
}

impl BufferType {
    pub fn to_code(&self, code: &mut String) {
        match self.size {
            TypeSize::Mat { rows, columns } => {
                if rows == columns {
                    code.push_str("mat");
                    code.push_str(rows.to_string().as_str());
                } else {
                    code.push_str("mat");
                    code.push_str(columns.to_string().as_str());
                    code.push_str("x");
                    code.push_str(rows.to_string().as_str());
                }
            }
            TypeSize::Vec(dim) => {
                match self.ty {
                    TypeKind::Float => (),
                    TypeKind::Sint => code.push_str("i"),
                    TypeKind::Uint => code.push_str("u"),
                }
                code.push_str("vec");
                code.push_str(dim.to_string().as_str());
            }
            TypeSize::Scalar => match self.ty {
                TypeKind::Float => code.push_str("float"),
                TypeKind::Sint => code.push_str("int"),
                TypeKind::Uint => code.push_str("uint"),
            },
        }
    }
}

/// kind
#[derive(Debug, Clone)]
pub enum TypeKind {
    Float,
    Sint,
    Uint,
}

#[derive(Debug, Clone)]
pub enum TypeSize {
    Mat { rows: u8, columns: u8 },
    Vec(u8),
    Scalar,
}

#[derive(Debug, Clone)]
pub enum ArrayLen {
    Constant(usize),
    Dynamic,
    None, // 不是数组
}

/// 输入输出描述
#[derive(Debug, Clone)]
pub struct InOut {
    pub location: u32,
    pub name: Atom,
    pub format: Atom,
    //#ifdef xxx
    pub defines: Vec<Define>,
}

fn check_defined(define_require: &Vec<Define>, defines: &XHashSet<Atom>) -> bool {
    if define_require.len() > 0 {
        for d in define_require.iter() {
            if defines.contains(&d.name) ^ d.is {
                return false;
            }
        }
    }
    true
}

impl InOut {
    /// * ty--in、out
    pub fn to_code(&self, code: &mut String, defines: &XHashSet<Atom>, ty: &str) {
        if !check_defined(&self.defines, defines) {
            return;
        }

        code.push_str("layout(location=");
        code.push_str(self.location.to_string().as_str());
        code.push_str(")");
        code.push_str(ty);
        code.push_str(" ");
        code.push_str(self.format.as_str());
        code.push_str(" ");
        code.push_str(self.name.as_str());
        code.push_str(";\n");
    }

    pub fn new(name: &str, format: &str, location: u32, defines: Vec<Define>) -> Self {
        Self {
            name: Atom::from(name),
            format: Atom::from(format),
            location,
            defines,
        }
    }
}

/// 宏开关
#[derive(Debug, Clone)]
pub struct Define {
    /// 为true时， 表示“name”代表的宏是否被定义
    /// 为false时， 表示“name”代表的宏是否未被定义
    pub is: bool,
    pub name: Atom,
}

impl Define {
    pub fn new(is: bool, name: Atom) -> Define {
        Define { is, name }
    }
}

/// 代码片段
#[derive(Debug, Clone, Default)]
pub struct BlockCodeAtom {
    /// 声明代码
    pub define: Vec<CodeSlice>,
    /// 运行代码
    pub running: Vec<CodeSlice>,
}
impl BlockCodeAtom {
    pub fn to_code(&self, code: &mut String, defines: &XHashSet<Atom>) {
        for c in self.define.iter() {
            c.to_code(code, defines)
        }
        code.push_str("void main(){\n");
        for c in self.running.iter() {
            c.to_code(code, defines)
        }
        code.push_str("}\n");
    }
}

/// 代码片段
#[derive(Debug, Clone, Default)]
pub struct CodeSlice {
    /// 代码
    pub code: Atom,
    /// #ifdef
    pub defines: Vec<Define>,
}

impl CodeSlice {
    pub fn to_code(&self, code: &mut String, defines: &XHashSet<Atom>) {
        if !check_defined(&self.defines, defines) {
            return;
        }
        code.push_str(self.code.as_str());
        code.push_str("\n");
    }

    pub fn push_defines_front(mut self, extends: &[Define]) -> Self {
        self.defines = merge_defines(&self.defines, extends);
        self
    }
}

// 合并defines
pub fn merge_defines(pre: &[Define], next: &[Define]) -> Vec<Define> {
    let mut r = Vec::with_capacity(pre.len() + next.len());
    r.extend_from_slice(pre);
    r.extend_from_slice(next);
    r
}

// pub struct BufferView<'a> {
//     pub buffer: &'a wgpu::Buffer,
//     pub offset: u32,
// }

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
    import: Option<ShaderImport>,
    imports: Vec<ShaderImport>,
}

impl Shader {
    /// 从 wgsl 得到的 Shader
    /// 注：wgsl 不需要指定 是哪个 stage 的 shader，因为里面有标识符指定
    pub fn from_wgsl(source: impl Into<Cow<'static, str>>) -> Shader {
        let source = source.into();
        let ShaderImports {
            imports,
            import_path,
        } = SHADER_IMPORT_PROCESSOR.get_imports_from_str(&source);
        Shader {
            id: ShaderId::new(),
            imports,
            import: import_path,
            source: Source::Wgsl(source),
        }
    }

    /// 从 glsl 得到的 Shader
    /// 需要指定 stage: vs, fs, cs
    pub fn from_glsl(source: impl Into<Cow<'static, str>>, stage: naga::ShaderStage) -> Shader {
        let source = source.into();
        let ShaderImports {
            imports,
            import_path,
        } = SHADER_IMPORT_PROCESSOR.get_imports_from_str(&source);
        Shader {
            id: ShaderId::new(),
            imports,
            import: import_path,
            source: Source::Glsl(source, stage),
        }
    }

    /// 从 spirv 二进制 得到的 Shader
    pub fn from_spirv(source: impl Into<Cow<'static, [u8]>>) -> Shader {
        Shader {
            id: ShaderId::new(),
            imports: Vec::default(),
            import: None,
            source: Source::SpirV(source.into()),
        }
    }

    pub fn id(&self) -> ShaderId {
        self.id
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    /// 设置 #import
    pub fn set_import_path<P: Into<String>>(&mut self, import_path: P) {
        self.import = Some(ShaderImport::Custom(import_path.into()));
    }

    /// 用 import-path，返回自身
    pub fn with_import_path<P: Into<String>>(mut self, import_path: P) -> Self {
        self.set_import_path(import_path);
        self
    }

    #[inline]
    pub fn import_path(&self) -> Option<&ShaderImport> {
        self.import.as_ref()
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
    bytes: Share<[u8]>,
) -> BoxFuture<'static, Result<Shader, anyhow::Error>> {
    Box::pin(async move {
        // 根据后缀名判断 是 那种类型
        let ext = path.extension().unwrap().to_str().unwrap();

        let shader = match ext {
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

        // shader.imports.import_path = Some(ShaderImport::Path(path.to_string_lossy().to_string()));

        Ok(shader)
    })
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

    #[error("load fail: {0:?}.")]
    LoadFail(PathBuf),
}

/// 预处理器：#import
pub struct ShaderImportProcessor {
    /// 路径 识别 #import "..."
    import_asset_path_regex: Regex,
    /// 定制 识别 #import ...
    import_custom_path_regex: Regex,
    /// 定制 识别 #define_import_path ...
    define_import_path_regex: Regex,
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
            import_asset_path_regex: Regex::new(r#"^\s*#\s*import\s*"([a-zA-Z0-9_:]+)""#).unwrap(),
            // #import ...
            import_custom_path_regex: Regex::new(r"^\s*#\s*import\s*([a-zA-Z0-9_:]+)").unwrap(),
            // #define_import_path ...
            define_import_path_regex: Regex::new(r"^\s*#\s*define_import_path\s+([a-zA-Z0-9_:]+)")
                .unwrap(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct ShaderImports {
    imports: Vec<ShaderImport>,
    import_path: Option<ShaderImport>,
}

impl ShaderImportProcessor {
    pub fn get_imports(&self, shader: &Shader) -> ShaderImports {
        match &shader.source {
            Source::Wgsl(source) => self.get_imports_from_str(source),
            Source::Glsl(source, _stage) => self.get_imports_from_str(source),
            Source::SpirV(_source) => ShaderImports::default(),
        }
    }

    pub fn get_imports_from_str(&self, shader: &str) -> ShaderImports {
        let mut shader_imports = ShaderImports::default();
        for line in shader.lines() {
            if let Some(cap) = self.import_asset_path_regex.captures(line) {
                let import = cap.get(1).unwrap();
                shader_imports
                    .imports
                    .push(ShaderImport::Path(import.as_str().to_string()));
            } else if let Some(cap) = self.import_custom_path_regex.captures(line) {
                let import = cap.get(1).unwrap();
                shader_imports
                    .imports
                    .push(ShaderImport::Custom(import.as_str().to_string()));
            } else if let Some(cap) = self.define_import_path_regex.captures(line) {
                let path = cap.get(1).unwrap();
                shader_imports.import_path = Some(ShaderImport::Custom(path.as_str().to_string()));
            }
        }

        shader_imports
    }
}

pub static SHADER_IMPORT_PROCESSOR: Lazy<ShaderImportProcessor> =
    Lazy::new(ShaderImportProcessor::default);

/// 预处理器：#ifdef, #ifndef, #else #endif
pub struct ShaderProcessor<L: CodeLoader> {
    ifdef_regex: Regex,
    ifndef_regex: Regex,
    else_regex: Regex,
    endif_regex: Regex,
    default_value_regex: Regex,
    loader: L,
}

impl Default for ShaderProcessor<CodeLoaderEmptyImpl> {
    fn default() -> Self {
        Self {
            ifdef_regex: Regex::new(r"^\s*#\s*ifdef\s*([\w|\d|_]+)").unwrap(),
            ifndef_regex: Regex::new(r"^\s*#\s*ifndef\s*([\w|\d|_]+)").unwrap(),
            else_regex: Regex::new(r"^\s*#\s*else").unwrap(),
            endif_regex: Regex::new(r"^\s*#\s*endif").unwrap(),
            default_value_regex: Regex::new(r#"^\s*@default\s*\(\s*([0-9.,\s]+)\)"#).unwrap(),
            loader: CodeLoaderEmptyImpl,
        }
    }
}

impl<L: CodeLoader> ShaderProcessor<L> {
    pub fn new(l: L) -> Self {
        Self {
            ifdef_regex: Regex::new(r"^\s*#\s*ifdef\s*([\w|\d|_]+)").unwrap(),
            ifndef_regex: Regex::new(r"^\s*#\s*ifndef\s*([\w|\d|_]+)").unwrap(),
            else_regex: Regex::new(r"^\s*#\s*else").unwrap(),
            endif_regex: Regex::new(r"^\s*#\s*endif").unwrap(),
            default_value_regex: Regex::new(r#"^\s*@default\s*\(\s*([0-9.,\s]+)\)"#).unwrap(),
            loader: l,
        }
    }
}

pub trait Defineds {
    fn contains(&self, value: &str) -> bool;
    fn is_empty(&self) -> bool;
}

impl Defineds for XHashSet<String> {
    fn contains(&self, value: &str) -> bool {
        self.contains(value)
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

/// 开启所有Defineds
pub struct AllDefineds;
impl Defineds for AllDefineds {
    fn contains(&self, _value: &str) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        false
    }
}

pub trait CodeLoader {
    fn load(&self, path: &PathBuf) -> Result<Vec<u8>, ProcessShaderError>;
}

pub struct CodeLoaderEmptyImpl;
impl CodeLoader for CodeLoaderEmptyImpl {
    fn load(&self, path: &PathBuf) -> Result<Vec<u8>, ProcessShaderError> {
        Err(ProcessShaderError::LoadFail(path.clone()))
    }
}

#[derive(Default)]
pub struct ShaderCodeMgr {
    pub shaders: XHashMap<ShaderId, (Shader, Option<String>)>,
    pub import_shaders: XHashMap<ShaderImport, ShaderId>,
}

impl<L: CodeLoader> ShaderProcessor<L> {
    /// 执行 替换 文本的 预处理
    /// shader: 文本
    /// shader_defs: 预处理 宏
    /// import_shaders: 提供给 该shader找的 import 的 其他 Shader
    pub fn process<D: Defineds>(
        &self,
        id: &ShaderId,
        shader_defs: &D,
        shader_mgr: &mut ShaderCodeMgr,
    ) -> Result<ProcessedShader, ProcessShaderError> {
        let (shader, _path) = shader_mgr.shaders.get(id).unwrap().clone();
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
                let import_path = cap.get(1).unwrap().as_str();
                // let import_path = match &path {
                //     Some(p) => import_path.to_string(),
                //     None => cap.get(1).unwrap().as_str().to_string(),
                // };
                let import = ShaderImport::Path(import_path.to_string());

                self.apply_import(id, shader_defs, shader_mgr, &import, &mut final_string)?;
            } else if let Some(cap) = SHADER_IMPORT_PROCESSOR
                .import_custom_path_regex
                .captures(line)
            {
                // 遇到 #import ... 语句
                let import = ShaderImport::Custom(cap.get(1).unwrap().as_str().to_string());
                self.apply_import(id, shader_defs, shader_mgr, &import, &mut final_string)?;
            } else if SHADER_IMPORT_PROCESSOR
                .define_import_path_regex
                .is_match(line)
                || self.default_value_regex.is_match(line)
            {
                // ignore import path lines
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
    pub fn apply_import<D: Defineds>(
        &self,
        id: &ShaderId,
        shader_defs: &D,
        shader_mgr: &mut ShaderCodeMgr,
        import: &ShaderImport,
        final_string: &mut String,
    ) -> Result<(), ProcessShaderError> {
        let imported_shader = match shader_mgr.import_shaders.get(import) {
            Some(r) => r.clone(),
            None => {
                if let ShaderImport::Path(p) = import {
                    let path = Path::new(p);
                    let suffix = path
                        .extension()
                        .ok_or_else(|| ProcessShaderError::UnresolvedImport(import.clone()))?;
                    let path = PathBuf::from(path);
                    let suffix = suffix.to_str().unwrap();
                    let code = self.loader.load(&path)?;
                    let shader = match suffix {
                        "spv" => Shader::from_spirv(code),
                        "wgsl" => Shader::from_wgsl(String::from_utf8(code).unwrap()),
                        // NOTE: naga::ShaderStage::Vertex 随便写的，好像没有影响？
                        "glsl" => Shader::from_glsl(
                            String::from_utf8(code).unwrap(),
                            naga::ShaderStage::Vertex,
                        ),
                        _ => return Err(ProcessShaderError::LoadFail(path)),
                    };
                    let id = shader.id();
                    shader_mgr.shaders.insert(id, (shader, Some(p.clone())));
                    shader_mgr.import_shaders.insert(import.clone(), id);
                    id
                } else {
                    return Err(ProcessShaderError::UnresolvedImport(import.clone()));
                }
            }
        };
        // let imported_shader = shader_mgr.import_shaders
        //     .get(import)
        // 	.map(|r| {r.clone()})
        //     .ok_or_else(|| {

        // 	})?;
        let imported_processed = self.process(&imported_shader, shader_defs, shader_mgr)?;

        let (shader, _path) = shader_mgr.shaders.get(id).unwrap();

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

// fn relative_path(mut file_path: &str, mut dir: &str) -> String {
//     let (file_path_len, dir_len) = (file_path.len(), dir.len());
//     if file_path_len == 0 {
//         return "".to_string();
//     }
//     // 不以 . 开头，就是绝对路径，直接返回
//     // 目录为空字符串，直接返回
//     if &file_path[0..1] != "." || dir.len() == 0 {
//         return file_path.to_string();
//     }

//     let (mut i, mut j) = (0, dir_len as isize - 1);

//     // 最后一个字符不是/，就代表dir不是目录，需要定位到目录
//     if j >= 0 && &dir[j as usize..dir_len] != "/" {
//         j = dir.rfind("/").map_or(-1, |r| r as isize);
//     }

//     while i < file_path_len {
//         if &file_path[i..i + 1] != "." {
//             break;
//         }
//         if let Some(r) = file_path.get(i + 1..i + 2) {
//             // ./的情况
//             if r == "/" {
//                 i += 2;
//                 break;
//             }
//         }

//         if let Some(r) = file_path.get(i + 1..i + 3) {
//             // ./的情况
//             if r != "./" {
//                 break;
//             }
//         }
//         // ../的情况
//         i += 3;

//         if j > 0 {
//             j = dir[0..j as usize].rfind("/").map_or(-1, |r| r as isize);
//         } else {
//             j = -1;
//         }
//     }

//     if i > 0 {
//         file_path = &file_path[i..file_path_len];
//     };

//     if j < 0 {
//         return file_path.to_string();
//     }

//     if j < dir_len as isize - 1 {
//         dir = &dir[0..(j + 1) as usize];
//     }

//     return dir.to_string() + file_path;
// }

#[cfg(test)]
mod tests {
    use crate::rhi::shader::{
        ProcessShaderError, Shader, ShaderCodeMgr, ShaderImport, ShaderProcessor,
    };
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
        shaders.insert(shader.id(), (shader, None));

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("TEXTURE".to_string());

        let processor = ShaderProcessor::default();
        let result = processor
            .process(
                &id,
                &shader_defs,
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders: XHashMap::default(),
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let processor = ShaderProcessor::default();
        let result = processor
            .process(
                &id,
                &XHashSet::default(),
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders: XHashMap::default(),
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let processor = ShaderProcessor::default();
        let result = processor
            .process(
                &id,
                &XHashSet::default(),
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders: XHashMap::default(),
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let processor = ShaderProcessor::default();
        let result = processor.process(
            &id,
            &XHashSet::default(),
            &mut ShaderCodeMgr {
                shaders,
                import_shaders: XHashMap::default(),
            },
        );
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
        shaders.insert(shader.id(), (shader, None));

        let processor = ShaderProcessor::default();
        let result = processor.process(
            &id,
            &XHashSet::default(),
            &mut ShaderCodeMgr {
                shaders,
                import_shaders: XHashMap::default(),
            },
        );
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
        shaders.insert(shader.id(), (shader, None));

        let processor = ShaderProcessor::default();
        let result = processor
            .process(
                &id,
                &XHashSet::default(),
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders: XHashMap::default(),
                },
            )
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
        shaders.insert(import_shader.id(), (import_shader, None));

        let shader = Shader::from_wgsl(INPUT);
        let id = shader.id();
        shaders.insert(shader.id(), (shader, None));

        let shader_defs = XHashSet::default();
        let result = processor
            .process(
                &id,
                &shader_defs,
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders,
                },
            )
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
        shaders.insert(foo.id(), (foo, None));

        let shader = Shader::from_glsl(INPUT, ShaderStage::Vertex);
        let id = shader.id();
        shaders.insert(shader.id(), (shader, None));

        let result = processor
            .process(
                &id,
                &XHashSet::default(),
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders,
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("TEXTURE".to_string());

        let processor = ShaderProcessor::default();
        let result = processor
            .process(
                &id,
                &shader_defs,
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders: XHashMap::default(),
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("TEXTURE".to_string());

        let processor = ShaderProcessor::default();
        let result = processor
            .process(
                &id,
                &shader_defs,
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders: XHashMap::default(),
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let processor = ShaderProcessor::default();
        let result = processor
            .process(
                &id,
                &XHashSet::default(),
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders: XHashMap::default(),
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let processor = ShaderProcessor::default();
        let result = processor
            .process(
                &id,
                &XHashSet::default(),
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders: XHashMap::default(),
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("ATTRIBUTE".to_string());

        let processor = ShaderProcessor::default();
        let result = processor
            .process(
                &id,
                &shader_defs,
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders: XHashMap::default(),
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let processor = ShaderProcessor::default();
        let result = processor
            .process(
                &id,
                &shader_defs,
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders: XHashMap::default(),
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let foo = Shader::from_wgsl(FOO);
        let mut import_shaders = XHashMap::default();
        import_shaders.insert(ShaderImport::Path("libs/foo".to_string()), foo.id());
        shaders.insert(foo.id(), (foo, None));

        let result = processor
            .process(
                &id,
                &shader_defs,
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders,
                },
            )
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
        shaders.insert(shader.id(), (shader, None));

        let mut import_shaders = XHashMap::default();

        let bar = Shader::from_wgsl(BAR);
        import_shaders.insert(ShaderImport::Custom("BAR".to_string()), bar.id());
        shaders.insert(bar.id(), (bar, None));

        let foo = Shader::from_wgsl(FOO);
        import_shaders.insert(ShaderImport::Custom("FOO".to_string()), foo.id());
        shaders.insert(foo.id(), (foo, None));

        let mut shader_defs = XHashSet::default();
        shader_defs.insert("DEEP".to_string());

        let result = processor
            .process(
                &id,
                &shader_defs,
                &mut ShaderCodeMgr {
                    shaders,
                    import_shaders,
                },
            )
            .unwrap();
        let _r = result.get_wgsl_source().unwrap();
        assert_eq!(result.get_wgsl_source().unwrap(), EXPECTED);
    }
}

// #[test]
// fn test_relative_path() {
//     println!("{:?}", relative_path("./aa", "xx/src"));
//     println!("{:?}", relative_path("../aa", "xx/src"));
//     println!("{:?}", relative_path("../../aa", "xx/src/src1/src2"));
// }
