//! Timestamps and durations as text, as JSON carries them: RFC 3339 for an instant; for a span,
//! seconds with an `s`, and for reading, an optional sign and the units [`humantime`] reads
//! (`1.5s`, `90m`, `1h30m`), which includes the text [`duration_text`] writes.

use chrono::{DateTime, TimeDelta, Timelike, Utc};

use crate::Error;

/// `t` as RFC 3339 in UTC, with the fraction only where there is one.
pub fn timestamp_text(t: DateTime<Utc>) -> String {
    t.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
}

/// `d` as seconds, with the fraction only where there is one: `1.5s`, `-7200s`.
pub fn duration_text(d: TimeDelta) -> String {
    let (secs, frac) = (d.num_seconds(), d.subsec_nanos());
    let sign = if secs < 0 || frac < 0 { "-" } else { "" };
    match frac {
        0 => format!("{sign}{}s", secs.unsigned_abs()),
        _ => {
            let digits = format!("{:09}", frac.unsigned_abs());
            format!(
                "{sign}{}.{}s",
                secs.unsigned_abs(),
                digits.trim_end_matches('0')
            )
        }
    }
}

/// An RFC 3339 `date-time`, its offset applied. A leap second reads as the next second.
/// Text that is no such time is [`Error::Kind`]; an instant chrono can't hold is [`Error::Range`].
pub fn parse_timestamp(text: &str) -> Result<DateTime<Utc>, Error> {
    let at = DateTime::parse_from_rfc3339(text).map_err(|_| Error::Kind)?;
    let at = at.with_timezone(&Utc);
    let (seconds, nanos) = (at.timestamp(), at.timestamp_subsec_nanos());
    // A leap second is the last of a UTC day.
    let (seconds, nanos) = match nanos.checked_sub(1_000_000_000) {
        Some(past) if (at.hour(), at.minute()) == (23, 59) => (seconds.saturating_add(1), past),
        Some(_) => return Err(Error::Kind),
        None => (seconds, nanos),
    };
    DateTime::from_timestamp(seconds, nanos).ok_or(Error::Range)
}

/// An optional `-` or `+`, then a span in [`humantime`]'s units. Text that is no duration is
/// [`Error::Kind`]; a span past what a [`TimeDelta`] holds is [`Error::Range`].
#[allow(clippy::disallowed_methods)]
pub fn parse_duration(text: &str) -> Result<TimeDelta, Error> {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let span = humantime::parse_duration(digits).map_err(|e| match e {
        humantime::DurationError::NumberOverflow => Error::Range,
        _ => Error::Kind,
    })?;
    let seconds = i64::try_from(span.as_secs()).map_err(|_| Error::Range)?;
    let nanos = i64::from(span.subsec_nanos());
    let (seconds, nanos) = match negative {
        true => (seconds.saturating_neg(), nanos.saturating_neg()),
        false => (seconds, nanos),
    };
    TimeDelta::try_seconds(seconds)
        .and_then(|whole| whole.checked_add(&TimeDelta::nanoseconds(nanos)))
        .ok_or(Error::Range)
}
