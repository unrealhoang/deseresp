//! Contains specific RESP types implemented with custom Serialize/Deserialize
//! to tell [`Serializer`](crate::Serializer)/[`Deserializer`](crate::Deserializer)
//! to se/der correctly.
pub(crate) const SIMPLE_ERROR_TOKEN: &str = "$SimpleError";
pub(crate) const BLOB_ERROR_TOKEN: &str = "$BulkError";
pub(crate) const SIMPLE_STRING_TOKEN: &str = "$SimpleString";
pub(crate) const BLOB_STRING_TOKEN: &str = "$BulkString";
pub(crate) const ATTRIBUTE_SKIP_TOKEN: &str = "$AttributeSkip";
pub(crate) const ATTRIBUTE_TOKEN: &str = "$WithAttribute";

use std::marker::PhantomData;

use serde::{
    de::{self, DeserializeOwned, Visitor},
    Deserialize, Serialize,
};
pub mod owned {
    //! Contain owned types (String, Vec)
    use serde::{de::Visitor, Serialize};

    use super::*;

    /// Expects a SimpleError from deserializer,
    /// Serialize as a RESP SimpleError
    pub struct SimpleError(pub String);
    /// Expects a BlobError from deserializer,
    /// Serialize as a RESP BlobError
    pub struct BlobError(pub String);
    /// Expects a SimpleString from deserializer,
    /// Serialize as a RESP SimpleString
    pub struct SimpleString(pub String);
    /// Expects a BlobString from deserializer,
    /// Serialize as a RESP BlobString
    pub struct BlobString(pub String);

    macro_rules! impl_initializers {
        ($type_name:ident) => {
            impl From<String> for $type_name {
                fn from(s: String) -> Self {
                    $type_name(s)
                }
            }
            impl From<&str> for $type_name {
                fn from(s: &str) -> Self {
                    $type_name(s.to_owned())
                }
            }
        };
    }
    impl_initializers!(SimpleError);
    impl_initializers!(BlobError);
    impl_initializers!(SimpleString);
    impl_initializers!(BlobString);

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

    macro_rules! impl_serialize {
        ($type_name:ident: $token:ident) => {
            impl Serialize for $type_name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_newtype_struct($token, &self.0)
                }
            }
        };
    }
    impl_serialize!(SimpleError: SIMPLE_ERROR_TOKEN);
    impl_serialize!(BlobError: BLOB_ERROR_TOKEN);
    impl_serialize!(SimpleString: SIMPLE_STRING_TOKEN);
    impl_serialize!(BlobString: BLOB_STRING_TOKEN);
}

pub mod borrowed {
    //! Contain borrowed types (&str, &[])
    use std::borrow::Cow;

    use serde::{de::Visitor, Serialize};

    use super::*;

    /// Expects a SimpleError from deserializer,
    /// Serialize as a RESP SimpleError
    pub struct SimpleError<'a>(pub Cow<'a, str>);
    /// Expects a BlobError from deserializer,
    /// Serialize as a RESP BlobError
    pub struct BlobError<'a>(pub Cow<'a, str>);
    /// Expects a SimpleString from deserializer,
    /// Serialize as a RESP SimpleString
    pub struct SimpleString<'a>(pub Cow<'a, str>);
    /// Expects a BlobString from deserializer,
    /// Serialize as a RESP BlobString
    pub struct BlobString<'a>(pub Cow<'a, str>);

    macro_rules! impl_initializers {
        ($type_name:ident<$lt:lifetime>) => {
            impl<$lt> From<String> for $type_name<$lt> {
                fn from(s: String) -> Self {
                    $type_name(Cow::from(s))
                }
            }
            impl<$lt> From<&$lt str> for $type_name<$lt> {
                fn from(s: &$lt str) -> Self {
                    $type_name(Cow::from(s))
                }
            }
        }
    }
    impl_initializers!(SimpleError<'a>);
    impl_initializers!(BlobError<'a>);
    impl_initializers!(SimpleString<'a>);
    impl_initializers!(BlobString<'a>);

    macro_rules! impl_deserialize {
        ($type_name:ident<$lt:lifetime>: $token:ident => $visitor_name:ident) => {
            struct $visitor_name;
            impl<'de> Visitor<'de> for $visitor_name {
                type Value = $type_name<'de>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "expecting borrowed str")
                }

                fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok($type_name(Cow::from(v)))
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok($type_name(Cow::from(v.to_owned())))
                }
            }
            impl<'de> Deserialize<'de> for $type_name<'de> {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    deserializer.deserialize_newtype_struct($token, $visitor_name)
                }
            }
        };
    }
    impl_deserialize!(SimpleError<'de>: SIMPLE_ERROR_TOKEN => SimpleErrorVisitor);
    impl_deserialize!(BlobError<'de>: BLOB_ERROR_TOKEN => BlobErrorVisitor);
    impl_deserialize!(SimpleString<'de>: SIMPLE_STRING_TOKEN => SimpleStringVisitor);
    impl_deserialize!(BlobString<'de>: BLOB_STRING_TOKEN => BlobStringVisitor);

    macro_rules! impl_serialize {
        ($type_name:ident<$lt:lifetime>: $token:ident) => {
            impl<$lt> Serialize for $type_name<$lt> {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    serializer.serialize_newtype_struct($token, &self.0)
                }
            }
        };
    }
    impl_serialize!(SimpleError<'a>: SIMPLE_ERROR_TOKEN);
    impl_serialize!(BlobError<'a>: BLOB_ERROR_TOKEN);
    impl_serialize!(SimpleString<'a>: SIMPLE_STRING_TOKEN);
    impl_serialize!(BlobString<'a>: BLOB_STRING_TOKEN);
}

macro_rules! empty_visit {
    ($visit_func:ident => $typ:ty) => {
        fn $visit_func<E>(self, _v: $typ) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(())
        }
    };
}

/// Custom struct to expect an attribute from [`crate::Deserializer`]
/// and ignore its value
pub(crate) struct AttributeSkip;
/// Custom struct to expect a value from [`crate::Deserializer`],
/// and ignore its value
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
        Deserialize::deserialize(deserializer)
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
        Deserialize::deserialize(deserializer)
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
        deserializer.deserialize_ignored_any(AnySkipVisitor)?;
        Ok(AnySkip)
    }
}

/// Embed a RESP value V with an attribute A
pub struct WithAttribute<A, V> {
    attr: A,
    value: V,
}
struct WithAttributeVisitor<A, V>(PhantomData<(A, V)>);

impl<A, V> WithAttribute<A, V> {
    /// Attach an attribute to a value
    pub fn new(attr: A, value: V) -> Self {
        WithAttribute { attr, value }
    }

    /// Unwrap underlying attribute and value
    pub fn into_inner(self) -> (A, V) {
        (self.attr, self.value)
    }

    /// Unwrap underlying attribute, drops the value
    pub fn into_attribute(self) -> A {
        self.attr
    }

    /// Unwrap underlying value, drop the attribute
    pub fn into_value(self) -> V {
        self.value
    }
}

impl<'de, A, V> Visitor<'de> for WithAttributeVisitor<A, V>
where
    A: DeserializeOwned,
    V: DeserializeOwned,
{
    type Value = WithAttribute<A, V>;

    fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
    where
        S: serde::de::SeqAccess<'de>,
    {
        let attr = seq
            .next_element::<A>()?
            .ok_or_else(|| de::Error::invalid_length(0, &"2 expected"))?;
        let value = seq
            .next_element::<V>()?
            .ok_or_else(|| de::Error::invalid_length(1, &"2 expected"))?;

        Ok(WithAttribute::<A, V> { attr, value })
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "expect seq (attribute, value)")
    }
}

impl<'de, A, V> Deserialize<'de> for WithAttribute<A, V>
where
    A: DeserializeOwned,
    V: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple_struct(
            ATTRIBUTE_TOKEN,
            2,
            WithAttributeVisitor::<A, V>(PhantomData),
        )
    }
}

/// OK Response from a command, equivalent to SimpleString("OK")
pub struct OkResponse;

impl<'de> Deserialize<'de> for OkResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: borrowed::SimpleString = Deserialize::deserialize(deserializer)?;
        if s.0.eq_ignore_ascii_case("ok") {
            return Err(de::Error::custom("expect +OK"));
        }
        Ok(OkResponse)
    }
}

impl Serialize for OkResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_newtype_struct(SIMPLE_STRING_TOKEN, "OK")
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{borrowed, owned};
    use crate::test_utils::{test_deserialize, test_serialize};

    #[test]
    fn serialize_borrowed_types() {
        test_serialize(
            |mut s| {
                let ss = borrowed::SimpleString::from("hello");
                ss.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"+hello\r\n");
            },
        );
        test_serialize(
            |mut s| {
                let ss = borrowed::SimpleError::from("hello");
                ss.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"-hello\r\n");
            },
        );
        test_serialize(
            |mut s| {
                let ss = borrowed::BlobString::from("hello");
                ss.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"$5\r\nhello\r\n");
            },
        );
        test_serialize(
            |mut s| {
                let ss = borrowed::BlobError::from("hello");
                ss.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"!5\r\nhello\r\n");
            },
        );
    }

    #[test]
    fn serialize_owned_types() {
        test_serialize(
            |mut s| {
                let ss = owned::SimpleString::from("hello");
                ss.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"+hello\r\n");
            },
        );
        test_serialize(
            |mut s| {
                let ss = owned::SimpleError::from("hello");
                ss.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"-hello\r\n");
            },
        );
        test_serialize(
            |mut s| {
                let ss = owned::BlobString::from("hello");
                ss.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"$5\r\nhello\r\n");
            },
        );
        test_serialize(
            |mut s| {
                let ss = owned::BlobError::from("hello");
                ss.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"!5\r\nhello\r\n");
            },
        );
    }

    #[test]
    fn deserialize_borrowed_types() {
        test_deserialize(b"+hello world\r\n", |value: borrowed::SimpleString| {
            assert_eq!(value.0, "hello world");
        });
        test_deserialize(b"-ERR hello world\r\n", |value: borrowed::SimpleError| {
            assert_eq!(value.0, "ERR hello world");
        });
        test_deserialize(b"$11\r\nhello world\r\n", |value: borrowed::BlobString| {
            assert_eq!(value.0, "hello world");
        });
        test_deserialize(
            b"!15\r\nERR hello world\r\n",
            |value: borrowed::BlobError| {
                assert_eq!(value.0, "ERR hello world");
            },
        );
    }

    #[test]
    fn deserialize_owned_types() {
        test_deserialize(b"+hello world\r\n", |value: owned::SimpleString| {
            assert_eq!(value.0, "hello world");
        });
        test_deserialize(b"-ERR hello world\r\n", |value: owned::SimpleError| {
            assert_eq!(value.0, "ERR hello world");
        });
        test_deserialize(b"$11\r\nhello world\r\n", |value: owned::BlobString| {
            assert_eq!(value.0, "hello world");
        });
        test_deserialize(b"!15\r\nERR hello world\r\n", |value: owned::BlobError| {
            assert_eq!(value.0, "ERR hello world");
        });
    }
}
