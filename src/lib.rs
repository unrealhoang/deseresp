#[doc = include_str!("../README.md")]
mod de;
mod error;
mod ser;
pub mod types;

pub use de::{from_read, from_slice, Deserializer};
pub use error::{Error, Result};
pub use ser::{from_write, to_vec, Serializer};

#[cfg(test)]
pub(crate) mod test_utils {
    use std::io::Cursor;

    use serde::Deserialize;

    use crate::Deserializer;

    pub(crate) fn test_deserialize<'de, T, F>(input: &'de [u8], test_fn: F)
    where
        T: Deserialize<'de>,
        F: Fn(T),
    {
        let mut read_d = Deserializer::from_read(Cursor::new(Vec::from(input)));
        let value: T = Deserialize::deserialize(&mut read_d).unwrap();
        test_fn(value);

        let mut slice_d = Deserializer::from_slice(input);
        let value: T = Deserialize::deserialize(&mut slice_d).unwrap();
        test_fn(value);
    }

    pub(crate) fn test_deserialize_result<'de, T, F>(input: &'de [u8], test_fn: F)
    where
        T: Deserialize<'de>,
        F: Fn(Result<T, super::Error>),
    {
        let mut read_d = Deserializer::from_read(Cursor::new(Vec::from(input)));
        let value: Result<T, super::Error> = Deserialize::deserialize(&mut read_d);
        test_fn(value);

        let mut slice_d = Deserializer::from_slice(input);
        let value: Result<T, super::Error> = Deserialize::deserialize(&mut slice_d);
        test_fn(value);
    }
}
