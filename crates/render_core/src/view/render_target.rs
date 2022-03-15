use crate::rhi::texture::TextureView;
use pi_ecs::entity::Entity;
use pi_share::ShareRefCell;

#[derive(Clone)]
pub enum RenderTarget {
    /// RenderWindow 的 Entity
    Window(Entity),
    Texture(ShareRefCell<TextureView>),
}

impl RenderTarget {
    pub fn new_with_window(entity: Entity) -> Self {
        Self::Window(entity)
    }

    pub fn new_with_texture(view: ShareRefCell<TextureView>) -> Self {
        Self::Texture(view)
    }
}
