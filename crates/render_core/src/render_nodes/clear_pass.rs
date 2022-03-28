use crate::{
    components::camera::{
        render_target::{RenderTarget, RenderTargetKey, RenderTargets, TextureViews},
        ClearColor,
    },
    components::camera::{Scissor, Viewport},
    render_graph::{
        node::{Node, NodeRunError, RealValue},
        node_slot::SlotInfo,
        RenderContext,
    },
};
use futures::{future::BoxFuture, FutureExt};
use log::{debug, info};
use pi_ecs::prelude::World;
use pi_share::ShareRefCell;
use pi_slotmap::{new_key_type, SlotMap};
use wgpu::CommandEncoder;

new_key_type! {
    pub struct ClearOptionKey;
}

pub type ClearOptions = SlotMap<ClearOptionKey, ClearOption>;

#[derive(Default)]
pub struct ClearOption {
    target: RenderTargetKey,
    pub viewport: Option<Viewport>,
    pub scissor: Option<Scissor>,
    pub color: Option<ClearColor>,
    pub depth: Option<f32>,
    pub stencil: Option<u32>,
}

impl ClearOption {
    pub fn set_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
        self.color = Some(ClearColor { r, g, b, a });
    }

    pub fn set_depth(&mut self, depth: Option<f32>) {
        self.depth = depth;
    }

    pub fn set_stencil(&mut self, stencil: Option<u32>) {
        self.stencil = stencil;
    }

    pub fn set_target(&mut self, target: RenderTargetKey) {
        self.target = target;
    }

    pub fn set_viewport(&mut self, viewport: Option<Viewport>) {
        self.viewport = viewport;
    }

    pub fn set_scissor(&mut self, scissor: Option<Scissor>) {
        self.scissor = scissor;
    }
}

#[inline]
pub fn insert_resources(world: &mut World) {
    world.insert_resource(ClearOptions::default());
}

#[derive(Default)]
pub struct ClearPassNode;

impl ClearPassNode {}

impl Node for ClearPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![]
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

    fn run(
        &self,
        context: RenderContext,
        mut commands: ShareRefCell<CommandEncoder>,
        _inputs: &[Option<RealValue>],
        _outputs: &[Option<RealValue>],
    ) -> BoxFuture<'static, Result<(), NodeRunError>> {
        let RenderContext { world, .. } = context;
        async move {
            let clears = world.get_resource::<ClearOptions>().unwrap();
            let rts = world.get_resource::<RenderTargets>().unwrap();
            let views = world.get_resource::<TextureViews>().unwrap();

            for (
                _,
                ClearOption {
                    target,
                    viewport,
                    scissor,
                    color,
                    ..
                },
            ) in clears.iter()
            {
                if color.is_none() {
                    continue;
                }

                let color = color.as_ref().unwrap();
                let RenderTarget { colors, .. } = rts.get(*target).unwrap();
                let color_attachments = colors
                    .iter()
                    .map(|view| {
                        let view = views.get(*view).unwrap();
                        wgpu::RenderPassColorAttachment {
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(color.into()),
                                store: true,
                            },
                            view: view.as_ref().unwrap(),
                        }
                    })
                    .collect::<Vec<wgpu::RenderPassColorAttachment>>();

                // TODO Detph-Stencil
                let depth_stencil_attachment = None;

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
}
