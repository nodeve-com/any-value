//! The text forms, and `Serialize` and `Deserialize` driven through JSON text.

use super::*;

fn read(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn write(x: &Value) -> String {
    serde_json::to_string(x).unwrap()
}

fn ms(text: &str) -> Result<i64, Error> {
    parse_duration(text).map(|d| d.num_milliseconds())
}

#[test]
fn a_document_round_trips() {
    for text in [
        "null",
        "true",
        "-3",
        "1.5",
        r#""héllo \" \n""#,
        "[]",
        "{}",
        r#"{"a":[1,{"b":null},[]],"c":{"d":"e"}}"#,
        "18446744073709551615",
    ] {
        assert_eq!(write(&read(text)), text);
    }
}

#[test]
fn numbers_keep_their_kind_at_the_boundaries() {
    assert_eq!(read("9223372036854775807"), Value::Int(i64::MAX));
    assert_eq!(read("-9223372036854775808"), Value::Int(i64::MIN));
    assert_eq!(read("9223372036854775808"), Value::UInt(1 << 63));
    assert_eq!(read("18446744073709551615"), Value::UInt(u64::MAX));
    assert_eq!(read("1.0"), Value::Double(1.0));
    assert_eq!(read("0"), Value::Int(0));
    assert!(matches!(read("18446744073709551616"), Value::Double(_)));
}

#[test]
fn a_repeated_key_keeps_its_last_value_in_its_first_place() {
    assert_eq!(write(&read(r#"{"z":1,"a":2,"z":3}"#)), r#"{"z":3,"a":2}"#);
}

#[test]
fn a_float32_writes_its_own_digits() {
    assert_eq!(write(&Value::Float32(9.3)), "9.3");
    assert_eq!(
        write(&Value::Double(f64::from(9.3_f32))),
        "9.300000190734863"
    );
}

#[test]
fn a_time_writes_its_text() {
    let at = DateTime::from_timestamp(1_700_000_000, 500_000_000).unwrap();
    let span = TimeDelta::milliseconds(1500);
    let list = Value::List(vec![at.into(), span.into()]);
    assert_eq!(write(&list), r#"["2023-11-14T22:13:20.500Z","1.5s"]"#);
}

#[test]
fn an_instant_reads_at_its_offset() {
    let at = parse_timestamp("2026-10-11T18:33:00.5+02:00").unwrap();
    assert_eq!(timestamp_text(at), "2026-10-11T16:33:00.500Z");
    assert_eq!(parse_timestamp("2026-10-11"), Err(Error::Kind));
}

#[test]
fn a_leap_second_is_the_next_second() {
    let at = |t| timestamp_text(parse_timestamp(t).unwrap());
    assert_eq!(at("2016-12-31T23:59:60Z"), "2017-01-01T00:00:00Z");
    assert_eq!(at("2016-12-31T18:59:60-05:00"), "2017-01-01T00:00:00Z");
    assert_eq!(parse_timestamp("2016-12-31T22:59:60Z"), Err(Error::Kind));
}

#[test]
fn spans_read_with_a_sign_and_humantime_units() {
    assert_eq!(ms("1.5s"), Ok(1500));
    assert_eq!(ms("-7200s"), Ok(-7_200_000));
    assert_eq!(ms("+90m"), Ok(5_400_000));
    assert_eq!(ms("1h30m"), Ok(5_400_000));
    assert_eq!(ms("1500ms"), Ok(1500));
    assert_eq!(ms("0"), Ok(0));
}

#[test]
fn what_a_span_writes_reads_back() {
    let spans = [
        TimeDelta::new(315_537_897_598, 999_999_999),
        TimeDelta::new(-315_537_897_598, 0),
        Some(TimeDelta::nanoseconds(1)),
        Some(TimeDelta::nanoseconds(-1)),
        Some(TimeDelta::milliseconds(1500)),
        Some(TimeDelta::seconds(-7200)),
        Some(TimeDelta::zero()),
    ];
    for d in spans.into_iter().flatten() {
        assert_eq!(parse_duration(&duration_text(d)), Ok(d), "{d}");
    }
}

#[test]
fn past_a_delta_is_a_range_and_junk_is_a_kind() {
    for text in ["99999999999999999999s", "9223372036854775807s"] {
        assert_eq!(parse_duration(text), Err(Error::Range), "{text}");
    }
    for bad in ["", "s", "P1D", "x", "--5s", "+-5s", "-", "1 fortnight"] {
        assert_eq!(parse_duration(bad), Err(Error::Kind), "{bad}");
    }
}

#[test]
fn null_writes_as_json_null() {
    assert_eq!(serde_json::to_string(&Null).unwrap(), "null");
}

#[test]
fn null_reads_only_null() {
    assert_eq!(serde_json::from_str::<Null>("null").unwrap(), Null);
    assert_eq!(serde_json::from_str::<Option<Null>>("null").unwrap(), None);
    assert!(serde_json::from_str::<Null>("0").is_err());
}
