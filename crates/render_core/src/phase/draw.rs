use crate::rhi::pipeline::{RenderPipeline, RenderPipelineId};

use super::TrackedRenderPass;
use pi_ecs::{
    entity::Entity,
    prelude::{
        Res, SystemParam, SystemParamFetch, SystemParamItem, SystemParamState, SystemState, World,
    },
};
use pi_ecs_macros::all_tuples;
use pi_hash::XHashMap;
use std::{any::TypeId, fmt::Debug, hash::Hash, marker::PhantomData, ops::Range};

/// A draw function which is used to draw a specific [`PhaseItem`].
///
/// They are the the general form of drawing items, whereas [`RenderCommands`](RenderCommand)
/// are more modular.
pub trait Draw<P: PhaseItem>: Send + Sync + 'static {
    /// Draws the [`PhaseItem`] by issuing draw calls via the [`TrackedRenderPass`].
    fn draw<'w>(
        &mut self,
        world: &'w World,
        pass: &mut TrackedRenderPass<'w>,
        view: Entity,
        item: &P,
    );
}

/// An item which will be drawn to the screen. A phase item should be queued up for rendering
/// during the [`RenderStage::Queue`](crate::RenderStage::Queue) stage.
/// Afterwards it will be sorted and rendered automatically  in the
/// [`RenderStage::PhaseSort`](crate::RenderStage::PhaseSort) stage and
/// [`RenderStage::Render`](crate::RenderStage::Render) stage, respectively.
pub trait PhaseItem: Send + Sync + 'static {
    /// The type used for ordering the items. The smallest values are drawn first.
    type SortKey: Ord;
    /// Determines the order in which the items are drawn during the corresponding [`RenderPhase`](super::RenderPhase).
    fn sort_key(&self) -> Self::SortKey;
    /// Specifies the [`Draw`] function used to render the item.
    fn draw_function(&self) -> DrawFunctionId;
}

// TODO: make this generic?
/// /// A [`Draw`] function identifier.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct DrawFunctionId(usize);

/// Stores all draw functions for the [`PhaseItem`] type.
/// For retrieval they are associated with their [`TypeId`].
pub struct DrawFunctions<P: PhaseItem> {
    pub draw_functions: Vec<Box<dyn Draw<P>>>,
    pub indices: XHashMap<TypeId, DrawFunctionId>,
}

impl<P: PhaseItem> DrawFunctions<P> {
    /// Adds the [`Draw`] function and associates it to its own type.
    pub fn add<T: Draw<P>>(&mut self, draw_function: T) -> DrawFunctionId {
        self.add_with::<T, T>(draw_function)
    }

    /// Adds the [`Draw`] function and associates it to the type `T`
    pub fn add_with<T: 'static, D: Draw<P>>(&mut self, draw_function: D) -> DrawFunctionId {
        self.draw_functions.push(Box::new(draw_function));
        let id = DrawFunctionId(self.draw_functions.len() - 1);
        self.indices.insert(TypeId::of::<T>(), id);
        id
    }

    /// Retrieves the [`Draw`] function corresponding to the `id` mutably.
    pub fn get_mut(&mut self, id: DrawFunctionId) -> Option<&mut dyn Draw<P>> {
        self.draw_functions.get_mut(id.0).map(|f| &mut **f)
    }

    /// Retrieves the id of the [`Draw`] function corresponding to their associated type `T`.
    pub fn get_id<T: 'static>(&self) -> Option<DrawFunctionId> {
        self.indices.get(&TypeId::of::<T>()).copied()
    }
}

/// [`RenderCommand`] is a trait that runs an ECS query and produces one or more
/// [`TrackedRenderPass`] calls. Types implementing this trait can be composed (as tuples).
///
/// They can be registered as a [`Draw`] function via the
/// [`AddRenderCommand::add_render_command`] method.
///
/// # Example
/// The `DrawPbr` draw function is created from the following render command
/// tuple.  Const generics are used to set specific bind group locations:
///
/// ```ignore
/// pub type DrawPbr = (
///     SetItemPipeline,
///     SetMeshViewBindGroup<0>,
///     SetStandardMaterialBindGroup<1>,
///     SetTransformBindGroup<2>,
///     DrawMesh,
/// );
/// ```
pub trait RenderCommand<P: PhaseItem> {
    /// Specifies all ECS data required by [`RenderCommand::render`].
    /// All parameters have to be read only.
    type Param: SystemParam;

    /// Renders the [`PhaseItem`] by issuing draw calls via the [`TrackedRenderPass`].
    fn render<'w>(
        view: Entity,
        item: &P,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult;
}

pub enum RenderCommandResult {
    Success,
    Failure,
}

pub trait EntityRenderCommand {
    type Param: SystemParam;
    fn render<'w>(
        view: Entity,
        item: Entity,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult;
}

pub trait EntityPhaseItem: PhaseItem {
    fn entity(&self) -> Entity;
}

pub trait RenderPipelinePhaseItem: PhaseItem {
    fn cached_pipeline(&self) -> RenderPipelineId;
}

/// A [`PhaseItem`] that can be batched dynamically.
///
/// Batching is an optimization that regroups multiple items in the same vertex buffer
/// to render them in a single draw call.
pub trait BatchedPhaseItem: EntityPhaseItem {
    /// Range in the vertex buffer of this item
    fn batch_range(&self) -> &Option<Range<u32>>;

    /// Range in the vertex buffer of this item
    fn batch_range_mut(&mut self) -> &mut Option<Range<u32>>;

    /// Batches another item within this item if they are compatible.
    /// Items can be batched together if they have the same entity, and consecutive ranges.
    /// If batching is successful, the `other` item should be discarded from the render pass.
    #[inline]
    fn add_to_batch(&mut self, other: &Self) -> BatchResult {
        let self_entity = self.entity();
        if let (Some(self_batch_range), Some(other_batch_range)) = (
            self.batch_range_mut().as_mut(),
            other.batch_range().as_ref(),
        ) {
            // If the items are compatible, join their range into `self`
            if self_entity == other.entity() {
                if self_batch_range.end == other_batch_range.start {
                    self_batch_range.end = other_batch_range.end;
                    return BatchResult::Success;
                } else if self_batch_range.start == other_batch_range.end {
                    self_batch_range.start = other_batch_range.start;
                    return BatchResult::Success;
                }
            }
        }
        BatchResult::IncompatibleItems
    }
}

pub enum BatchResult {
    /// The `other` item was batched into `self`
    Success,
    /// `self` and `other` cannot be batched together
    IncompatibleItems,
}

impl<P: EntityPhaseItem, E: EntityRenderCommand> RenderCommand<P> for E {
    type Param = E::Param;

    fn render<'w>(
        view: Entity,
        item: &P,
        param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        <E as EntityRenderCommand>::render(view, item.entity(), param, pass)
    }
}

pub struct SetItemPipeline;
impl<P: RenderPipelinePhaseItem> RenderCommand<P> for SetItemPipeline {
    type Param = Res<'static, RenderPipeline>;
    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: &P,
        pipeline: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let pipeline = pipeline.into_inner();
        pass.set_render_pipeline(pipeline);
        RenderCommandResult::Success
    }
}

macro_rules! render_command_tuple_impl {
    ($($name: ident),*) => {
        impl<P: PhaseItem, $($name: RenderCommand<P>),*> RenderCommand<P> for ($($name,)*) {
            type Param = ($($name::Param,)*);

            #[allow(non_snake_case)]
            fn render<'w>(
                _view: Entity,
                _item: &P,
                ($($name,)*): SystemParamItem<'w, '_, Self::Param>,
                _pass: &mut TrackedRenderPass<'w>,
            ) -> RenderCommandResult{
                $(if let RenderCommandResult::Failure = $name::render(_view, _item, $name, _pass) {
                    return RenderCommandResult::Failure;
                })*
                RenderCommandResult::Success
            }
        }
    };
}

all_tuples!(render_command_tuple_impl, 0, 15, C);

/// Wraps a [`RenderCommand`] into a state so that it can be used as a [`Draw`] function.
/// Therefore the [`RenderCommand::Param`] is queried from the ECS and passed to the command.
pub struct RenderCommandState<P: PhaseItem, C: RenderCommand<P>> {
    state: SystemState,
    fecth: <C::Param as SystemParam>::Fetch,
    p: PhantomData<P>,
    c: PhantomData<C>,
}

impl<P: PhaseItem, C: RenderCommand<P>> RenderCommandState<P, C> {
    pub fn new(world: &mut World) -> Self {
        let mut state = SystemState::new::<C::Param>();
        let config = <<C::Param as SystemParam>::Fetch>::default_config();
        let fecth = <<C::Param as SystemParam>::Fetch>::init(world, &mut state, config);

        Self {
            state,
            fecth,
            p: PhantomData::<P>,
            c: PhantomData::<C>,
        }
    }
}

impl<P: PhaseItem, C: RenderCommand<P> + Send + Sync + 'static> Draw<P>
    for RenderCommandState<P, C>
{
    /// Prepares the ECS parameters for the wrapped [`RenderCommand`] and then renders it.
    fn draw<'w>(
        &mut self,
        world: &'w World,
        pass: &mut TrackedRenderPass<'w>,
        view: Entity,
        item: &P,
    ) {
        let change_tick = world.read_change_tick();

        let param = unsafe {
            <<<C as RenderCommand<P>>::Param as SystemParam>::Fetch as SystemParamFetch::<'w, '_>>::get_param(
                &mut self.fecth,
                &self.state,
                world,
                change_tick,
            )
        };

        C::render(view, item, param, pass);
    }
}

fn add_render_command<'w, 's, P: PhaseItem, C: RenderCommand<P> + Send + Sync + 'static>(
    mut world: World,
) where
    <C::Param as SystemParam>::Fetch: SystemParamFetch<'w, 's>,
{
    let draw_function = RenderCommandState::<P, C>::new(&mut world);
    let draw_functions = world
        .get_resource_mut::<DrawFunctions<P>>()
        .unwrap_or_else(|| {
            panic!(
                "DrawFunctions<{}> must be added to the world as a resource \
                     before adding render commands to it",
                std::any::type_name::<P>(),
            );
        });
    draw_functions.add_with::<C, _>(draw_function);
}
