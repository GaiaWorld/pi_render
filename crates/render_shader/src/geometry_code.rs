
pub trait GeometryCode {
    fn defines_code(&self) -> String;
    fn running_code(&self) -> String;
}
