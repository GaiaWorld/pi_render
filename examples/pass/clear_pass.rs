use futures::{future::BoxFuture, FutureExt};
use pi_ecs::prelude::World;
use pi_render::{
    components::view::target::{RenderTarget, RenderTargetKey, RenderTargets, TextureViews},
    graph::{
        node::{Node, NodeRunError},
        RenderContext,
    },
};
use pi_share::ShareRefCell;
use pi_slotmap::{new_key_type, SlotMap};
use wgpu::CommandEncoder;

new_key_type! {
    pub struct ClearOptionKey;
}

#[derive(Debug, Default)]
pub struct ClearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ClearColor {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<&ClearColor> for wgpu::Color {
    fn from(clear: &ClearColor) -> Self {
        Self {
            r: clear.r as f64,
            g: clear.g as f64,
            b: clear.b as f64,
            a: clear.a as f64,
        }
    }
}

pub type ClearOptions = SlotMap<ClearOptionKey, ClearOption>;

#[derive(Default)]
pub struct ClearOption {
    target: RenderTargetKey,
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
}

#[inline]
pub fn insert_resources(world: &mut World) {
    world.insert_resource(ClearOptions::default());
}

#[derive(Default)]
pub struct ClearPassNode;

impl ClearPassNode {}

impl Node for ClearPassNode {
    type Output = ();

    fn run<'a>(
        &'a self,
        context: RenderContext,
        commands: ShareRefCell<CommandEncoder>,
        _inputs: &'a [()],
    ) -> BoxFuture<'a, Result<Self::Output, NodeRunError>> {
        let RenderContext { world, .. } = context;
        async move {
            let clears = world.get_resource::<ClearOptions>().unwrap();
            let rts = world.get_resource::<RenderTargets>().unwrap();
            let views = world.get_resource::<TextureViews>().unwrap();

            for (_, ClearOption { target, color, .. }) in clears.iter() {
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
                // let depth_stencil_attachment = None;

                {
                    // let render_pass = commands.0.borrow_mut().begin_render_pass(&wgpu::RenderPassDescriptor {
                    //     label: None,
                    //     color_attachments: &color_attachments,
                    //     depth_stencil_attachment,
                    // });
                }
                
            }
            Ok(())
        }
        .boxed()
    }

    fn prepare<'a>(
        &'a self,
        _context: RenderContext,
    ) -> Option<BoxFuture<'a, Result<(), NodeRunError>>> {
        None
    }

    fn finish<'a>(
        &'a self,
        _context: RenderContext,
        _inputs: &'a [Self::Output],
    ) -> BoxFuture<'a, Result<(), NodeRunError>> {
        async { Ok(()) }.boxed()
    }
}
