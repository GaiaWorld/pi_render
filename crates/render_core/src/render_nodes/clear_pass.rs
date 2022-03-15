use std::ops::Deref;

use crate::{
    camera::{ClearOption, Scissor, Viewport},
    render_graph::{
        node::{Node, NodeRunError, RealValue},
        node_slot::SlotInfo,
        RenderContext,
    },
    view::{render_target::RenderTarget, render_window::RenderWindow},
    Active, RenderArchetype,
};
use futures::{future::BoxFuture, FutureExt};
use pi_ecs::{
    prelude::{QueryState, With, World},
    world::ArchetypeInfo,
};
use pi_share::ShareRefCell;
use wgpu::CommandEncoder;

#[inline]
pub fn register_components(archetype: ArchetypeInfo) -> ArchetypeInfo {
    archetype
}

#[inline]
pub fn insert_resources(_world: &mut World) {}

pub struct ClearPassNode {
    window_query: ShareRefCell<QueryState<RenderArchetype, &'static RenderWindow>>,
    clear_query: ShareRefCell<
        QueryState<
            RenderArchetype,
            (
                &'static ClearOption,
                &'static RenderTarget,
                &'static Option<Viewport>,
                &'static Option<Scissor>,
            ),
            With<Active>,
        >,
    >,
}

impl ClearPassNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            window_query: ShareRefCell::new(QueryState::new(world)),
            clear_query: ShareRefCell::new(QueryState::new(world)),
        }
    }
}

impl Node for ClearPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn run(
        &self,
        context: RenderContext,
        mut commands: ShareRefCell<CommandEncoder>,
        _inputs: &[Option<RealValue>],
        _outputs: &[Option<RealValue>],
    ) -> BoxFuture<'static, Result<(), NodeRunError>> {
        let RenderContext { world, .. } = context;

        let mut clear_query = self.clear_query.clone();
        let window_query = self.window_query.clone();

        async move {
            for (clear, rt, viewport, scissor) in clear_query.iter(&world) {
                let view = match rt {
                    RenderTarget::Window(w) => {
                        let window = window_query.get(&world, w.clone()).unwrap();
                        window.rt.as_ref().unwrap()
                    }
                    RenderTarget::Texture(v) => v.deref(),
                };

                let color_attachments = match &clear.color {
                    None => vec![],
                    Some(color) => vec![wgpu::RenderPassColorAttachment {
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(color.into()),
                            store: true,
                        },
                        view,
                    }],
                };

                let depth_ops = clear.depth.map(|depth| wgpu::Operations {
                    load: wgpu::LoadOp::Clear(depth),
                    store: true,
                });

                let stencil_ops = clear.stencil.map(|stencil| wgpu::Operations {
                    load: wgpu::LoadOp::Clear(stencil),
                    store: true,
                });

                let depth_stencil_attachment = if depth_ops.is_none() && stencil_ops.is_none() {
                    None
                } else {
                    todo!();
                    
                    // Some(wgpu::RenderPassDepthStencilAttachment {
                    //     view: view,
                    //     depth_ops,
                    //     stencil_ops,
                    // })
                };

                let mut render_pass = commands.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &color_attachments,
                    depth_stencil_attachment,
                });

                if let Some(Viewport {
                    x,
                    y,
                    width,
                    height,
                    min_depth,
                    max_depth,
                }) = viewport
                {
                    render_pass.set_viewport(
                        *x as f32,
                        *y as f32,
                        *width as f32,
                        *height as f32,
                        *min_depth,
                        *max_depth,
                    );
                }

                if let Some(Scissor {
                    x,
                    y,
                    width,
                    height,
                }) = scissor
                {
                    render_pass.set_scissor_rect(*x, *y, *width, *height);
                }
            }

            Ok(())
        }
        .boxed()
    }

    fn output(&self) -> Vec<SlotInfo> {
        vec![]
    }

    fn prepare(
        &self,
        _context: RenderContext,
        _inputs: &[Option<RealValue>],
        _outputs: &[Option<RealValue>],
    ) -> Option<BoxFuture<'static, Result<(), NodeRunError>>> {
        None
    }
}
