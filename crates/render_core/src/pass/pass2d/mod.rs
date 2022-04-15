//! Pass2D Entity

pub mod target;
pub mod draw_object;
pub mod camera;

use crate::rhi::{CommandEncoder, RenderPassDescriptor};
use self::{target::{RenderTargetKey, RenderTargets}, camera::Camera2D, draw_object::DrawState};
use pi_ecs::{prelude::{World, Query, Res}, entity::Entity};

/// Pass2D 原型，描述 将 2D物件 渲染到 指定 渲染目标的流程
/// 用指定 Camera2D 的 视图矩阵 和 投影矩阵
/// 输出到 RenderTargetKey 指定的 渲染目标
/// 挨个 渲染 不透明物件 Opaque2D
/// 挨个 渲染 半透明物件 Transparent2D
pub struct Pass2DArchetype;

// 渲染 物件 列表
pub struct Draw2DList {
    /// 不透明 列表
    /// 注：渲染时，假设 Vec已经 排好序 了
    // Entity 是 DrawObjectArchetype
    pub opaque: Vec<Entity>,

    /// 透明 列表
    /// 注：渲染时，假设 Vec已经 排好序 了
    /// Entity 是 DrawObjectArchetype
    pub transparent: Vec<Entity>,
}

impl Default for Draw2DList {
    fn default() -> Self {
        Self { 
            opaque: Vec::default(), 
            transparent: Vec::default(),
        }
    }
}

/// 初始化 ECS
pub fn init_ecs(world: &mut World) {
    world
        .new_archetype::<Pass2DArchetype>()
        .register::<Camera2D>()
        .register::<RenderTargetKey>()
        .register::<Draw2DList>()
        .create();
}

/// 由 渲染节点 调用
/// 渲染 对应 一组 2D物件 到 指定 目标
fn draw_2d(
    world: World,
    encoder: &mut CommandEncoder,
    rts: Res<RenderTargets>,
    q: Query<Pass2DArchetype, (&Camera2D, &RenderTargetKey, &Draw2DList)>
) {
    for (camera, rt_key, list) in q.iter() {
        let view = camera.get_view();
        let proj = camera.get_projection();
        
        let rt = rts.get(*rt_key).unwrap();
        
        let Draw2DList {opaque, transparent} = list;

        let rp = encoder.begin_render_pass(&RenderPassDescriptor {
            label: todo!(),
            color_attachments: todo!(),
            depth_stencil_attachment: todo!(),
        });

        // 先渲染不透明 列表
        for e in opaque {
            // 每个 e 的 原型都是 DrawObjectArchetype
        }

        // 后渲染 透明 列表
        for e in transparent {
            // 每个 e 的 原型都是 DrawObjectArchetype
            // ? 怎么 从 e 取到 DrawState 组件
            let state: world .get<>&DrawState = Query::get<DrawObjectArchetype, DrawState>(e);
            state.draw();
        }
    }
}