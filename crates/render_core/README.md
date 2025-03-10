# Render Core

[![Crates.io](https://img.shields.io/crates/v/render_core)](https://crates.io/crates/render_core)
[![Docs.rs](https://docs.rs/render_core/badge.svg)](https://docs.rs/render_core)

Render Core 是高性能渲染引擎的核心抽象层，提供跨图形后端的通用渲染基础设施。

## 功能特性

- **跨后端抽象**：支持Vulkan/Metal/DX12等现代图形API的通用抽象
- **资源管理**：
  - 自动化的Buffer/Texture内存分配（BlockAlloc/BufferAlloc）
  - 动态Uniform Buffer管理
  - 纹理资源生命周期管理
- **依赖图系统**：
  - 基于节点的渲染管线编排
  - 自动化的资源依赖追踪
  - 多线程友好的任务调度
- **组件系统**：
  - 视图目标分配管理
  - 调试工具支持
  - 字体渲染基础设施
- **Shader管理**：
  - 结构化Shader元数据
  - 自动化的BindGroup生成
  - 多后端SPIR-V支持

## 核心模块

### `depend_graph`
- **graph**：渲染依赖图的核心实现
- **node**：可组合的渲染节点系统
- **param**：节点参数传递机制
- **sub_graph**：子图嵌套支持

### `rhi` (Rendering Hardware Interface)
- **buffer_alloc**：高效的内存块分配器
- **dyn_uniform_buffer**：动态Uniform Buffer管理
- **texture**：多格式纹理资源管理
- **bind_group**：自动化的资源绑定管理

### `components`
- **view**：渲染目标视图管理
- **target_alloc**：多目标渲染支持
- **font**：基于SDF的字体渲染

### `renderer`
- **texture**：纹理加载与处理管线
- **draw_obj**：可扩展的绘制对象系统

## 开发指南

### 安转
```bash
[dependencies]
render_core = "0.1"
```

## 贡献

欢迎通过Issue和PR参与贡献，请遵循项目的代码风格规范。

## 许可证

MIT License © 2024 Pi Engine Team
