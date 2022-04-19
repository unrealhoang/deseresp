mod de;
mod error;
mod ser;
pub mod types;

pub use de::{from_read, from_slice, Deserializer};
pub use error::{Error, Result};
pub use ser::{from_write, Serializer};

#[cfg(test)]
pub(crate) mod test_utils {
    use std::io::Cursor;

    use serde::Deserialize;

    use crate::{from_read, from_slice, from_write, Serializer};

    pub(crate) fn test_serialize<F, T>(serialize_fn: F, test_fn: T)
    where
        F: Fn(Serializer<&mut Vec<u8>>),
        T: Fn(&[u8]),
    {
        let mut buf = Vec::new();
        {
            let serializer = from_write(&mut buf);
            serialize_fn(serializer);
        }
        test_fn(&buf[..]);
    }

    pub(crate) fn test_deserialize<'de, T, F>(input: &'de [u8], test_fn: F)
    where
        T: Deserialize<'de>,
        F: Fn(T),
    {
        let mut read_d = from_read(Cursor::new(Vec::from(input)));
        let value: T = Deserialize::deserialize(&mut read_d).unwrap();
        test_fn(value);

        let mut slice_d = from_slice(input);
        let value: T = Deserialize::deserialize(&mut slice_d).unwrap();
        test_fn(value);
    }
}
