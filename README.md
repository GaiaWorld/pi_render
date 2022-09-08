# pi_render

Renderer for PI Engine

## 2022.08.31

+ 删除 RenderGraphRunner （包括 Res），相关方法通过 RenderGraph 调用
+ 去掉 RenderGraph 的 O 泛型，输入输出 对 RenderGraph 隐藏；
+ 去掉 extract_stage 阶段
+ 去掉 trait Node 的 prepare 操作
+ 将 Node 的 Param 改为 Input 和 Output，可用 下面两个宏 自动展开
    - 派生宏 #[derive(NodeParam, Clone, Debug, Default)]
        * 对应的 属性宏 #[field_slots]

## prepare_***

+ build_graph
+ prepare_windows

## Res

+ RenderGraph
+ RenderInstance
+ RenderDevice
+ RenderQueue
+ RenderWindows