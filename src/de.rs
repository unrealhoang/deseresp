use std::{
    io::{self, Read},
    str,
};

use num::{CheckedAdd, CheckedMul};
use serde::Deserialize;

use crate::{types::AttributeSkip, Error, Result};

pub enum Reference<'b, 'c, T: ?Sized + 'static> {
    Borrowed(&'b T),
    Copied(&'c T),
}

pub trait Reader<'de> {
    fn read_slice<'a>(
        &'a mut self,
        len: usize,
        consume_crlf: bool,
    ) -> Result<Reference<'de, 'a, [u8]>>;
    fn read_slice_until<'a, F>(
        &'a mut self,
        until_fn: F,
        consume_crlf: bool,
    ) -> Result<Reference<'de, 'a, [u8]>>
    where
        F: Fn(u8) -> bool;

    fn peek_u8(&mut self) -> Result<Option<u8>>;
    fn read_u8(&mut self) -> Result<Option<u8>>;

    fn read_length(&mut self) -> Result<usize> {
        self.read_unsigned()
    }

    fn read_unsigned<T>(&mut self) -> Result<T>
    where
        T: CheckedMul + CheckedAdd + From<u8>,
    {
        let peek = self.peek_u8()?.ok_or_else(|| Error::eof())?;
        match peek {
            b'0' => {
                self.read_u8()?;
                match self.peek_u8()? {
                    Some(b'0'..=b'9') => Err(Error::unexpected_value("number after 0")),
                    _ => Ok(T::from(0)),
                }
            }
            ch @ b'1'..=b'9' => {
                self.read_u8()?;
                let mut num = T::from(ch - b'0');
                loop {
                    match self.peek_u8()? {
                        Some(c @ b'0'..=b'9') => {
                            let digit = T::from(c - b'0');
                            let ten = T::from(10);
                            if let Some(r) = num
                                .checked_mul(&ten)
                                .map(|n| n.checked_add(&digit))
                                .flatten()
                            {
                                num = r;
                            } else {
                                return Err(Error::overflow());
                            }
                            self.read_u8()?;
                        }
                        _ => {
                            return Ok(num);
                        }
                    }
                }
            }
            _ => Err(Error::expected_value("number")),
        }
    }

    fn read_double(&mut self) -> Result<f64> {
        let mut buf = Vec::new();
        let mut negative = false;
        let mut inf = false;
        if let Some(b'-') = self.peek_u8()? {
            negative = true;
            self.read_u8()?;
        }
        if let Some(b'i') = self.peek_u8()? {
            self.read_ident(b"inf")?;
            inf = true;
        }
        if inf {
            if negative {
                return Ok(f64::NEG_INFINITY);
            } else {
                return Ok(f64::INFINITY);
            }
        }

        loop {
            match self.peek_u8()? {
                Some(ch) if ch != b'\r' && ch != b'\n' => {
                    self.read_u8()?;
                    buf.push(ch);
                }
                None => return Err(Error::eof()),
                _ => break,
            }
        }
        let str = str::from_utf8(&buf[..]).map_err(|e| Error::utf8(e.valid_up_to()))?;
        let result = str.parse::<f64>().map_err(|_e| Error::parse())?;

        Ok(result)
    }

    fn read_bool(&mut self) -> Result<bool> {
        match self.peek_u8()? {
            Some(b't') => {
                self.read_u8()?;
                self.read_crlf()?;
                Ok(true)
            }
            Some(b'f') => {
                self.read_u8()?;
                self.read_crlf()?;
                Ok(false)
            }
            _ => Err(Error::expected_value("bool")),
        }
    }

    fn read_ident(&mut self, ident: &[u8]) -> Result<()>;

    fn read_crlf(&mut self) -> Result<()> {
        self.read_ident(b"\r\n")
    }
}

pub struct ReadReader<R: Read> {
    r: io::Bytes<R>,
    ch: Option<u8>,
    buf: Vec<u8>,
}

fn peek_u8<R: Read>(r: &mut io::Bytes<R>, ch: &mut Option<u8>) -> Result<Option<u8>> {
    match ch {
        Some(next) => Ok(Some(*next)),
        None => read_u8(r, ch),
    }
}

fn read_u8<R: Read>(r: &mut io::Bytes<R>, ch: &mut Option<u8>) -> Result<Option<u8>> {
    r
        .next()
        .transpose()
        .map_err(|e| Error::io(e))
        .map(|next| {
            *ch = next;
            next
        })
}

fn read_ident<R: Read>(r: &mut io::Bytes<R>, ch: &mut Option<u8>, ident: &[u8]) -> Result<()> {
    for expected in ident {
        match peek_u8(r, ch)? {
            None => return Err(Error::eof()),
            Some(next) => {
                if next != *expected {
                    return Err(Error::expected_value("ident"));
                }
                read_u8(r, ch)?;
            }
        }
    }

    Ok(())
}

impl<'de, R: Read> Reader<'de> for ReadReader<R> {
    fn read_slice<'a>(
        &'a mut self,
        len: usize,
        consume_crlf: bool,
    ) -> Result<Reference<'de, 'a, [u8]>> {
        self.buf.clear();
        for _count in 0..len {
            let ch = peek_u8(&mut self.r, &mut self.ch)?
                .ok_or_else(|| Error::eof())?;
            self.buf.push(ch);
            read_u8(&mut self.r, &mut self.ch)?;
        }

        if consume_crlf {
            read_ident(&mut self.r, &mut self.ch, b"\r\n")?;
        }

        Ok(Reference::Copied(&self.buf[..]))
    }

    fn read_slice_until<'a, F>(
        &'a mut self,
        until_fn: F,
        consume_crlf: bool,
    ) -> Result<Reference<'de, 'a, [u8]>>
    where
        F: Fn(u8) -> bool
    {
        self.buf.clear();
        loop {
            let ch = peek_u8(&mut self.r, &mut self.ch)?
                .ok_or_else(|| Error::eof())?;
            if until_fn(ch) {
                break
            }
            self.buf.push(ch);
            read_u8(&mut self.r, &mut self.ch)?;
        }

        if consume_crlf {
            read_ident(&mut self.r, &mut self.ch, b"\r\n")?;
        }

        Ok(Reference::Copied(&self.buf[..]))
    }

    fn peek_u8(&mut self) -> Result<Option<u8>> {
        peek_u8(&mut self.r, &mut self.ch)
    }

    fn read_u8(&mut self) -> Result<Option<u8>> {
        read_u8(&mut self.r, &mut self.ch)
    }

    fn read_ident(&mut self, ident: &[u8]) -> Result<()> {
        read_ident(&mut self.r, &mut self.ch, ident)
    }
}

pub struct Deserializer<R>
{
    reader: R,
    skip_attribute: bool,
}

impl<R> ReadReader<R>
where
    R: Read,
{
    fn from_read(r: R) -> Self {
        ReadReader {
            r: r.bytes(),
            ch: None,
            buf: Vec::new(),
        }
    }
}

impl<R: Read> Deserializer<ReadReader<R>> {
    pub fn from_read(r: R) -> Self {
        Deserializer {
            reader: ReadReader::from_read(r),
            skip_attribute: true,
        }
    }
}

impl<'de, R: Reader<'de>> Deserializer<R> {
    fn parse_blob_string<'a>(&'a mut self) -> Result<Reference<'de, 'a, [u8]>> {
        let len = self.reader.read_length()?;
        self.reader.read_crlf()?;

        let slice = self.reader
            .read_slice(len, true)?;

        Ok(slice)
    }

    fn parse_simple_string<'a>(&'a mut self) -> Result<Reference<'de, 'a, [u8]>> {
        let slice = self.reader
            .read_slice_until(|ch| ch == b'\r' || ch == b'\n', true)?;

        Ok(slice)
    }

    fn parse_double(&mut self) -> Result<f64> {
        let val = self.reader.read_double()?;
        self.reader.read_crlf()?;

        Ok(val)
    }

    fn skip_attribute(&mut self) -> Result<()> {
        let _s: AttributeSkip = Deserialize::deserialize(self)?;

        Ok(())
    }

    fn peek_skip_attribute(&mut self) -> Result<u8> {
        let peek = self.reader.peek_u8()?.ok_or_else(|| Error::eof())?;

        if peek == b'|' && self.skip_attribute {
            self.skip_attribute()?;
            return self.reader.peek_u8()?.ok_or_else(|| Error::eof());
        }

        Ok(peek)
    }
}

fn visit_ref_bytes<'de, 'a, V>(r: Reference<'de, 'a, [u8]>, visitor: V) -> Result<V::Value>
where
    V: serde::de::Visitor<'de>
{
    match r {
        Reference::Copied(s) => visitor.visit_bytes(s),
        Reference::Borrowed(s) => visitor.visit_borrowed_bytes(s),
    }
}

fn visit_ref_str<'de, 'a, V>(r: Reference<'de, 'a, [u8]>, visitor: V) -> Result<V::Value>
where
    V: serde::de::Visitor<'de>
{
    match r {
        Reference::Copied(s) => {
            let string = str::from_utf8(s).map_err(|e| Error::utf8(e.valid_up_to()))?;
            visitor.visit_str(string)
        }
        Reference::Borrowed(s) => {
            let string = str::from_utf8(s).map_err(|e| Error::utf8(e.valid_up_to()))?;
            visitor.visit_borrowed_str(string)
        }
    }
}

impl<'de, 'a, R> serde::Deserializer<'de> for &'a mut Deserializer<R>
where
    R: Reader<'de>,
{
    type Error = super::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            // blob string
            b'$' => self.deserialize_str(visitor),
            // verbatim string
            b'=' => self.deserialize_str(visitor),
            // blob error
            b'!' => self.deserialize_str(visitor),
            // simple string
            b'+' => self.deserialize_str(visitor),
            // simple error
            b'-' => self.deserialize_str(visitor),
            // boolean
            b'#' => self.deserialize_bool(visitor),
            // number
            b':' => self.deserialize_i64(visitor),
            // floating point
            b',' => self.deserialize_f64(visitor),
            // array
            b'*' => self.deserialize_seq(visitor),
            b'~' => self.deserialize_seq(visitor),
            // map
            b'%' => self.deserialize_map(visitor),
            b'|' => self.deserialize_map(visitor),
            _ => Err(Error::expected_value("type header")),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            b'#' => {
                self.reader.read_u8()?;
                let val = self.reader.read_bool()?;
                visitor.visit_bool(val)
            }
            _ => Err(Error::expected_marker(&"bool")),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            b':' => {
                self.reader.read_u8()?;
                match self.reader.peek_u8()? {
                    Some(b'-') => {
                        self.reader.read_u8()?;
                        let num: i64 = self.reader.read_unsigned()?;
                        self.reader.read_crlf()?;
                        visitor.visit_i64(-num)
                    }
                    Some(b'0'..=b'9') => {
                        let num: i64 = self.reader.read_unsigned()?;
                        self.reader.read_crlf()?;
                        visitor.visit_i64(num)
                    }
                    _ => Err(Error::expected_value("number")),
                }
            }
            _ => Err(Error::expected_marker("number")),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            b':' => {
                self.reader.read_u8()?;
                match self.reader.peek_u8()? {
                    Some(b'-') => Err(Error::unexpected_value("signed")),
                    Some(b'0'..=b'9') => {
                        let num: u64 = self.reader.read_unsigned()?;
                        self.reader.read_crlf()?;
                        visitor.visit_u64(num)
                    }
                    _ => Err(Error::expected_value("number")),
                }
            }
            _ => Err(Error::expected_marker("number")),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            b':' => {
                self.reader.read_u8()?;
                match self.reader.peek_u8()? {
                    Some(b'-') => {
                        self.reader.read_u8()?;
                        let num: i64 = self.reader.read_unsigned()?;
                        self.reader.read_crlf()?;
                        visitor.visit_f64(-num as f64)
                    }
                    Some(b'0'..=b'9') => {
                        let num: i64 = self.reader.read_unsigned()?;
                        self.reader.read_crlf()?;
                        visitor.visit_f64(num as f64)
                    }
                    _ => Err(Error::expected_value("number")),
                }
            }
            b',' => {
                self.reader.read_u8()?;
                let num = self.parse_double()?;
                visitor.visit_f64(num)
            }
            _ => Err(Error::expected_marker("number|double")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            b'+' => {
                self.reader.read_u8()?;
                let bytes = self.parse_simple_string()?;
                visit_ref_str(bytes, visitor)
            }
            b'-' => {
                self.reader.read_u8()?;
                let bytes = self.parse_simple_string()?;
                visit_ref_str(bytes, visitor)
            }
            b'$' => {
                self.reader.read_u8()?;
                let bytes = self.parse_blob_string()?;
                visit_ref_str(bytes, visitor)
            }
            b'!' => {
                self.reader.read_u8()?;
                let bytes = self.parse_blob_string()?;
                visit_ref_str(bytes, visitor)
            }
            b'=' => {
                self.reader.read_u8()?;
                let bytes = self.parse_blob_string()?;
                visit_ref_str(bytes, visitor)
            }
            _ => Err(Error::expected_marker("string|error")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            b'+' => {
                self.reader.read_u8()?;
                let bytes = self.parse_simple_string()?;
                visit_ref_bytes(bytes, visitor)
            }
            b'-' => {
                self.reader.read_u8()?;
                let bytes = self.parse_simple_string()?;
                visit_ref_bytes(bytes, visitor)
            }
            b'$' => {
                self.reader.read_u8()?;
                let bytes = self.parse_blob_string()?;
                visit_ref_bytes(bytes, visitor)
            }
            b'!' => {
                self.reader.read_u8()?;
                let bytes = self.parse_blob_string()?;
                visit_ref_bytes(bytes, visitor)
            }
            b'=' => {
                self.reader.read_u8()?;
                let bytes = self.parse_blob_string()?;
                visit_ref_bytes(bytes, visitor)
            }
            _ => Err(Error::expected_marker("string|error")),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            // "_\r\n" => null
            b'_' => {
                self.reader.read_u8()?;
                self.reader.read_crlf()?;
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            b'_' => {
                self.reader.read_u8()?;
                self.reader.read_crlf()?;
                visitor.visit_unit()
            }
            _ => Err(Error::expected_marker("null")),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.reader.peek_u8()?.ok_or_else(|| Error::eof())?;

        match name {
            crate::types::SIMPLE_ERROR_TOKEN => {
                if peek == b'-' {
                    self.reader.read_u8()?;
                    let bytes = self.parse_simple_string()?;
                    visit_ref_str(bytes, visitor)
                } else {
                    Err(Error::expected_marker("simple error"))
                }
            }
            crate::types::BLOB_ERROR_TOKEN => {
                if peek == b'!' {
                    self.reader.read_u8()?;
                    let bytes = self.parse_blob_string()?;
                    visit_ref_str(bytes, visitor)
                } else {
                    Err(Error::expected_marker("blob error"))
                }
            }
            crate::types::SIMPLE_STRING_TOKEN => {
                if peek == b'+' {
                    self.reader.read_u8()?;
                    let bytes = self.parse_simple_string()?;
                    visit_ref_str(bytes, visitor)
                } else {
                    Err(Error::expected_marker("simple string"))
                }
            }
            crate::types::BLOB_STRING_TOKEN => {
                if peek == b'$' {
                    self.reader.read_u8()?;
                    let bytes = self.parse_blob_string()?;
                    visit_ref_str(bytes, visitor)
                } else {
                    Err(Error::expected_marker("blob string"))
                }
            }
            crate::types::ATTRIBUTE_SKIP_TOKEN => {
                if peek == b'|' {
                    self.reader.read_u8()?;
                    let len = self.reader.read_length()?;
                    self.reader.read_crlf()?;
                    visitor.visit_map(CountMapAccess::new(self, len))
                } else {
                    Err(Error::expected_marker("blob string"))
                }
            }
            _ => visitor.visit_newtype_struct(self),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            b'*' => {
                self.reader.read_u8()?;
                let len = self.reader.read_length()?;
                self.reader.read_crlf()?;
                visitor.visit_seq(CountSeqAccess::new(self, len))
            }
            b'~' => {
                self.reader.read_u8()?;
                let len = self.reader.read_length()?;
                self.reader.read_crlf()?;
                visitor.visit_seq(CountSeqAccess::new(self, len))
            }
            _ => Err(Error::expected_marker("array|set")),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.reader.peek_u8()?.ok_or_else(|| Error::eof())?;

        match name {
            crate::types::ATTRIBUTE_TOKEN => {
                if peek == b'|' {
                    let last_skip = self.skip_attribute;
                    self.skip_attribute = false;
                    let r = visitor.visit_seq(CountSeqAccess::new(self, 2));
                    self.skip_attribute = last_skip;
                    r
                } else {
                    Err(Error::expected_marker("attribute"))
                }
            }
            _ => self.deserialize_seq(visitor),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.peek_skip_attribute()?;

        match peek {
            b'%' => {
                self.reader.read_u8()?;
                let len = self.reader.read_length()?;
                self.reader.read_crlf()?;
                visitor.visit_map(CountMapAccess::new(self, len))
            }
            b'|' => {
                self.reader.read_u8()?;
                let len = self.reader.read_length()?;
                self.reader.read_crlf()?;
                let last_skip = self.skip_attribute;
                self.skip_attribute = true;
                let r = visitor.visit_map(CountMapAccess::new(self, len));
                self.skip_attribute = last_skip;
                r
            }
            _ => Err(Error::expected_marker("map")),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}
struct CountSeqAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    len: usize,
}

impl<'a, R> CountSeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: usize) -> Self {
        CountSeqAccess { de, len }
    }
}

impl<'de, 'a, R: Reader<'de> + 'a> serde::de::SeqAccess<'de> for CountSeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.len > 0 {
            let result = seed.deserialize(&mut *self.de).map(|r| Some(r));
            self.len -= 1;

            result
        } else {
            Ok(None)
        }
    }
}

struct CountMapAccess<'a, R> {
    de: &'a mut Deserializer<R>,
    len: usize,
}

impl<'a, R> CountMapAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: usize) -> Self {
        CountMapAccess { de, len }
    }
}

impl<'de, 'a, R: Reader<'de> + 'a> serde::de::MapAccess<'de> for CountMapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.len > 0 {
            let key = seed.deserialize(&mut *self.de).map(|r| Some(r));
            self.len -= 1;
            key
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, io::Cursor};

    use serde::Deserialize;

    use crate::types::{
        owned::{BlobError, BlobString, SimpleError, SimpleString},
        WithAttribute,
    };

    use super::*;

    #[test]
    fn test_blob_string() {
        let mut d = Deserializer::from_read(Cursor::new(String::from("$11\r\nhello world\r\n")));
        let value: String = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, "hello world");

        let mut d = Deserializer::from_read(Cursor::new(String::from("$11\r\nhello world\r\n")));
        let value: BlobString = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value.0, "hello world");

        let mut d = Deserializer::from_read(Cursor::new(String::from("+hello world\r\n")));
        let value: Result<BlobString> = Deserialize::deserialize(&mut d);
        assert!(matches!(value, Err(_)));
    }

    #[test]
    fn test_simple_string() {
        let mut d = Deserializer::from_read(Cursor::new(String::from("+hello world\r\n")));
        let value: String = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, "hello world");

        let mut d = Deserializer::from_read(Cursor::new(String::from("+hello world\r\n")));
        let value: SimpleString = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value.0, "hello world");
    }

    #[test]
    fn test_blob_error() {
        let mut d =
            Deserializer::from_read(Cursor::new(String::from("!15\r\nERR hello world\r\n")));
        let value: BlobError = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value.0, "ERR hello world");
    }

    #[test]
    fn test_simple_error() {
        let mut d = Deserializer::from_read(Cursor::new(String::from("-ERR hello world\r\n")));
        let value: SimpleError = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value.0, "ERR hello world");
    }

    #[test]
    fn test_bool() {
        let mut d = Deserializer::from_read(Cursor::new(String::from("#t\r\n")));
        let value: bool = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, true);

        let mut d = Deserializer::from_read(Cursor::new(String::from("#f\r\n")));
        let value: bool = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, false);
    }

    #[test]
    fn test_number() {
        let mut d = Deserializer::from_read(Cursor::new(String::from(":12345\r\n")));
        let value: i64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, 12345);

        let mut d = Deserializer::from_read(Cursor::new(String::from(":-12345\r\n")));
        let value: i64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, -12345);
    }

    #[test]
    fn test_double() {
        let mut d = Deserializer::from_read(Cursor::new(String::from(",1.23\r\n")));
        let value: f64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, 1.23);

        let mut d = Deserializer::from_read(Cursor::new(String::from(",10\r\n")));
        let value: f64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, 10.0);

        let mut d = Deserializer::from_read(Cursor::new(String::from(",inf\r\n")));
        let value: f64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, f64::INFINITY);

        let mut d = Deserializer::from_read(Cursor::new(String::from(",-inf\r\n")));
        let value: f64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, f64::NEG_INFINITY);
    }

    #[test]
    fn test_char() {
        let mut d = Deserializer::from_read(Cursor::new(String::from("+a\r\n")));
        let value: char = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, 'a');
    }

    #[test]
    fn test_seq() {
        let mut d =
            Deserializer::from_read(Cursor::new(String::from("*3\r\n:1\r\n:2\r\n:3\r\n")));
        let value: Vec<u64> = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, [1, 2, 3]);

        let mut d = Deserializer::from_read(Cursor::new(String::from(
            "*2\r\n*3\r\n:1\r\n$5\r\nhello\r\n:2\r\n#f\r\n",
        )));
        let value: ((u64, String, u64), bool) = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, ((1, String::from("hello"), 2), false));
    }

    #[test]
    fn test_map() {
        let mut d = Deserializer::from_read(Cursor::new(String::from(
            "%2\r\n+first\r\n:1\r\n+second\r\n:2\r\n",
        )));
        let value: HashMap<String, usize> = Deserialize::deserialize(&mut d).unwrap();
        let kv = value.into_iter().collect::<Vec<_>>();
        assert!(kv.contains(&("first".to_string(), 1)));
        assert!(kv.contains(&("second".to_string(), 2)));

        #[derive(PartialEq, Deserialize, Debug)]
        struct CustomMap {
            first: usize,
            second: f64,
        }
        let mut d = Deserializer::from_read(Cursor::new(String::from(
            "%2\r\n+first\r\n:1\r\n+second\r\n:2\r\n",
        )));
        let value: CustomMap = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(
            value,
            CustomMap {
                first: 1,
                second: 2.0
            }
        );
    }

    #[test]
    fn test_ignore_attribute() {
        // |1<CR><LF>
        //     +key-popularity<CR><LF>
        //     %2<CR><LF>
        //         $1<CR><LF>
        //         a<CR><LF>
        //         ,0.1923<CR><LF>
        //         $1<CR><LF>
        //         b<CR><LF>
        //         ,0.0012<CR><LF>
        //
        let data = "|1\r\n+key-popularity\r\n%2\r\n$1\r\na\r\n,0.1923\r\n$1\r\nb\r\n,0.0012\r\n*2\r\n:2039123\r\n:9543892\r\n";
        let mut d = Deserializer::from_read(Cursor::new(data));
        let value: (u64, u64) = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, (2039123, 9543892));

        let simple = "|1\r\n+hello\r\n+world\r\n#t\r\n";
        let mut d = Deserializer::from_read(Cursor::new(simple));
        let value: bool = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, true);
    }

    #[test]
    fn test_deserialize_attribute() {
        // |1<CR><LF>
        //     +key-popularity<CR><LF>
        //     %2<CR><LF>
        //         $1<CR><LF>
        //         a<CR><LF>
        //         ,0.1923<CR><LF>
        //         $1<CR><LF>
        //         b<CR><LF>
        //         ,0.0012<CR><LF>
        //
        let data = "|1\r\n+key-popularity\r\n%2\r\n$1\r\na\r\n,0.1923\r\n$1\r\nb\r\n,0.0012\r\n*2\r\n:2039123\r\n:9543892\r\n";
        let mut d = Deserializer::from_read(Cursor::new(data));
        #[derive(Deserialize)]
        struct KeyPop {
            a: f64,
            b: f64,
        }
        #[derive(Deserialize)]
        struct Meta {
            #[serde(rename = "key-popularity")]
            key_popularity: KeyPop,
        }
        #[derive(Deserialize, PartialEq, Eq, Debug)]
        struct Pair(u64, u64);

        let with_attr: WithAttribute<Meta, Pair> = Deserialize::deserialize(&mut d).unwrap();
        let (attr, value) = with_attr.into_inner();
        assert_eq!(value, Pair(2039123, 9543892));
        assert_eq!(attr.key_popularity.a, 0.1923);
        assert_eq!(attr.key_popularity.b, 0.0012);
    }

    #[test]
    fn test_nested_deserialize_attribute() {
        //  |1\r\n
        //      +a\r\n
        //      |1\r\n
        //          +b\r\n
        //          +c\r\n
        //      :200\r\n
        //  :300\r\n
        let nested_attr_data = "|1\r\n+a\r\n|1\r\n+b\r\n+c\r\n:200\r\n:300\r\n";
        let mut d = Deserializer::from_read(Cursor::new(nested_attr_data));
        #[derive(Deserialize)]
        struct Test {
            a: usize,
        }
        let with_attr: WithAttribute<Test, usize> = Deserialize::deserialize(&mut d).unwrap();
        let (attr, value) = with_attr.into_inner();
        assert_eq!(attr.a, 200);
        assert_eq!(value, 300);
    }
}
