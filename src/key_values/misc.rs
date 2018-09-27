//! Just a playground for some adapters

use std::{
    borrow::{
        Cow,
        Borrow,
    },
    collections::BTreeMap,
};

use super::{
    KeyValues,
    KeyValue,
    ToKey,
    ToValue,
    Visitor,
    Key,
    Value,
};

/// Sort the inner key values, retaining the last one with a given key.
#[derive(Debug)]
pub struct SortLast<KVS> {
    source: KVS,
}

impl<KVS> KeyValues for SortLast<KVS>
where
    KVS: KeyValues,
{
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>) {
        // The `'kvs` lifetime allows us to capture keys and values.
        // We need an allocated map to sort the keys, but we don't
        // need to try get owned copies of the keys or values themselves.
        struct Seen<'kvs>(BTreeMap<Key<'kvs>, Value<'kvs>>);

        impl<'kvs> Visitor<'kvs> for Seen<'kvs> {
            fn visit_pair<'vis>(&'vis mut self, k: Key<'kvs>, v: Value<'kvs>) {
                self.0.insert(k, v);
            }
        }

        let mut seen = Seen(BTreeMap::new());
        self.source.visit(&mut seen);

        for (k, v) in seen.0 {
            visitor.visit_pair(k, v);
        }
    }
}
