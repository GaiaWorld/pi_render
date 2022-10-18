# 材质

## Uniform 处理

### 一般数值 Uniform
* set = 0
  * 具体shader渲染参数
    * binding = 0 - 每drawCall信息
  * 渲染时自动设置
    * binding = 1 - 每drawCall信息
      * 包含
        * mat4 PI_ObjectToWorld
        * mat4 PI_WorldToObject
        * ...
  * 在一个RenderPass中公用的
    * binding = 2 - 整个场景的基础数据
      * 包含
        * mat4 PI_MATRIX_V
        * mat4 PI_MATRIX_P
        * mat4 PI_MATRIX_VP
        * vec4 PI_Time
        * vec4 PI_DeltaTime
        * vec4 PI_Screen
    * binding = 3 - 整个场景的效果类
      * 包含
        * vec4 PI_FogParams
        * vec4 PI_FogColor
    * binding = 4 - 整个场景的效果类
      * 包含
        * vec4 PI_FogParams
        * vec4 PI_FogColor
  * ...

### 纹理类

* set = 1 - 每 draw 一个
  * 第 N 个纹理
    * Sampler - binding = N * 2 + 0
    * TextureView - binding = N * 2 + 1

* set = 2 - 场景管理的纹理
  * Relection
  * BRDF
  * Environment
  * Shadow
  * Shadow

### 数组类 - 只能有一个binding - 待定

* set = 3 - 每 draw 一个
  * N 个数组
    * 每个数组内
      * lightColor
      * lightPosition
      * lightDirection
      * lightParams

## 材质

### 功能

* 记录 bindgroup

### draw

* 

## 材质属性块

### 功能

* 内部记录 TMaterialBlockKindKey 对应 要修改的 内存buffer 及 修改偏移

### 

### 材质属性修改接口

* modify(&mut self, TMaterialBlockKindKey, UniformData)