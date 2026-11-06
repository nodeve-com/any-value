use core::fmt;

/// What reading a value or a time's text fails with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// A value of a kind the place does not take, or text that is no timestamp or duration.
    Kind,
    /// A number or time past the width or range of the place.
    Range,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::Kind => "a value of another kind",
            Error::Range => "a value out of range",
        })
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(_: T) -> Self {
        Error::Kind
    }
}
