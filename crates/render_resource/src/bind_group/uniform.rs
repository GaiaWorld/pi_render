use super::bind::TBindValue;

pub trait TUniformValue {
    fn write_data<T: TBindValue>(&self, bind: &T);
}