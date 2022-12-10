
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ESkinCode {
    None,
    Normal
}
impl ESkinCode {
    pub fn vs_begin_code(&self) -> String {
        match self {
            ESkinCode::None => String::from(""),
            ESkinCode::Normal => Self::normal(),
        }
    }
    fn normal() -> String {
        String::from("
        mat4 influence;
        influence = readMatrixFromRawSampler(boneSampler, matricesIndices[0])*matricesWeights[0];
        influence += readMatrixFromRawSampler(boneSampler, matricesIndices[1])*matricesWeights[1];
        influence += readMatrixFromRawSampler(boneSampler, matricesIndices[2])*matricesWeights[2];
        influence += readMatrixFromRawSampler(boneSampler, matricesIndices[3])*matricesWeights[3];
        PI_ObjectToWorld = PI_ObjectToWorld * influence; 
        ")
    }
}