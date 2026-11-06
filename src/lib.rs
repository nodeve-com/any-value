#![doc = include_str!("../README.md")]

mod build;
mod error;
mod read;
mod time;
mod write;

pub use chrono::{DateTime, TimeDelta, Utc};
pub use error::Error;
pub use read::to;
pub use time::{duration_text, parse_duration, parse_timestamp, timestamp_text};

/// The null value: a value of its own, which an optional holds apart from absence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Null;

/// A value whose kind is known only when read. A float32 is a double that keeps its own
/// shortest digits when written.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Double(f64),
    Float32(f32),
    Str(String),
    Timestamp(DateTime<Utc>),
    Duration(TimeDelta),
    List(Vec<Value>),
    Map(Vec<(String, Value)>),
}

macro_rules! from {
    ($($t:ty => $v:ident),*) => {$(
        impl From<$t> for Value {
            fn from(x: $t) -> Value {
                Value::$v(x)
            }
        }
    )*};
}

from!(bool => Bool, i64 => Int, u64 => UInt, f64 => Double, f32 => Float32, String => Str,
    DateTime<Utc> => Timestamp, TimeDelta => Duration, Vec<Value> => List,
    Vec<(String, Value)> => Map);

/// A nullable value: what it holds, or null.
impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(x: Option<T>) -> Value {
        x.map_or(Value::Null, Into::into)
    }
}

impl From<Null> for Value {
    fn from(_: Null) -> Value {
        Value::Null
    }
}

#[cfg(test)]
mod tests;
