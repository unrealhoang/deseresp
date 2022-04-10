pub const SIMPLE_ERROR_TOKEN: &str = "$SimpleError";
pub const BLOB_ERROR_TOKEN: &str = "$BulkError";
pub const SIMPLE_STRING_TOKEN: &str = "$SimpleString";
pub const BLOB_STRING_TOKEN: &str = "$BulkString";
pub const ATTRIBUTE_SKIP_TOKEN: &str = "$AttributeSkip";
pub const ATTRIBUTE_TOKEN: &str = "$WithAttribute";

use std::marker::PhantomData;

use serde::{
    de::{self, DeserializeOwned, Visitor},
    Deserialize,
};
pub mod owned {
    use serde::{de::Visitor, Serialize};

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

pub struct WithAttribute<A, V> {
    attr: A,
    value: V,
}
struct WithAttributeVisitor<A, V>(PhantomData<(A, V)>);

impl<A, V> WithAttribute<A, V> {
    pub fn into_inner(self) -> (A, V) {
        (self.attr, self.value)
    }

    pub fn into_attribute(self) -> A {
        self.attr
    }

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
