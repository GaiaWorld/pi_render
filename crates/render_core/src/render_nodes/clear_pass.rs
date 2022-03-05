//! 清屏

use super::clear_color::{ClearColor, RenderTargetClearColors};
use crate::{
    camera::{render_target::RenderTarget, RenderCamera},
    render_graph::{
        node::{Node, NodeRunError, RealValue},
        node_slot::SlotInfo,
        runner::CommandEncoderWrap,
        RenderContext,
    },
    rhi::{
        LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
        RenderPassDescriptor,
    },
    view::{window::RenderWindows, RenderView, ViewDepthTexture, ViewTarget},
    RenderArchetype,
};
use futures::{future::BoxFuture, FutureExt};
use pi_ecs::prelude::{QueryState, With, World};
use pi_hash::XHashSet;
use pi_share::{cell::TrustCell, Share};
use std::{ops::DerefMut, sync::Arc};

pub struct ClearPassNode {
    query: Share<
        TrustCell<
            QueryState<
                RenderArchetype,
                (
                    &'static ViewTarget,
                    Option<&'static ViewDepthTexture>,
                    Option<&'static RenderCamera>,
                ),
                With<RenderView>,
            >,
        >,
    >,
}

impl ClearPassNode {
    pub fn new(world: &mut World) -> Self {
        Self {
            query: Share::new(TrustCell::new(QueryState::new(world))),
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
        commands: CommandEncoderWrap,
        _inputs: &[Option<RealValue>],
        _outputs: &[Option<RealValue>],
    ) -> BoxFuture<'static, Result<(), NodeRunError>> {
        let RenderContext { world, .. } = context;

        let query = self.query.clone();

        async move {
            let mut cleared_targets = XHashSet::default();
            let clear_color = world.get_resource::<ClearColor>().unwrap();
            let render_target_clear_colors =
                world.get_resource::<RenderTargetClearColors>().unwrap();

            let c = &mut commands.0.borrow_mut();
            let c = c.deref_mut();
            let c = c.as_mut().unwrap();

            // This gets all ViewTargets and ViewDepthTextures and clears its attachments
            // TODO: This has the potential to clear the same target multiple times, if there
            // are multiple views drawing to the same target. This should be fixed when we make
            // clearing happen on "render targets" instead of "views" (see the TODO below for more context).
            for (target, depth, camera) in query.borrow_mut().iter(&world) {
                let mut color = &clear_color.0;

                if let Some(camera) = camera {
                    cleared_targets.insert(&camera.target);
                    if let Some(target_color) = render_target_clear_colors.get(&camera.target) {
                        color = target_color;
                    }
                }
                let pass_descriptor = RenderPassDescriptor {
                    label: Some("clear_pass"),
                    color_attachments: &[target.get_color_attachment(Operations {
                        load: LoadOp::Clear((*color).into()),
                        store: true,
                    })],
                    depth_stencil_attachment: depth.map(|depth| RenderPassDepthStencilAttachment {
                        view: &depth.view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(0.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                };

                c.begin_render_pass(&pass_descriptor);
            }

            // TODO: This is a hack to ensure we don't call present() on frames without any work,
            // which will cause panics. The real fix here is to clear "render targets" directly
            // instead of "views". This should be removed once full RenderTargets are implemented.
            let windows = world.get_resource::<RenderWindows>().unwrap();
            for target in render_target_clear_colors.colors.keys().cloned().chain(
                windows
                    .values()
                    .map(|window| RenderTarget::Window(window.id)),
            ) {
                // skip windows that have already been cleared
                if cleared_targets.contains(&target) {
                    continue;
                }
                let pass_descriptor = RenderPassDescriptor {
                    label: Some("clear_pass"),
                    color_attachments: &[RenderPassColorAttachment {
                        view: target.get_texture_view(windows).unwrap(),
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(
                                (*render_target_clear_colors
                                    .get(&target)
                                    .unwrap_or(&clear_color.0))
                                .into(),
                            ),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                };
                c.begin_render_pass(&pass_descriptor);
            }

            Ok(())
        }
        .boxed()
    }

    fn is_finish(&self) -> bool {
        false
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
