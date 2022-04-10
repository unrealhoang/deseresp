mod de;
mod error;
mod ser;
pub mod types;

pub use de::{from_read, from_slice, Deserializer};
pub use error::{Error, Result};
pub use ser::{from_write, Serializer};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
