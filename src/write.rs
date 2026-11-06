//! A [`Value`] written by any serde `Serializer`: null as a unit, a float32 as its own float,
//! a timestamp and a duration as their text.

use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Serialize, Serializer};

use crate::{duration_text, timestamp_text, Value};

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Null => s.serialize_unit(),
            Value::Bool(b) => s.serialize_bool(*b),
            Value::Int(i) => s.serialize_i64(*i),
            Value::UInt(u) => s.serialize_u64(*u),
            Value::Double(d) => s.serialize_f64(*d),
            Value::Float32(f) => s.serialize_f32(*f),
            Value::Str(t) => s.serialize_str(t),
            Value::Timestamp(t) => s.serialize_str(&timestamp_text(*t)),
            Value::Duration(d) => s.serialize_str(&duration_text(*d)),
            Value::List(xs) => {
                let mut seq = s.serialize_seq(Some(xs.len()))?;
                for x in xs {
                    seq.serialize_element(x)?;
                }
                seq.end()
            }
            Value::Map(m) => {
                let mut map = s.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}
