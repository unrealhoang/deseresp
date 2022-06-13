use std::{io::Write, str};

use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize,
};

use crate::{
    types::{
        BLOB_ERROR_TOKEN, BLOB_STRING_TOKEN, PUSH_TOKEN, SIMPLE_ERROR_TOKEN,
        SIMPLE_STRING_TOKEN, WITH_ATTRIBUTE_TOKEN,
    },
    Error,
};

/// A RESP Serializer
pub struct Serializer<W> {
    writer: W,
}

/// Creates a [`Serializer`] from an underlying [`Write`]
pub fn from_write<W: Write>(w: W) -> Serializer<W> {
    Serializer { writer: w }
}

/// Serialize to Vec<u8>
pub fn to_vec<S: Serialize>(s: &S) -> Result<Vec<u8>, Error> {
    let mut result = Vec::new();
    let mut serializer = from_write(&mut result);
    s.serialize(&mut serializer)?;

    Ok(result)
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

enum SeqKind {
    KnownLength,
    UnknownLength,
}

pub struct SeqSerializer<'a, W> {
    se: &'a mut Serializer<W>,
    kind: SeqKind,
    with_key: bool,
}

impl<'a, W> SeqSerializer<'a, W> {
    fn known_length(se: &'a mut Serializer<W>) -> Self {
        SeqSerializer {
            se,
            kind: SeqKind::KnownLength,
            with_key: true,
        }
    }

    fn unknown_length(se: &'a mut Serializer<W>) -> Self {
        SeqSerializer {
            se,
            kind: SeqKind::UnknownLength,
            with_key: true,
        }
    }

    fn without_key(mut self) -> Self {
        self.with_key = false;
        self
    }
}

impl<'a, W: Write> SerializeSeq for SeqSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.kind {
            SeqKind::UnknownLength => self.se.write_end(),
            SeqKind::KnownLength => Ok(()),
        }
    }
}

impl<'a, W: Write> SerializeTuple for SeqSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.kind {
            SeqKind::UnknownLength => self.se.write_end(),
            SeqKind::KnownLength => Ok(()),
        }
    }
}

impl<'a, W: Write> SerializeTupleStruct for SeqSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.kind {
            SeqKind::UnknownLength => self.se.write_end(),
            SeqKind::KnownLength => Ok(()),
        }
    }
}

impl<'a, W: Write> SerializeTupleVariant for SeqSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.kind {
            SeqKind::UnknownLength => self.se.write_end(),
            SeqKind::KnownLength => Ok(()),
        }
    }
}

impl<'a, W: Write> SerializeMap for SeqSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        key.serialize(&mut *self.se)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.kind {
            SeqKind::UnknownLength => self.se.write_end(),
            SeqKind::KnownLength => Ok(()),
        }
    }
}

impl<'a, W: Write> SerializeStruct for SeqSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        if self.with_key {
            key.serialize(&mut *self.se)?;
        }
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.kind {
            SeqKind::UnknownLength => self.se.write_end(),
            SeqKind::KnownLength => Ok(()),
        }
    }
}

impl<'a, W: Write> SerializeStructVariant for SeqSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        if self.with_key {
            key.serialize(&mut *self.se)?;
        }
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.kind {
            SeqKind::UnknownLength => self.se.write_end(),
            SeqKind::KnownLength => Ok(()),
        }
    }
}

macro_rules! serialize_err {
    ($name:ident$(<$gen_name:ident: ?Sized>)?, $($ty:ty),* => $expr:expr) => {
        fn $name$(<$gen_name: ?Sized>)?(self, $(_: $ty),*) -> Result<Self::Ok, Self::Error> {
            $expr
        }
    };
    ($name:ident$(<$gen_name:ident: ?Sized>)?, $($ty:ty),*: $return_typ:ty => $expr:expr) => {
        fn $name$(<$gen_name: ?Sized>)?(self, $(_: $ty),*) -> $return_typ {
            $expr
        }
    };
}

/// Custom type Serializer for Specific RESP types,
/// supports: SimpleError, BlobError, SimpleString, BlobString
struct RespSpecificSerializer<'a, W: Write> {
    se: &'a mut Serializer<W>,
    resp_kind: &'static str,
}

impl<'a, W: Write> serde::Serializer for RespSpecificSerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = serde::ser::Impossible<(), Error>;
    type SerializeTuple = serde::ser::Impossible<(), Error>;
    type SerializeTupleStruct = serde::ser::Impossible<(), Error>;
    type SerializeTupleVariant = serde::ser::Impossible<(), Error>;
    type SerializeMap = serde::ser::Impossible<(), Error>;
    type SerializeStruct = serde::ser::Impossible<(), Error>;
    type SerializeStructVariant = serde::ser::Impossible<(), Error>;

    serialize_err!(serialize_bool, bool => Err(Error::unexpected_value("bool")));
    serialize_err!(serialize_i8, i8 => Err(Error::unexpected_value("i8")));
    serialize_err!(serialize_i16, i16 => Err(Error::unexpected_value("i16")));
    serialize_err!(serialize_i32, i32 => Err(Error::unexpected_value("i32")));
    serialize_err!(serialize_i64, i64 => Err(Error::unexpected_value("i64")));
    serialize_err!(serialize_u8, u8 => Err(Error::unexpected_value("u8")));
    serialize_err!(serialize_u16, u16 => Err(Error::unexpected_value("u16")));
    serialize_err!(serialize_u32, u32 => Err(Error::unexpected_value("u32")));
    serialize_err!(serialize_u64, u64 => Err(Error::unexpected_value("u64")));
    serialize_err!(serialize_f32, f32 => Err(Error::unexpected_value("f32")));
    serialize_err!(serialize_f64, f64 => Err(Error::unexpected_value("f64")));
    serialize_err!(serialize_char, char => Err(Error::unexpected_value("char")));
    serialize_err!(serialize_none, => Err(Error::unexpected_value("none")));
    serialize_err!(serialize_unit, => Err(Error::unexpected_value("unit")));
    serialize_err!(serialize_some<T: ?Sized>, &T => Err(Error::unexpected_value("some")));
    serialize_err!(serialize_unit_struct, &'static str => Err(Error::unexpected_value("unit")));
    serialize_err!(serialize_unit_variant, &'static str, u32, &'static str =>
        Err(Error::unexpected_value("unit_variant"))
    );
    serialize_err!(serialize_newtype_variant<T: ?Sized>, &'static str, u32, &'static str, &T =>
        Err(Error::unexpected_value("newtype_variant"))
    );
    serialize_err!(serialize_struct_variant, &'static str, u32, &'static str, usize: Result<Self::SerializeStructVariant, Self::Error> =>
        Err(Error::unexpected_value("struct_variant"))
    );
    serialize_err!(serialize_seq, Option<usize>:
        Result<Self::SerializeSeq, Self::Error> =>
        Err(Error::unexpected_value("seq"))
    );
    serialize_err!(serialize_tuple, usize:
        Result<Self::SerializeTuple, Self::Error> =>
        Err(Error::unexpected_value("tuple"))
    );
    serialize_err!(serialize_tuple_struct, &'static str, usize:
        Result<Self::SerializeTupleStruct, Self::Error> =>
        Err(Error::unexpected_value("tuple_struct"))
    );
    serialize_err!(serialize_tuple_variant,
        &'static str, u32, &'static str, usize:
        Result<Self::SerializeTupleStruct, Self::Error> =>
        Err(Error::unexpected_value("tuple_variant"))
    );
    serialize_err!(serialize_map, Option<usize>:
        Result<Self::SerializeMap, Self::Error> =>
        Err(Error::unexpected_value("map"))
    );
    serialize_err!(serialize_struct, &'static str, usize:
        Result<Self::SerializeMap, Self::Error> =>
        Err(Error::unexpected_value("struct"))
    );
    serialize_err!(serialize_newtype_struct<T: ?Sized>, &'static str, &T =>
        Err(Error::unexpected_value("struct"))
    );

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        match self.resp_kind {
            SIMPLE_ERROR_TOKEN => {
                self.se.write_simple_error(v)?;
                Ok(())
            }
            BLOB_ERROR_TOKEN => {
                self.se.write_blob_error(v)?;
                Ok(())
            }
            SIMPLE_STRING_TOKEN => {
                self.se.write_simple_string(v)?;
                Ok(())
            }
            BLOB_STRING_TOKEN => {
                self.se.write_blob_string(v)?;
                Ok(())
            }
            _ => unimplemented!(),
        }
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let s = str::from_utf8(v).map_err(|e| Error::utf8(e.valid_up_to()))?;
        self.serialize_str(s)
    }
}

struct PushSerializer<'a, W: Write> {
    se: &'a mut Serializer<W>,
}

impl<'a, W: Write> serde::Serializer for PushSerializer<'a, W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SeqSerializer<'a, W>;
    type SerializeTuple = SeqSerializer<'a, W>;
    type SerializeTupleStruct = SeqSerializer<'a, W>;
    type SerializeTupleVariant = SeqSerializer<'a, W>;
    type SerializeMap = SeqSerializer<'a, W>;
    type SerializeStruct = SeqSerializer<'a, W>;
    type SerializeStructVariant = SeqSerializer<'a, W>;

    serialize_err!(serialize_bool, bool => Err(Error::unexpected_value("bool")));
    serialize_err!(serialize_i8, i8 => Err(Error::unexpected_value("i8")));
    serialize_err!(serialize_i16, i16 => Err(Error::unexpected_value("i16")));
    serialize_err!(serialize_i32, i32 => Err(Error::unexpected_value("i32")));
    serialize_err!(serialize_i64, i64 => Err(Error::unexpected_value("i64")));
    serialize_err!(serialize_u8, u8 => Err(Error::unexpected_value("u8")));
    serialize_err!(serialize_u16, u16 => Err(Error::unexpected_value("u16")));
    serialize_err!(serialize_u32, u32 => Err(Error::unexpected_value("u32")));
    serialize_err!(serialize_u64, u64 => Err(Error::unexpected_value("u64")));
    serialize_err!(serialize_f32, f32 => Err(Error::unexpected_value("f32")));
    serialize_err!(serialize_f64, f64 => Err(Error::unexpected_value("f64")));
    serialize_err!(serialize_char, char => Err(Error::unexpected_value("char")));
    serialize_err!(serialize_none, => Err(Error::unexpected_value("none")));
    serialize_err!(serialize_unit, => Err(Error::unexpected_value("unit")));
    serialize_err!(serialize_some<T: ?Sized>, &T => Err(Error::unexpected_value("some")));
    serialize_err!(serialize_unit_struct, &'static str => Err(Error::unexpected_value("unit_struct")));
    serialize_err!(serialize_unit_variant, &'static str, u32, &'static str =>
        Err(Error::unexpected_value("unit_variant"))
    );
    serialize_err!(serialize_newtype_variant<T: ?Sized>, &'static str, u32, &'static str, &T =>
        Err(Error::unexpected_value("newtype_variant"))
    );
    serialize_err!(serialize_str, &str => Err(Error::unexpected_value("string")));
    serialize_err!(serialize_bytes, &[u8] => Err(Error::unexpected_value("bytes")));
    serialize_err!(serialize_newtype_struct<T: ?Sized>, &'static str, &T =>
        Err(Error::unexpected_value("newtype_struct"))
    );
    serialize_err!(serialize_map, Option<usize>: Result<Self::SerializeMap, Self::Error> => Err(Error::unexpected_value("map")));

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(len) = len {
            self.serialize_tuple(len)
        } else {
            Err(Error::unexpected_value("unknown len seq"))
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.se.write_push_len_marker(len)?;
        Ok(SeqSerializer::known_length(self.se).without_key())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_tuple(len)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.se.write_push_len_marker(len)?;
        Ok(SeqSerializer::known_length(self.se).without_key())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.serialize_struct(variant, len)
    }
}

impl<W: Write> Serializer<W> {
    fn write_i64(&mut self, v: i64) -> Result<(), Error> {
        write!(self.writer, ":{}\r\n", v).map_err(Error::io)?;

        Ok(())
    }
    fn write_u64(&mut self, v: u64) -> Result<(), Error> {
        write!(self.writer, ":{}\r\n", v).map_err(Error::io)?;

        Ok(())
    }
    fn write_f64(&mut self, v: f64) -> Result<(), Error> {
        if v.is_nan() {
            return Err(Error::nan());
        }

        if v.is_infinite() {
            if v.is_sign_positive() {
                write!(self.writer, ",inf\r\n").map_err(Error::io)?;
            } else {
                write!(self.writer, ",-inf\r\n").map_err(Error::io)?;
            }

            return Ok(());
        }

        write!(self.writer, ",{:.}\r\n", v).map_err(Error::io)?;

        Ok(())
    }
    fn write_bool(&mut self, v: bool) -> Result<(), Error> {
        if v {
            write!(self.writer, "#t\r\n").map_err(Error::io)?;
        } else {
            write!(self.writer, "#f\r\n").map_err(Error::io)?;
        }

        Ok(())
    }
    fn write_simple_string_char(&mut self, c: char) -> Result<(), Error> {
        write!(self.writer, "+{}\r\n", c).map_err(Error::io)?;

        Ok(())
    }
    fn write_simple_string(&mut self, s: &str) -> Result<(), Error> {
        write!(self.writer, "+{}\r\n", s).map_err(Error::io)?;

        Ok(())
    }
    fn write_blob_string(&mut self, s: &str) -> Result<(), Error> {
        write!(self.writer, "${}\r\n{}\r\n", s.len(), s).map_err(Error::io)?;

        Ok(())
    }
    fn write_simple_error(&mut self, s: &str) -> Result<(), Error> {
        write!(self.writer, "-{}\r\n", s).map_err(Error::io)?;

        Ok(())
    }
    fn write_blob_error(&mut self, s: &str) -> Result<(), Error> {
        write!(self.writer, "!{}\r\n{}\r\n", s.len(), s).map_err(Error::io)?;

        Ok(())
    }
    fn write_null(&mut self) -> Result<(), Error> {
        write!(self.writer, "_\r\n").map_err(Error::io)?;

        Ok(())
    }
    fn write_push_len_marker(&mut self, len: usize) -> Result<(), Error> {
        write!(self.writer, ">{}\r\n", len).map_err(Error::io)?;

        Ok(())
    }
    fn write_array_len_marker(&mut self, len: usize) -> Result<(), Error> {
        write!(self.writer, "*{}\r\n", len).map_err(Error::io)?;

        Ok(())
    }
    fn write_array_nolen_marker(&mut self) -> Result<(), Error> {
        write!(self.writer, "*?\r\n").map_err(Error::io)?;

        Ok(())
    }
    fn write_map_len_marker(&mut self, len: usize) -> Result<(), Error> {
        write!(self.writer, "%{}\r\n", len).map_err(Error::io)?;

        Ok(())
    }
    fn write_map_nolen_marker(&mut self) -> Result<(), Error> {
        write!(self.writer, "%?\r\n").map_err(Error::io)?;

        Ok(())
    }
    fn write_end(&mut self) -> Result<(), Error> {
        write!(self.writer, ".\r\n").map_err(Error::io)?;

        Ok(())
    }
}

impl<'a, W: Write> serde::Serializer for &'a mut Serializer<W> {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SeqSerializer<'a, W>;
    type SerializeTuple = SeqSerializer<'a, W>;
    type SerializeTupleStruct = SeqSerializer<'a, W>;
    type SerializeTupleVariant = SeqSerializer<'a, W>;
    type SerializeMap = SeqSerializer<'a, W>;
    type SerializeStruct = SeqSerializer<'a, W>;
    type SerializeStructVariant = SeqSerializer<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.write_bool(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.write_i64(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.write_u64(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.write_f64(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.write_simple_string_char(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.write_simple_string(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let s = str::from_utf8(v).map_err(|e| Error::utf8(e.valid_up_to()))?;
        self.write_blob_string(s)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.write_null()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.write_null()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.write_null()
    }

    /// Serialize as { variant => null }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.write_map_len_marker(1)?;
        self.write_simple_string(variant)?;
        self.write_null()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        match name {
            SIMPLE_ERROR_TOKEN | BLOB_ERROR_TOKEN | SIMPLE_STRING_TOKEN | BLOB_STRING_TOKEN => {
                let se = RespSpecificSerializer {
                    se: self,
                    resp_kind: name,
                };
                value.serialize(se)
            }
            PUSH_TOKEN => {
                let se = PushSerializer { se: self };
                value.serialize(se)
            }
            _ => value.serialize(self),
        }
    }

    /// Serialize as { variant => T }
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        self.write_map_len_marker(1)?;
        self.write_simple_string(variant)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        match len {
            Some(l) => {
                self.write_array_len_marker(l)?;
                Ok(SeqSerializer::known_length(self))
            }
            None => {
                self.write_array_nolen_marker()?;
                Ok(SeqSerializer::unknown_length(self))
            }
        }
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        match name {
            WITH_ATTRIBUTE_TOKEN => Ok(SeqSerializer::known_length(self)),
            _ => self.serialize_seq(Some(len)),
        }
    }

    /// Serialize as { variant => [tuple ele, .. ] }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.write_map_len_marker(1)?;
        self.write_simple_string(variant)?;
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        match len {
            Some(l) => {
                self.write_map_len_marker(l)?;
                Ok(SeqSerializer::known_length(self))
            }
            None => {
                self.write_map_nolen_marker()?;
                Ok(SeqSerializer::unknown_length(self))
            }
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    /// Serialize as { variant => { struct .. } }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.write_map_len_marker(1)?;
        self.write_simple_string(variant)?;
        self.serialize_map(Some(len))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_serialize_bool() {
        let bool_t = true;
        let buf = to_vec(&bool_t).unwrap();
        assert_eq!(buf, b"#t\r\n");

        let bool_t = false;
        let buf = to_vec(&bool_t).unwrap();
        assert_eq!(buf, b"#f\r\n");
    }

    #[test]
    fn test_serialize_number() {
        let num: i64 = 12345;
        let buf = to_vec(&num).unwrap();
        assert_eq!(buf, b":12345\r\n");

        let num: i64 = -12345;
        let buf = to_vec(&num).unwrap();
        assert_eq!(buf, b":-12345\r\n");
    }

    #[test]
    fn test_serialize_double() {
        let num: f64 = 12345.1;
        let buf = to_vec(&num).unwrap();
        assert_eq!(buf, b",12345.1\r\n");

        let num: f64 = f64::NEG_INFINITY;
        let buf = to_vec(&num).unwrap();
        assert_eq!(buf, b",-inf\r\n");

        let num: f64 = f64::INFINITY;
        let buf = to_vec(&num).unwrap();
        assert_eq!(buf, b",inf\r\n");
    }

    #[test]
    fn test_serialize_char() {
        let chr: char = 'e';
        let buf = to_vec(&chr).unwrap();
        assert_eq!(buf, b"+e\r\n");
    }

    #[test]
    fn test_serialize_str() {
        let str: &str = "hello world";
        let buf = to_vec(&str).unwrap();
        assert_eq!(buf, b"+hello world\r\n");
    }

    #[test]
    fn test_serialize_option() {
        let str: Option<&str> = None;
        let buf = to_vec(&str).unwrap();
        assert_eq!(buf, b"_\r\n");

        let str: Option<&str> = Some("hello world");
        let buf = to_vec(&str).unwrap();
        assert_eq!(buf, b"+hello world\r\n");
    }

    #[test]
    fn test_serialize_unit() {
        let unit: () = ();
        let buf = to_vec(&unit).unwrap();
        assert_eq!(buf, b"_\r\n");
    }

    #[test]
    fn test_serialize_struct() {
        // newtype struct
        #[derive(Serialize)]
        struct NewType(usize);

        let newtype = NewType(123);
        let buf = to_vec(&newtype).unwrap();
        assert_eq!(buf, b":123\r\n");

        // struct struct
        #[derive(Serialize)]
        struct StructStruct {
            a: usize,
            b: String,
        }

        let structstruct = StructStruct {
            a: 123,
            b: String::from("abc"),
        };
        let buf = to_vec(&structstruct).unwrap();
        assert_eq!(buf, b"%2\r\n+a\r\n:123\r\n+b\r\n+abc\r\n");

        // unit struct
        #[derive(Serialize)]
        struct UnitT;

        let unit: UnitT = UnitT;
        let buf = to_vec(&unit).unwrap();
        assert_eq!(buf, b"_\r\n");

        // tuple struct
        #[derive(Serialize)]
        struct Tuple(usize, String);

        let tuple = Tuple(123, String::from("abcd"));
        let buf = to_vec(&tuple).unwrap();
        assert_eq!(buf, b"*2\r\n:123\r\n+abcd\r\n");
    }

    #[test]
    fn test_serialize_map() {
        let mut map = BTreeMap::new();
        map.insert("a", "b");
        map.insert("c", "d");
        let buf = to_vec(&map).unwrap();
        assert_eq!(buf, b"%2\r\n+a\r\n+b\r\n+c\r\n+d\r\n");
    }

    #[test]
    fn test_serialize_seq() {
        let seq = vec!["a", "b", "c", "d"];
        let buf = to_vec(&seq).unwrap();
        assert_eq!(buf, b"*4\r\n+a\r\n+b\r\n+c\r\n+d\r\n");

        let seq = (1, 3, String::from("abc"), 10.5);
        let buf = to_vec(&seq).unwrap();
        assert_eq!(buf, b"*4\r\n:1\r\n:3\r\n+abc\r\n,10.5\r\n");
    }

    #[test]
    fn test_serialize_enum() {
        #[derive(Serialize)]
        enum Enum {
            Struct { a: usize, b: String },
            Tuple(usize, String),
            Unit,
        }

        // struct variant
        let struct_variant = Enum::Struct {
            a: 123,
            b: String::from("abc"),
        };
        let buf = to_vec(&struct_variant).unwrap();
        assert_eq!(buf, b"%1\r\n+Struct\r\n%2\r\n+a\r\n:123\r\n+b\r\n+abc\r\n");

        // tuple variant
        let tuple_variant = Enum::Tuple(123, String::from("abcd"));
        let buf = to_vec(&tuple_variant).unwrap();
        assert_eq!(buf, b"%1\r\n+Tuple\r\n*2\r\n:123\r\n+abcd\r\n");

        // unit variant
        let unit_variant = Enum::Unit;
        let buf = to_vec(&unit_variant).unwrap();
        assert_eq!(buf, b"%1\r\n+Unit\r\n_\r\n");
    }
}
