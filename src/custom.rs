pub const SIMPLE_ERROR_TOKEN: &str = "$SimpleError";
pub const BLOB_ERROR_TOKEN: &str = "$BulkError";
pub const SIMPLE_STRING_TOKEN: &str = "$SimpleString";
pub const BLOB_STRING_TOKEN: &str = "$BulkString";
pub const ATTRIBUTE_SKIP_TOKEN: &str = "$AttributeSkip";

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

    macro_rules! empty_visit {
        ($visit_func:ident => $typ:ty) => {
            fn $visit_func<E>(self, _v: $typ) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(())
            }
        }
    }

    pub struct AttributeSkip;
    pub struct AnySkip;
    struct AnySkipVisitor;
    impl<'de> Visitor<'de> for AnySkipVisitor {
        type Value = ();

        empty_visit!(visit_bool => bool);
        empty_visit!(visit_i8 => i8);
        empty_visit!(visit_i16 => i16);
        empty_visit!(visit_i32 => i32);
        empty_visit!(visit_i64 => i64);
        empty_visit!(visit_u8 => u8);
        empty_visit!(visit_u16 => u16);
        empty_visit!(visit_u32 => u32);
        empty_visit!(visit_u64 => u64);
        empty_visit!(visit_f32 => f32);
        empty_visit!(visit_f64 => f64);
        empty_visit!(visit_char => char);
        empty_visit!(visit_str => &str);
        empty_visit!(visit_borrowed_str => &'de str);
        empty_visit!(visit_string => String);
        empty_visit!(visit_bytes => &[u8]);
        empty_visit!(visit_borrowed_bytes => &'de [u8]);
        empty_visit!(visit_byte_buf => Vec<u8>);

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(())
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ok(())
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(())
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            Ok(())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            while let Some(_s) = seq.next_element::<AnySkip>()? {}

            Ok(())
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            while let Some(_s) = map.next_key::<AnySkip>()? {
                println!("skipped");
                map.next_value::<AnySkip>()?;
            }

            Ok(())
        }

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "expect everything")
        }
    }

    impl<'de> Deserialize<'de> for AttributeSkip {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_newtype_struct(ATTRIBUTE_SKIP_TOKEN, AnySkipVisitor)?;
            Ok(AttributeSkip)
        }
    }

    impl<'de> Deserialize<'de> for AnySkip {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_any(AnySkipVisitor)?;
            Ok(AnySkip)
        }
    }

}
