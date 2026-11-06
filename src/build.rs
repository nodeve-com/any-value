//! A [`Value`] built by any serde `Deserializer`, JSON's included.
//!
//! A whole number builds as an int, a uint past int64, any other number as a double. A repeated
//! map key keeps its last value, in the place of its first.

use core::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::{Null, Value};

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Value, D::Error> {
        d.deserialize_any(Build)
    }
}

/// Builds a [`Value`].
struct Build;

impl<'de> Visitor<'de> for Build {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a value of any kind")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }

    fn visit_bool<E: de::Error>(self, b: bool) -> Result<Value, E> {
        Ok(Value::Bool(b))
    }

    fn visit_i64<E: de::Error>(self, i: i64) -> Result<Value, E> {
        Ok(Value::Int(i))
    }

    fn visit_u64<E: de::Error>(self, u: u64) -> Result<Value, E> {
        Ok(i64::try_from(u).map_or(Value::UInt(u), Value::Int))
    }

    fn visit_f64<E: de::Error>(self, d: f64) -> Result<Value, E> {
        Ok(Value::Double(d))
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Value, E> {
        self.visit_string(s.into())
    }

    fn visit_string<E: de::Error>(self, s: String) -> Result<Value, E> {
        Ok(Value::Str(s))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element::<Value>()? {
            items.push(item);
        }
        Ok(Value::List(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut entries = Entries::default();
        while let Some(key) = map.next_key::<String>()? {
            entries.put(key, map.next_value::<Value>()?);
        }
        Ok(Value::Map(entries.list))
    }
}

/// A map's entries as they arrive: a key seen again replaces the value it had. `at` holds each
/// key's place in `list`, sorted by key.
#[derive(Default)]
struct Entries {
    list: Vec<(String, Value)>,
    at: Vec<(String, usize)>,
}

impl Entries {
    fn put(&mut self, key: String, value: Value) {
        match self.at.binary_search_by(|(k, _)| k.as_str().cmp(&key)) {
            Ok(i) => {
                let place = self.at.get(i).map(|(_, p)| *p);
                if let Some(e) = place.and_then(|p| self.list.get_mut(p)) {
                    e.1 = value;
                }
            }
            Err(i) => {
                self.at.insert(i, (key.clone(), self.list.len()));
                self.list.push((key, value));
            }
        }
    }
}

impl<'de> Deserialize<'de> for Null {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Null, D::Error> {
        d.deserialize_unit(NullVisitor)
    }
}

/// Accepts the unit: JSON's `null`.
struct NullVisitor;

impl Visitor<'_> for NullVisitor {
    type Value = Null;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("null")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Null, E> {
        Ok(Null)
    }
}
