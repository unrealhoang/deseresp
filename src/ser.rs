use std::io::Write;
use std::str;

use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize,
};

use crate::Error;

pub struct Serializer<W> {
    writer: W,
}

impl<W: Write> Serializer<W> {
    pub fn from_writer(w: W) -> Self {
        Serializer { writer: w }
    }
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
            SeqKind::UnknownLength => {
                self.se.write_end()
            }
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
            SeqKind::UnknownLength => {
                self.se.write_end()
            }
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
            SeqKind::UnknownLength => {
                self.se.write_end()
            }
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
            SeqKind::UnknownLength => {
                self.se.write_end()
            }
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
            SeqKind::UnknownLength => {
                self.se.write_end()
            }
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
            SeqKind::UnknownLength => {
                self.se.write_end()
            }
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
            SeqKind::UnknownLength => {
                self.se.write_end()
            }
            SeqKind::KnownLength => Ok(()),
        }
    }
}

impl<W: Write> Serializer<W> {
    fn write_i64(&mut self, v: i64) -> Result<(), Error> {
        write!(self.writer, ":{}\r\n", v).map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_u64(&mut self, v: u64) -> Result<(), Error> {
        write!(self.writer, ":{}\r\n", v).map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_f64(&mut self, v: f64) -> Result<(), Error> {
        if v.is_nan() {
            return Err(Error::nan());
        }

        if v.is_infinite() {
            if v.is_sign_positive() {
                write!(self.writer, ",inf\r\n").map_err(|e| Error::io(e))?;
            } else {
                write!(self.writer, ",-inf\r\n").map_err(|e| Error::io(e))?;
            }

            return Ok(());
        }

        write!(self.writer, ",{:.}\r\n", v).map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_bool(&mut self, v: bool) -> Result<(), Error> {
        if v {
            write!(self.writer, "#t\r\n").map_err(|e| Error::io(e))?;
        } else {
            write!(self.writer, "#f\r\n").map_err(|e| Error::io(e))?;
        }

        Ok(())
    }
    fn write_simple_string_char(&mut self, c: char) -> Result<(), Error> {
        write!(self.writer, "+{}\r\n", c).map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_simple_string(&mut self, s: &str) -> Result<(), Error> {
        write!(self.writer, "+{}\r\n", s).map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_blob_string(&mut self, s: &str) -> Result<(), Error> {
        write!(self.writer, "${}\r\n{}\r\n", s.len(), s).map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_null(&mut self) -> Result<(), Error> {
        write!(self.writer, "_\r\n").map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_array_len_marker(&mut self, len: usize) -> Result<(), Error> {
        write!(self.writer, "*{}\r\n", len).map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_array_nolen_marker(&mut self) -> Result<(), Error> {
        write!(self.writer, "*?\r\n").map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_map_len_marker(&mut self, len: usize) -> Result<(), Error> {
        write!(self.writer, "%{}\r\n", len).map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_map_nolen_marker(&mut self) -> Result<(), Error> {
        write!(self.writer, "%?\r\n").map_err(|e| Error::io(e))?;

        Ok(())
    }
    fn write_end(&mut self) -> Result<(), Error> {
        write!(self.writer, ".\r\n").map_err(|e| Error::io(e))?;

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
        self.write_blob_string(v)
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
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
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
    use super::*;

    fn test_serialize<F, T>(serialize_fn: F, test_fn: T)
    where
        F: Fn(Serializer<&mut Vec<u8>>),
        T: Fn(&[u8]),
    {
        let mut buf = Vec::new();
        {
            let serializer = Serializer::from_writer(&mut buf);
            serialize_fn(serializer);
        }
        test_fn(&buf[..]);
    }

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
                assert_eq!(buf, b"$11\r\nhello world\r\n");
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
                assert_eq!(buf, b"$11\r\nhello world\r\n");
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
    }

    #[test]
    fn test_serialize_newtype_struct() {
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
    }
}
