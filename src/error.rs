use serde::de::Expected;

#[derive(Debug)]
pub enum Error {
    InvalidBytes,
}

impl Error {
    pub fn io(io_err: &std::io::Error) -> Self {
        todo!()
    }

    pub fn eof() -> Self {
        todo!()
    }

    /// expect some value but got something else
    pub fn expected_value(expecting: &str) -> Self {
        todo!()
    }

    pub fn invalid_length() -> Self {
        todo!()
    }

    pub fn expect_ident() -> Self {
        todo!()
    }

    pub fn utf8(valid_up_to: usize) -> Self {
        todo!()
    }

    pub fn invalid_type(expecting: &str) -> Self {
        todo!()
    }

    pub fn overflow() -> Self {
        todo!()
    }

    pub fn expected_number() -> Self {
        todo!()
    }

    pub fn parse() -> Self {
        todo!()
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
        Error::InvalidBytes
    }
}

pub type Result<T> = std::result::Result<T, Error>;
