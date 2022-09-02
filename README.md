# pi_render

Renderer for PI Engine

## 2022.08.31

+ 删除 RenderGraphRunner （包括 Res），相关方法通过 RenderGraph 调用
+ 去掉 RenderGraph 的 O 泛型，改为 A 异步运行时
+ 去掉 extract_stage 阶段
+ 去掉 trait Node 的 prepare 操作
+ 将 Node 的 Param 改为 Input 和 Output

## prepare_***

+ build_graph
+ prepare_windows

## Res

+ RenderGraph
+ RenderInstance
+ RenderDevice
+ RenderQueue
+ RenderWindows