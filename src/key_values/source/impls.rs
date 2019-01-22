use super::{Source, ToKey, ToValue, Visitor, Value, Error};

impl<'a, T: ?Sized> Source for &'a T
where
    T: Source,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        (*self).visit(visitor)
    }

    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<Value<'kvs>>
    where
        Q: ToKey,
    {
        (*self).get(key)
    }
}

impl<K, V> Source for (K, V)
where
    K: ToKey,
    V: ToValue,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>
    {
        visitor.visit_pair(self.0.to_key(), self.1.to_value())
    }
}

impl<KVS> Source for [KVS] where KVS: Source {
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        for kv in self {
            kv.visit(visitor)?;
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use std::borrow::Borrow;
    use std::sync::Arc;
    use std::rc::Rc;
    use std::hash::Hash;
    use std::collections::{HashMap, BTreeMap};

    impl<KVS: ?Sized> Source for Box<KVS> where KVS: Source {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
            (**self).visit(visitor)
        }
    }

    impl<KVS: ?Sized> Source for Arc<KVS> where KVS: Source  {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
            (**self).visit(visitor)
        }
    }

    impl<KVS: ?Sized> Source for Rc<KVS> where KVS: Source  {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
            (**self).visit(visitor)
        }
    }

    impl<KVS> Source for Vec<KVS> where KVS: Source {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
            self.as_slice().visit(visitor)
        }
    }

    impl<K, V> Source for BTreeMap<K, V>
    where
        K: Borrow<str> + Ord,
        V: ToValue,
    {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>
        {
            for (k, v) in self {
                visitor.visit_pair(k.borrow().to_key(), v.to_value())?;
            }

            Ok(())
        }

        fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<Value<'kvs>>
        where
            Q: ToKey,
        {
            BTreeMap::get(self, key.to_key().borrow()).map(|v| v.to_value())
        }
    }

    impl<K, V> Source for HashMap<K, V>
    where
        K: Borrow<str> + Eq + Hash,
        V: ToValue,
    {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>
        {
            for (k, v) in self {
                visitor.visit_pair(k.borrow().to_key(), v.to_value())?;
            }

            Ok(())
        }

        fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<Value<'kvs>>
        where
            Q: ToKey,
        {
            HashMap::get(self, key.to_key().borrow()).map(|v| v.to_value())
        }
    }
}
