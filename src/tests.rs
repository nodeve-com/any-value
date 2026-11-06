use chrono::{DateTime, TimeDelta, Utc};
use serde::Deserialize;

use super::*;

mod text;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Big,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Inner {
    n: u8,
}

#[derive(Debug, PartialEq, Deserialize)]
struct Message {
    name: String,
    count: i64,
    ratio: f64,
    kind: Kind,
    at: DateTime<Utc>,
    tags: Vec<String>,
    inner: Inner,
    items: Vec<Inner>,
    maybe: Option<bool>,
    absent: Option<bool>,
    #[serde(default)]
    missing: u32,
}

fn entry(k: &str, v: impl Into<Value>) -> (String, Value) {
    (k.into(), v.into())
}

#[test]
fn a_map_fills_a_derived_struct() {
    let at = DateTime::from_timestamp(86_400, 0).unwrap_or_default();
    let n = |n: i64| Value::Map(vec![entry("n", n)]);
    let x = Value::Map(vec![
        entry("name", "a".to_string()),
        entry("count", -3_i64),
        entry("ratio", 1.5_f32),
        entry("kind", "big".to_string()),
        entry("at", at),
        entry("tags", Value::List(vec![Value::Str("t".into())])),
        entry("inner", n(7)),
        entry("items", Value::List(vec![n(1), n(2)])),
        entry("maybe", true),
        entry("absent", Value::Null),
    ]);
    let got: Message = to(&x).unwrap();
    assert_eq!(got.name, "a");
    assert_eq!((got.count, got.ratio, got.kind), (-3, 1.5, Kind::Big));
    assert_eq!((got.at, got.tags), (at, vec!["t".to_string()]));
    assert_eq!(got.inner, Inner { n: 7 });
    assert_eq!(got.items, vec![Inner { n: 1 }, Inner { n: 2 }]);
    assert_eq!((got.maybe, got.absent, got.missing), (Some(true), None, 0));
}

#[test]
fn a_list_fills_a_vec() {
    let xs = Value::List(vec![Value::Int(1), Value::Int(2)]);
    assert_eq!(to::<Vec<i64>>(&xs), Ok(vec![1, 2]));
}

#[test]
fn time_reads_as_its_text() {
    let span = TimeDelta::seconds(90);
    assert_eq!(
        to::<String>(&Value::Duration(span)),
        Ok(duration_text(span))
    );
}

#[test]
fn a_wrong_kind_is_a_kind_error() {
    assert_eq!(to::<Inner>(&Value::Int(1)), Err(Error::Kind));
    let bad = Value::Map(vec![entry("n", "x".to_string())]);
    assert_eq!(to::<Inner>(&bad), Err(Error::Kind));
    assert_eq!(to::<u8>(&Value::Double(1.5)), Err(Error::Kind));
    assert_eq!(to::<i64>(&Value::Null), Err(Error::Kind));
}

#[test]
fn a_number_past_the_width_is_a_range() {
    let big = Value::Map(vec![entry("n", 300_i64)]);
    assert_eq!(to::<Inner>(&big), Err(Error::Range));
    assert_eq!(to::<u32>(&Value::Int(-1)), Err(Error::Range));
    assert_eq!(to::<i8>(&Value::Double(200.0)), Err(Error::Range));
    assert_eq!(to::<i64>(&Value::UInt(u64::MAX)), Err(Error::Range));
    assert_eq!(to::<i64>(&Value::Double(1e30)), Err(Error::Range));
    assert_eq!(to::<f32>(&Value::Double(1e300)), Err(Error::Range));
    assert_eq!(to::<f32>(&Value::Double(1.5)), Ok(1.5));
}

#[test]
fn whole_numbers_and_doubles_cross_in_a_read() {
    assert_eq!(to::<f64>(&Value::Int(5)), Ok(5.0));
    assert_eq!(to::<f64>(&Value::UInt(3)), Ok(3.0));
    assert_eq!(to::<f64>(&Value::Float32(9.3)), Ok(9.300000190734863));
    assert_eq!(to::<f32>(&Value::Int(5)), Ok(5.0));
    assert_eq!(to::<i64>(&Value::Double(5.0)), Ok(5));
    assert_eq!(to::<i64>(&Value::Float32(-2.0)), Ok(-2));
    assert_eq!(to::<u8>(&Value::Double(5.0)), Ok(5));
    assert_eq!(to::<u64>(&Value::Int(5)), Ok(5));
    assert!(to::<f64>(&Value::Double(f64::NAN)).is_ok_and(f64::is_nan));
    assert_eq!(to::<f64>(&Value::Str("1.5".into())), Err(Error::Kind));
}

#[test]
fn typed_values_convert_into_a_value() {
    assert_eq!(Value::from(Null), Value::Null);
    assert_eq!(Value::from(Some(1_i64)), Value::Int(1));
    assert_eq!(Value::from(None::<i64>), Value::Null);
    assert_eq!(Value::from(1.5_f32), Value::Float32(1.5));
    assert_eq!(Value::from("a".to_string()), Value::Str("a".into()));
    let m = Value::from(vec![entry("k", true)]);
    assert_eq!(m, Value::Map(vec![entry("k", Value::Bool(true))]));
}
