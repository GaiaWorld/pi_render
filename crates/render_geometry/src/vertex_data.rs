use render_data_container::{TVertexDataKindKey, TVertexBufferID};

#[derive(Debug, Clone, Copy)]
pub enum EVertexDataKind {
    Position,
    Position2D,
    ColorKind,
    UV,
    Normal,
    Tangent,
    MatricesIndicesKind,
    MatricesWeightsKind,
    MatricesIndicesExtraKind,
    MatricesWeightsExtraKind,
    UV2,
    UV3,
    UV4,
    UV5,
    UV6,
    UV7,
    UV8,
    UV9,
    UV10,
    UV11,
    UV12,
    UV13,
    UV14,
    UV15,
    UV16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VertexAttributeDesc<VDK: TVertexDataKindKey> {
    kind: VDK,
    location: usize,
}

#[derive(Debug, Clone)]
pub struct VertexBufferU8<GBID: TVertexBufferID> {
    pub data: GBID,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct VertexBufferU16<GBID: TVertexBufferID> {
    pub data: GBID,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct VertexBufferU32<GBID: TVertexBufferID> {
    pub data: GBID,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct VertexBufferF32<GBID: TVertexBufferID> {
    pub data: GBID,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct VertexBufferF64<GBID: TVertexBufferID> {
    pub data: GBID,
    pub offset: u32,
    pub size: u32,
}