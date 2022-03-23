use std::{
    io::{self, Read},
    iter::Peekable,
    str,
};

use num::{CheckedAdd, CheckedMul};
use serde::de::Unexpected;

use crate::{Error, Result};

pub struct Reader<R: Read> {
    r: Peekable<io::Bytes<R>>,
}

pub struct Deserializer<R: Read> {
    reader: Reader<R>,
    buf: Vec<u8>,
}

impl<R> Reader<R>
where
    R: Read,
{
    pub fn from_reader(r: R) -> Self {
        Reader {
            r: r.bytes().peekable(),
        }
    }

    fn eat_char(&mut self) {
        self.r.next();
    }

    fn next_char(&mut self) -> Result<Option<u8>> {
        self.r.next().transpose().map_err(|e| Error::io(&e))
    }

    fn peek_char(&mut self) -> Result<Option<u8>> {
        match self.r.peek() {
            Some(Ok(ch)) => Ok(Some(*ch)),
            Some(Err(err)) => Err(Error::io(&err)),
            None => Ok(None),
        }
    }

    fn read_length(&mut self) -> Result<usize> {
        self.read_unsigned()
    }

    fn read_unsigned<T>(&mut self) -> Result<T>
    where
        T: CheckedMul + CheckedAdd + From<u8>,
    {
        let next = match self.next_char()? {
            Some(ch) => ch,
            None => {
                return Err(Error::eof());
            }
        };
        match next {
            b'0' => match self.peek_char()? {
                Some(b'0'..=b'9') => Err(Error::invalid_length()),
                _ => Ok(T::from(0)),
            },
            ch @ b'1'..=b'9' => {
                let mut num = T::from(ch - b'0');
                loop {
                    match self.peek_char()? {
                        Some(c @ b'0'..=b'9') => {
                            let mut digit = T::from(c - b'0');
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
                            self.eat_char();
                        }
                        _ => {
                            return Ok(num);
                        }
                    }
                }
            }
            _ => Err(Error::invalid_length()),
        }
    }

    fn read_str_len(&mut self, len: usize, buf: &mut Vec<u8>) -> Result<()> {
        for _count in 0..len {
            match self.next_char()? {
                Some(c) => {
                    buf.push(c);
                }
                None => return Err(Error::eof()),
            }
        }

        Ok(())
    }

    fn read_non_crlf_str(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        loop {
            match self.peek_char()? {
                Some(ch) if ch != b'\r' && ch != b'\n' => {
                    self.eat_char();
                    buf.push(ch);
                }
                None => return Err(Error::eof()),
                _ => break,
            }
        }

        Ok(())
    }

    fn parse_ident(&mut self, ident: &[u8]) -> Result<()> {
        for expected in ident {
            match self.next_char()? {
                None => return Err(Error::eof()),
                Some(ch) => {
                    if ch != *expected {
                        return Err(Error::expect_ident());
                    }
                }
            }
        }

        Ok(())
    }

    fn read_double(&mut self, buf: &mut Vec<u8>) -> Result<f64> {
        let mut negative = false;
        let mut inf = false;
        if let Some(b'-') = self.peek_char()? {
            negative = true;
            self.eat_char();
        }
        if let Some(b'i') = self.peek_char()? {
            self.parse_ident(b"inf")?;
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
            match self.peek_char()? {
                Some(ch) if ch != b'\r' && ch != b'\n' => {
                    self.eat_char();
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
        match self.peek_char()? {
            Some(b't') => {
                self.eat_char();
                self.eat_crlf()?;
                Ok(true)
            }
            Some(b'f') => {
                self.eat_char();
                self.eat_crlf()?;
                Ok(false)
            }
            _ => Err(Error::expect_ident()),
        }
    }

    fn eat_crlf(&mut self) -> Result<()> {
        self.parse_ident(b"\r\n")
    }
}

impl<R: Read> Deserializer<R> {
    pub fn from_reader(r: R) -> Self {
        Deserializer {
            reader: Reader::from_reader(r),
            buf: Vec::new(),
        }
    }

    fn parse_blob_string(&mut self) -> Result<&[u8]> {
        self.buf.clear();

        let len = self.reader.read_length()?;
        self.reader.eat_crlf()?;

        self.reader.read_str_len(len, &mut self.buf)?;
        self.reader.eat_crlf()?;

        Ok(&self.buf[..])
    }

    fn parse_simple_string(&mut self) -> Result<&str> {
        self.buf.clear();
        self.reader.read_non_crlf_str(&mut self.buf)?;
        self.reader.eat_crlf()?;

        let val = str::from_utf8(&self.buf[..]).map_err(|e| Error::utf8(e.valid_up_to()))?;
        Ok(val)
    }

    fn parse_double(&mut self) -> Result<f64> {
        self.buf.clear();
        let val = self.reader.read_double(&mut self.buf)?;
        self.reader.eat_crlf()?;

        Ok(val)
    }
}

impl<'de, 'a, R: Read + 'de> serde::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = super::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.reader.peek_char()?.ok_or_else(|| Error::eof())?;

        match peek {
            // blob string
            b'$' => self.deserialize_bytes(visitor),
            b'+' => self.deserialize_str(visitor),
            _ => Err(Error::expected_value("type header")),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = match self.reader.peek_char()? {
            Some(ch) => ch,
            None => return Err(Error::eof()),
        };

        match peek {
            b'#' => {
                self.reader.eat_char();
                let val = self.reader.read_bool()?;
                visitor.visit_bool(val)
            }
            _ => Err(Error::invalid_type(&"bool")),
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
        let peek = self.reader.peek_char()?.ok_or_else(|| Error::eof())?;

        match peek {
            b':' => {
                self.reader.eat_char();
                match self.reader.peek_char()? {
                    Some(b'-') => {
                        self.reader.eat_char();
                        let num: i64 = self.reader.read_unsigned()?;
                        self.reader.eat_crlf()?;
                        visitor.visit_i64(-num)
                    }
                    Some(b'0'..=b'9') => {
                        let num: i64 = self.reader.read_unsigned()?;
                        self.reader.eat_crlf()?;
                        visitor.visit_i64(num)
                    }
                    _ => Err(Error::expected_number()),
                }
            }
            _ => Err(Error::invalid_type("number")),
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
        let peek = self.reader.peek_char()?.ok_or_else(|| Error::eof())?;

        match peek {
            b':' => {
                self.reader.eat_char();
                match self.reader.peek_char()? {
                    Some(b'-') => Err(Error::invalid_type("signed")),
                    Some(b'0'..=b'9') => {
                        let num: u64 = self.reader.read_unsigned()?;
                        self.reader.eat_crlf()?;
                        visitor.visit_u64(num)
                    }
                    _ => Err(Error::expected_number()),
                }
            }
            ch => {
                println!("peeked {}", ch);
                Err(Error::invalid_type("number"))
            }
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
        let peek = self.reader.peek_char()?.ok_or_else(|| Error::eof())?;

        match peek {
            b':' => {
                self.reader.eat_char();
                match self.reader.peek_char()? {
                    Some(b'-') => {
                        self.reader.eat_char();
                        let num: i64 = self.reader.read_unsigned()?;
                        self.reader.eat_crlf()?;
                        visitor.visit_f64(-num as f64)
                    }
                    Some(b'0'..=b'9') => {
                        let num: i64 = self.reader.read_unsigned()?;
                        self.reader.eat_crlf()?;
                        visitor.visit_f64(num as f64)
                    }
                    _ => Err(Error::expected_number()),
                }
            }
            b',' => {
                self.reader.eat_char();
                let num = self.parse_double()?;
                visitor.visit_f64(num)
            }
            _ => Err(Error::expected_number()),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.reader.peek_char()?.ok_or_else(|| Error::eof())?;

        match peek {
            b'+' => {
                self.reader.eat_char();
                let string = self.parse_simple_string()?;
                visitor.visit_str(string)
            }
            b'-' => {
                self.reader.eat_char();
                let string = self.parse_simple_string()?;
                visitor.visit_str(string)
            }
            b'$' => {
                self.reader.eat_char();
                let bytes = self.parse_blob_string()?;
                let string = str::from_utf8(bytes).map_err(|e| Error::utf8(e.valid_up_to()))?;
                visitor.visit_str(string)
            }
            b'!' => {
                self.reader.eat_char();
                let bytes = self.parse_blob_string()?;
                let string = str::from_utf8(bytes).map_err(|e| Error::utf8(e.valid_up_to()))?;
                visitor.visit_str(string)
            }
            _ => Err(Error::invalid_type("string")),
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
        self.deserialize_any(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.reader.peek_char()?.ok_or_else(|| Error::eof())?;

        match peek {
            b'_' => {
                self.reader.eat_char();
                self.reader.eat_crlf()?;
                visitor.visit_none()
            }
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.reader.peek_char()?.ok_or_else(|| Error::eof())?;

        match peek {
            b'_' => {
                self.reader.eat_char();
                self.reader.eat_crlf()?;
                visitor.visit_unit()
            }
            _ => Err(Error::expected_value("null")),
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
        let peek = self.reader.peek_char()?.ok_or_else(|| Error::eof())?;

        match name {
            crate::custom::SIMPLE_ERROR_TOKEN => {
                if peek == b'-' {
                    self.reader.eat_char();
                    let string = self.parse_simple_string()?;
                    visitor.visit_str(string)
                } else {
                    Err(Error::invalid_type("simple error"))
                }
            }
            crate::custom::BLOB_ERROR_TOKEN => {
                if peek == b'!' {
                    self.reader.eat_char();
                    let bytes = self.parse_blob_string()?;
                    let string = str::from_utf8(bytes).map_err(|e| Error::utf8(e.valid_up_to()))?;
                    visitor.visit_str(string)
                } else {
                    Err(Error::invalid_type("simple error"))
                }
            }
            crate::custom::SIMPLE_STRING_TOKEN => {
                if peek == b'+' {
                    self.reader.eat_char();
                    let string = self.parse_simple_string()?;
                    visitor.visit_str(string)
                } else {
                    Err(Error::invalid_type("simple error"))
                }
            }
            crate::custom::BLOB_STRING_TOKEN => {
                if peek == b'$' {
                    self.reader.eat_char();
                    let bytes = self.parse_blob_string()?;
                    let string = str::from_utf8(bytes).map_err(|e| Error::utf8(e.valid_up_to()))?;
                    visitor.visit_str(string)
                } else {
                    Err(Error::invalid_type("simple error"))
                }
            }
            _ => visitor.visit_newtype_struct(self),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let peek = self.reader.peek_char()?.ok_or_else(|| Error::eof())?;

        match peek {
            b'*' => {
                self.reader.eat_char();
                let len = self.reader.read_length()?;
                self.reader.eat_crlf()?;
                visitor.visit_seq(CountSeqAccess::new(self, len))
            }
            _ => Err(Error::invalid_type("array")),
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
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
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
        todo!()
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        todo!()
    }
}
struct CountSeqAccess<'a, R: Read> {
    de: &'a mut Deserializer<R>,
    len: usize,
}

impl<'a, R: Read> CountSeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: usize) -> Self {
        CountSeqAccess { de, len }
    }
}

impl<'de, 'a, R: Read + 'a + 'de> serde::de::SeqAccess<'de> for CountSeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.len > 0 {
            println!("peek {:?}", self.de.reader.peek_char());
            let result = seed.deserialize(&mut *self.de).map(|r| Some(r));
            self.len -= 1;

            result
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use serde::Deserialize;

    use crate::custom::owned::{BlobError, BlobString, SimpleError, SimpleString};

    use super::*;

    #[test]
    fn test_blob_string() {
        let mut d = Deserializer::from_reader(Cursor::new(String::from("$11\r\nhello world\r\n")));
        let value: String = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, "hello world");

        let mut d = Deserializer::from_reader(Cursor::new(String::from("$11\r\nhello world\r\n")));
        let value: BlobString = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value.0, "hello world");
    }

    #[test]
    fn test_simple_string() {
        let mut d = Deserializer::from_reader(Cursor::new(String::from("+hello world\r\n")));
        let value: String = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, "hello world");

        let mut d = Deserializer::from_reader(Cursor::new(String::from("+hello world\r\n")));
        let value: SimpleString = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value.0, "hello world");
    }

    #[test]
    fn test_blob_error() {
        let mut d =
            Deserializer::from_reader(Cursor::new(String::from("!15\r\nERR hello world\r\n")));
        let value: BlobError = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value.0, "ERR hello world");
    }

    #[test]
    fn test_simple_error() {
        let mut d = Deserializer::from_reader(Cursor::new(String::from("-ERR hello world\r\n")));
        let value: SimpleError = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value.0, "ERR hello world");
    }

    #[test]
    fn test_bool() {
        let mut d = Deserializer::from_reader(Cursor::new(String::from("#t\r\n")));
        let value: bool = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, true);

        let mut d = Deserializer::from_reader(Cursor::new(String::from("#f\r\n")));
        let value: bool = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, false);
    }

    #[test]
    fn test_number() {
        let mut d = Deserializer::from_reader(Cursor::new(String::from(":12345\r\n")));
        let value: i64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, 12345);

        let mut d = Deserializer::from_reader(Cursor::new(String::from(":-12345\r\n")));
        let value: i64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, -12345);
    }

    #[test]
    fn test_double() {
        let mut d = Deserializer::from_reader(Cursor::new(String::from(",1.23\r\n")));
        let value: f64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, 1.23);

        let mut d = Deserializer::from_reader(Cursor::new(String::from(",10\r\n")));
        let value: f64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, 10.0);

        let mut d = Deserializer::from_reader(Cursor::new(String::from(",inf\r\n")));
        let value: f64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, f64::INFINITY);

        let mut d = Deserializer::from_reader(Cursor::new(String::from(",-inf\r\n")));
        let value: f64 = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, f64::NEG_INFINITY);
    }

    #[test]
    fn test_seq() {
        let mut d =
            Deserializer::from_reader(Cursor::new(String::from("*3\r\n:1\r\n:2\r\n:3\r\n")));
        let value: Vec<u64> = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, [1, 2, 3]);

        let mut d = Deserializer::from_reader(Cursor::new(String::from(
            "*2\r\n*3\r\n:1\r\n$5\r\nhello\r\n:2\r\n#f\r\n",
        )));
        let value: ((u64, String, u64), bool) = Deserialize::deserialize(&mut d).unwrap();
        assert_eq!(value, ((1, String::from("hello"), 2), false));
    }
}
