use std::{io::Write, str};

use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize,
};

use crate::{
    types::{BLOB_ERROR_TOKEN, BLOB_STRING_TOKEN, SIMPLE_ERROR_TOKEN, SIMPLE_STRING_TOKEN},
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
}

impl<'a, W> SeqSerializer<'a, W> {
    fn known_length(se: &'a mut Serializer<W>) -> Self {
        SeqSerializer {
            se,
            kind: SeqKind::KnownLength,
        }
    }

    fn unknown_length(se: &'a mut Serializer<W>) -> Self {
        SeqSerializer {
            se,
            kind: SeqKind::UnknownLength,
        }
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
        key.serialize(&mut *self.se)?;
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
        key.serialize(&mut *self.se)?;
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self.kind {
            SeqKind::UnknownLength => self.se.write_end(),
            SeqKind::KnownLength => Ok(()),
        }
    }
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

    fn serialize_bool(self, _: bool) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("bool"))
    }

    fn serialize_i8(self, _: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("i8"))
    }

    fn serialize_i16(self, _: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("i16"))
    }

    fn serialize_i32(self, _: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("i32"))
    }

    fn serialize_i64(self, _: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("i64"))
    }

    fn serialize_u8(self, _: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("u8"))
    }

    fn serialize_u16(self, _: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("u16"))
    }

    fn serialize_u32(self, _: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("u32"))
    }

    fn serialize_u64(self, _: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("u64"))
    }

    fn serialize_f32(self, _: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("f32"))
    }

    fn serialize_f64(self, _: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("f64"))
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("char"))
    }

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

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("none"))
    }

    fn serialize_some<T: ?Sized>(self, _: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::unexpected_value("some"))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("unit"))
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("unit_struct"))
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Error::unexpected_value("unit_variant"))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::unexpected_value("newtype_struct"))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::unexpected_value("newtype_variant"))
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::unexpected_value("seq"))
    }

    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::unexpected_value("tuple"))
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::unexpected_value("tuple_struct"))
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::unexpected_value("tuple_variant"))
    }

    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::unexpected_value("map"))
    }

    fn serialize_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::unexpected_value("struct"))
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::unexpected_value("struct_variant"))
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
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
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
    use crate::{
        test_utils::test_serialize,
        types::owned::{BlobError, BlobString, SimpleError, SimpleString},
    };

    #[test]
    fn test_serialize_bool() {
        test_serialize(
            |mut s| {
                let bool_t = true;
                bool_t.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"#t\r\n");
            },
        );

        test_serialize(
            |mut s| {
                let bool_t = false;
                bool_t.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"#f\r\n");
            },
        );
    }

    #[test]
    fn test_serialize_number() {
        test_serialize(
            |mut s| {
                let num: i64 = 12345;
                num.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b":12345\r\n");
            },
        );

        test_serialize(
            |mut s| {
                let num: i64 = -12345;
                num.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b":-12345\r\n");
            },
        );
    }

    #[test]
    fn test_serialize_double() {
        test_serialize(
            |mut s| {
                let num: f64 = 12345.1;
                num.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b",12345.1\r\n");
            },
        );

        test_serialize(
            |mut s| {
                let num: f64 = f64::NEG_INFINITY;
                num.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b",-inf\r\n");
            },
        );

        test_serialize(
            |mut s| {
                let num: f64 = f64::INFINITY;
                num.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b",inf\r\n");
            },
        );
    }

    #[test]
    fn test_serialize_char() {
        test_serialize(
            |mut s| {
                let chr: char = 'e';
                chr.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"+e\r\n");
            },
        );
    }

    #[test]
    fn test_serialize_str() {
        test_serialize(
            |mut s| {
                let str: &str = "hello world";
                str.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"+hello world\r\n");
            },
        );
    }

    #[test]
    fn test_serialize_option() {
        test_serialize(
            |mut s| {
                let str: Option<&str> = None;
                str.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"_\r\n");
            },
        );

        test_serialize(
            |mut s| {
                let str: Option<&str> = Some("hello world");
                str.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"+hello world\r\n");
            },
        );
    }

    #[test]
    fn test_serialize_unit() {
        test_serialize(
            |mut s| {
                let unit: () = ();
                unit.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"_\r\n");
            },
        );
    }

    #[test]
    fn test_serialize_struct() {
        // newtype struct
        test_serialize(
            |mut s| {
                #[derive(Serialize)]
                struct NewType(usize);

                let newtype = NewType(123);
                newtype.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b":123\r\n");
            },
        );

        // struct struct
        test_serialize(
            |mut s| {
                #[derive(Serialize)]
                struct StructStruct {
                    a: usize,
                    b: String,
                }

                let structstruct = StructStruct {
                    a: 123,
                    b: String::from("abc"),
                };
                structstruct.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"%2\r\n+a\r\n:123\r\n+b\r\n+abc\r\n");
            },
        );

        // unit struct
        test_serialize(
            |mut s| {
                #[derive(Serialize)]
                struct UnitT;
                let unit: UnitT = UnitT;
                unit.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"_\r\n");
            },
        );

        // tuple struct
        test_serialize(
            |mut s| {
                #[derive(Serialize)]
                struct Tuple(usize, String);

                let tuple = Tuple(123, String::from("abcd"));
                tuple.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"*2\r\n:123\r\n+abcd\r\n");
            },
        );
    }

    #[test]
    fn test_serialize_map() {
        test_serialize(
            |mut s| {
                let mut map = BTreeMap::new();
                map.insert("a", "b");
                map.insert("c", "d");
                map.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"%2\r\n+a\r\n+b\r\n+c\r\n+d\r\n");
            },
        );
    }

    #[test]
    fn test_serialize_seq() {
        test_serialize(
            |mut s| {
                let seq = vec!["a", "b", "c", "d"];
                seq.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"*4\r\n+a\r\n+b\r\n+c\r\n+d\r\n");
            },
        );

        test_serialize(
            |mut s| {
                let seq = (1, 3, String::from("abc"), 10.5);
                seq.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"*4\r\n:1\r\n:3\r\n+abc\r\n,10.5\r\n");
            },
        );
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
        test_serialize(
            |mut s| {
                let struct_variant = Enum::Struct {
                    a: 123,
                    b: String::from("abc"),
                };
                struct_variant.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"%1\r\n+Struct\r\n%2\r\n+a\r\n:123\r\n+b\r\n+abc\r\n");
            },
        );

        // tuple variant
        test_serialize(
            |mut s| {
                let tuple_variant = Enum::Tuple(123, String::from("abcd"));
                tuple_variant.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"%1\r\n+Tuple\r\n*2\r\n:123\r\n+abcd\r\n");
            },
        );

        // unit variant
        test_serialize(
            |mut s| {
                let unit_variant = Enum::Unit;
                unit_variant.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"%1\r\n+Unit\r\n_\r\n");
            },
        );
    }

    #[test]
    fn test_specific_resp_type() {
        // simple error
        test_serialize(
            |mut s| {
                let value = SimpleError(String::from("ERR error"));
                value.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"-ERR error\r\n");
            },
        );

        // blob error
        test_serialize(
            |mut s| {
                let value = BlobError(String::from("ERR error"));
                value.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"!9\r\nERR error\r\n");
            },
        );

        // simple string
        test_serialize(
            |mut s| {
                let value = SimpleString(String::from("hello world"));
                value.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"+hello world\r\n");
            },
        );

        // blob string
        test_serialize(
            |mut s| {
                let value = BlobString(String::from("hello world"));
                value.serialize(&mut s).unwrap();
            },
            |buf| {
                assert_eq!(buf, b"$11\r\nhello world\r\n");
            },
        );
    }
}
