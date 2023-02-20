

use crate::{skin_code::ESkinCode};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyShaderModelAbout {
    pub skin: ESkinCode,
}