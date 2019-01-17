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
        Value::from_any(self, |visit, v| visit.u64(*v as u64))
    }
}

impl ToValue for i8 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |visit, v| visit.i64(*v as i64))
    }
}

impl<'a> ToValue for &'a str {
    fn to_value(&self) -> Value {
        Value::from_any(self, |visit, v| visit.str(v))
    }
}
