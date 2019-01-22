use super::*;

impl<'a, T> ToValue for &'a T
where
    T: ToValue,
{
    fn to_value(&self) -> Value {
        (**self).to_value()
    }
}

impl ToValue for () {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, _| from.none())
    }
}

impl ToValue for u8 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.u64(*v as u64))
    }
}

impl ToValue for u16 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.u64(*v as u64))
    }
}

impl ToValue for u32 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.u64(*v as u64))
    }
}

impl ToValue for u64 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.u64(*v))
    }
}

impl ToValue for i8 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.i64(*v as i64))
    }
}

impl ToValue for i16 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.i64(*v as i64))
    }
}

impl ToValue for i32 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.i64(*v as i64))
    }
}

impl ToValue for i64 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.i64(*v))
    }
}

impl ToValue for f32 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.f64(*v as f64))
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.f64(*v))
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.bool(*v))
    }
}

impl ToValue for char {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.char(*v))
    }
}

impl<T> ToValue for Option<T>
where
    T: ToValue,
{
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| match v {
            Some(ref v) => from.value(v.to_value()),
            None => from.none(),
        })
    }
}

impl<'a> ToValue for &'a str {
    fn to_value(&self) -> Value {
        Value::from_any(self, |from, v| from.str(*v))
    }
}
