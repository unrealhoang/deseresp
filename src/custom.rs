pub const SIMPLE_ERROR_TOKEN: &str = "$SimpleError";
pub const BLOB_ERROR_TOKEN: &str = "$BulkError";
pub const SIMPLE_STRING_TOKEN: &str = "$SimpleString";
pub const BLOB_STRING_TOKEN: &str = "$BulkString";

use serde::Deserialize;
pub mod owned {
    use serde::de::Visitor;

    use super::*;

    pub struct SimpleError(pub String);
    pub struct BlobError(pub String);
    pub struct SimpleString(pub String);
    pub struct BlobString(pub String);

    macro_rules! impl_deserialize {
        ($type_name:ident: $token:ident => $visitor_name:ident) => {
            struct $visitor_name;
            impl<'de> Visitor<'de> for $visitor_name {
                type Value = $type_name;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "expecting str")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok($type_name(v.to_owned()))
                }
            }
            impl<'de> Deserialize<'de> for $type_name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer.deserialize_newtype_struct($token, $visitor_name)
                }
            }
        };
    }
    impl_deserialize!(SimpleError: SIMPLE_ERROR_TOKEN => SimpleErrorVisitor);
    impl_deserialize!(BlobError: BLOB_ERROR_TOKEN => BlobErrorVisitor);
    impl_deserialize!(SimpleString: SIMPLE_STRING_TOKEN => SimpleStringVisitor);
    impl_deserialize!(BlobString: BLOB_STRING_TOKEN => BlobStringVisitor);
}
