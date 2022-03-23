mod de;
mod error;
mod ser;
pub mod custom;

pub use de::{Deserializer};
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
