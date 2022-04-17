/// Serialize or Deserialize error
#[derive(Debug)]
pub enum Error {
    /// Error coming from underlying io
    IO(std::io::Error),
    /// Unexpected EOF encountered
    EOF,
    /// Expected a marker of one type but received something else
    ExpectedMarker(&'static str),
    /// Expected a value of one type but received something else
    ExpectedValue(&'static str),
    /// Received an unexpected value
    UnexpectedValue(&'static str),
    /// Failed to convert the underlying bytes to utf8, value return the offset right before the
    /// invalid utf8
    UTF8(usize),
    /// Failed to parse a float value
    Parse,
    /// Received a NaN
    NaN,
    /// Custom error from serialize/deserialize
    Custom(String),
}

impl Error {
    pub fn io(err: std::io::Error) -> Self {
        Error::IO(err)
    }

    pub fn eof() -> Self {
        Error::EOF
    }

    pub fn expected_marker(expecting: &'static str) -> Self {
        Error::ExpectedMarker(expecting)
    }

    /// expect some value but got something else
    pub fn expected_value(expecting: &'static str) -> Self {
        Error::ExpectedValue(expecting)
    }

    pub fn unexpected_value(unexpected: &'static str) -> Self {
        Error::UnexpectedValue(unexpected)
    }

    pub fn utf8(valid_up_to: usize) -> Self {
        Error::UTF8(valid_up_to)
    }

    pub fn overflow() -> Self {
        Error::Parse
    }

    pub fn parse() -> Self {
        Error::Parse
    }

    pub fn nan() -> Self {
        Error::NaN
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(e) => write!(f, "IO Error:\n{}", e),
            Error::EOF => write!(f, "Reach unexpected EOF"),
            Error::ExpectedMarker(m) => write!(f, "expected marker {}, received other", m),
            Error::ExpectedValue(v) => write!(f, "expected value {}, received other", v),
            Error::UnexpectedValue(v) => write!(f, "received unexpected value {}", v),
            Error::UTF8(_) => write!(f, "failed to parse input as utf8"),
            Error::Parse => write!(f, "failed to parse number or overflow"),
            Error::NaN => write!(f, "NaN received"),
            Error::Custom(c) => write!(f, "Custom error:\n{}", c),
        }
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

/// Result from serialize/deserialize
pub type Result<T> = std::result::Result<T, Error>;
