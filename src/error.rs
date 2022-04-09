#[derive(Debug)]
pub enum Error {
    InvalidBytes,
    IO(std::io::Error),
    EOF,
    ExpectedMarker(&'static str),
    ExpectedValue(&'static str),
    UnexpectedValue(&'static str),
    UTF8(usize),
    Parse,
    NaN,
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
        write!(f, "TODO")
    }
}

impl std::error::Error for Error {
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display
    {
        Error::Custom(msg.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
