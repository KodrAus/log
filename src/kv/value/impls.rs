use super::*;

impl<'a, T> ToValue for &'a T
where
    T: ToValue,
{
    fn to_value(&self) -> Value {
        (**self).to_value()
    }
}

impl ToValue for u8 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |v, visit| visit.u64(*v as u64))
    }
}
