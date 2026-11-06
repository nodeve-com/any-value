//! A [`Value`] read by any serde `Deserialize`: a type fills itself from a value, no copy of the
//! value made. A timestamp or duration reads as its text.

use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::de::{self, DeserializeOwned, IntoDeserializer, Visitor};
use serde::forward_to_deserialize_any;

use crate::{duration_text, timestamp_text, Error, Value};

/// A `T` filled from `x`; a value `T` does not accept is [`Error::Kind`], and a number past the
/// width of the integer or float32 it fills is [`Error::Range`].
pub fn to<T: DeserializeOwned>(x: &Value) -> Result<T, Error> {
    T::deserialize(x)
}

/// `x` as a whole number: an int, a uint, or a double with no fraction. A fractional double or
/// another kind is [`Error::Kind`]; a double past `i128` is [`Error::Range`].
fn whole(x: &Value) -> Result<i128, Error> {
    let d = match x {
        Value::Int(i) => return Ok(i128::from(*i)),
        Value::UInt(u) => return Ok(i128::from(*u)),
        Value::Double(d) => *d,
        Value::Float32(f) => f64::from(*f),
        _ => return Err(Error::Kind),
    };
    if d.fract() != 0.0 {
        return Err(Error::Kind);
    }
    // 2^100 is far past every width; the cast below stays exact.
    if d.abs() >= 2f64.powi(100) {
        return Err(Error::Range);
    }
    Ok(d as i128)
}

/// `x` as a double: any int, uint, or double.
fn real(x: &Value) -> Result<f64, Error> {
    match x {
        Value::Int(i) => Ok(*i as f64),
        Value::UInt(u) => Ok(*u as f64),
        Value::Double(d) => Ok(*d),
        Value::Float32(f) => Ok(f64::from(*f)),
        _ => Err(Error::Kind),
    }
}

macro_rules! integers {
    ($($method:ident $visit:ident $ty:ty),*) => {$(
        fn $method<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
            let n = whole(self)?;
            v.$visit(<$ty>::try_from(n).map_err(|_| Error::Range)?)
        }
    )*};
}

impl<'de> IntoDeserializer<'de, Error> for &'de Value {
    type Deserializer = &'de Value;

    fn into_deserializer(self) -> &'de Value {
        self
    }
}

impl<'de> de::Deserializer<'de> for &'de Value {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        match self {
            Value::Null => v.visit_unit(),
            Value::Bool(b) => v.visit_bool(*b),
            Value::Int(i) => v.visit_i64(*i),
            Value::UInt(u) => v.visit_u64(*u),
            Value::Double(d) => v.visit_f64(*d),
            Value::Float32(f) => v.visit_f32(*f),
            Value::Str(s) => v.visit_borrowed_str(s),
            Value::Timestamp(t) => v.visit_string(timestamp_text(*t)),
            Value::Duration(d) => v.visit_string(duration_text(*d)),
            Value::List(xs) => v.visit_seq(SeqDeserializer::new(xs.iter())),
            Value::Map(m) => {
                let entries = m.iter().map(|(k, v)| (k.as_str(), v));
                v.visit_map(MapDeserializer::new(entries))
            }
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        match self {
            Value::Null => v.visit_none(),
            x => v.visit_some(x),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        v: V,
    ) -> Result<V::Value, Error> {
        match self {
            Value::Str(s) => v.visit_enum(s.as_str().into_deserializer()),
            x => x.deserialize_any(v),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        v: V,
    ) -> Result<V::Value, Error> {
        v.visit_newtype_struct(self)
    }

    integers! {
        deserialize_i8 visit_i8 i8, deserialize_i16 visit_i16 i16,
        deserialize_i32 visit_i32 i32, deserialize_i64 visit_i64 i64,
        deserialize_i128 visit_i128 i128, deserialize_u8 visit_u8 u8,
        deserialize_u16 visit_u16 u16, deserialize_u32 visit_u32 u32,
        deserialize_u64 visit_u64 u64, deserialize_u128 visit_u128 u128
    }

    fn deserialize_f64<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        v.visit_f64(real(self)?)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Error> {
        let d = real(self)?;
        if d.is_finite() && d.abs() > f64::from(f32::MAX) {
            return Err(Error::Range);
        }
        v.visit_f32(d as f32)
    }

    forward_to_deserialize_any! {
        bool char str string bytes byte_buf unit
        unit_struct seq tuple tuple_struct map struct identifier ignored_any
    }
}
