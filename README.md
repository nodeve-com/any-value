# any-value

A value of any one kind that JSON holds, plus timestamps and durations, and the [serde](https://serde.rs) code that reads and writes it.

serde is Rust's common library for turning data into text and back. A type that implements its `Serialize` trait (a set of functions a type provides) can go out as text. A type that implements `Deserialize` can come in from text.

Use it where a program learns the kind of its data only on reading it: a field that might hold a number, a text, a list, or a map.

## Types

- **`Value`**: one value of a kind known only at run time. Its variants:
  - `Null`
  - `Bool`
  - `Int`, a signed 64-bit integer
  - `UInt`, an unsigned 64-bit integer
  - `Double`, a 64-bit float
  - `Float32`, a 32-bit float
  - `Str`, text
  - `Timestamp`, an instant, a chrono `DateTime<Utc>`
  - `Duration`, a length of time, a chrono `TimeDelta`
  - `List`, a `Vec` of values
  - `Map`, a `Vec` of key and value pairs, keys text, in the order the source gave them
- **`Null`**: a unit type (a type with exactly one value) for the null value itself. A caller that holds an optional keeps "no value" apart from "the value null" by holding `Option<Null>`. `Value::from(Null)` is `Value::Null`.
- **`Error`**: what reading fails with. `Kind` means a value of a kind the place does not take, or text that is no timestamp or duration. `Range` means a number or time past the width or range of the place.

A float32 is a double that keeps its own shortest digits on output: `Float32(9.3)` writes `9.3`, where the same number as a `Double` writes `9.300000190734863`.

`Value` converts from `bool`, `i64`, `u64`, `f64`, `f32`, `String`, `DateTime<Utc>`, `TimeDelta`, `Vec<Value>`, a `Vec` of text and value pairs, `Null`, and an `Option` of any of them, where `None` is null.

## Writing a value

`Serialize` writes a `Value` to any serde format. Null goes out as a unit, which JSON writes as `null`. A timestamp and a duration write as their text (below). A map keeps its pair order.

## Building a value

`Deserialize` builds a `Value` from any serde format, JSON included. A whole number builds as an `Int`, a number past `i64` that fits `u64` as a `UInt`, any other number as a `Double`. A map key seen again keeps its last value, in the place of its first.

## Filling a typed value

`to::<T>(&value)` fills any type `T` that implements `Deserialize` from a value, with no copy of the value. A `&Value` is a serde `Deserializer`: the half of serde that hands data to a type as it reads. So a type with serde's derive macro takes a `Map` as its struct, a `List` as its `Vec`, and a `Str` as its enum variant. A `Null` fills an `Option` as `None`.

Numbers cross kinds only where they lose nothing:

- An integer place takes an `Int`, a `UInt`, or a double with no fraction. A fractional double is `Kind`. A number past the place's width is `Range`.
- A double place takes any `Int`, `UInt`, or double. A float32 place takes a double within its range, else `Range`.
- Text never fills a number, and a number never fills text. Another kind is `Kind`.
- A timestamp or duration fills a text place as its text. A typed place that reads text, such as chrono's `DateTime<Utc>`, then parses it.

## Timestamps and durations as text

- `timestamp_text(t)`: RFC 3339 in UTC, with a fraction only where there is one: `2026-10-11T16:33:00.500Z`.
- `duration_text(d)`: seconds with an `s`, with a fraction only where there is one: `1.5s`, `-7200s`.
- `parse_timestamp(text)`: an RFC 3339 date and time, its offset applied. A leap second (second 60 of a UTC day's last minute) reads as the next second. Text that is no such time is `Kind`; an instant chrono cannot hold is `Range`.
- `parse_duration(text)`: an optional `-` or `+`, then the units the [humantime](https://docs.rs/humantime) crate reads, such as `1.5s`, `90m`, or `1h30m`. It reads everything `duration_text` writes. Text that is no duration is `Kind`; a span past what a `TimeDelta` holds is `Range`.

Neither parser limits a time to any language's own range. A caller with a narrower range checks the result.

## Dependencies

- `serde`, with no derive macro: `Serialize`, `Deserialize`, and the `Deserializer` for `to`.
- `chrono`, without its clock or zone database: timestamps and durations, and their RFC 3339 text.
- `humantime`: a duration's units.

The crate uses the standard library. Its tests also use `serde_json` as the format they read and write through.
