use super::*;
use core::marker::PhantomData;
use core::str;
use serde::de::*;
use serde::serde_if_integer128;

pub fn deserialize<'a, T: Deserialize<'a>, R: CoreRead + 'a>(reader: R) -> Result<T, R::Error> {
    unimplemented!();
}

pub enum DeserializeError<R: CoreRead> {
    Read(R::Error),
    InvalidBoolValue(u8),
    InvalidCharEncoding,
    Utf8(str::Utf8Error),
    InvalidOptionValue(u8),
}

impl<R: CoreRead> From<str::Utf8Error> for DeserializeError<R> {
    fn from(err: str::Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl<R: CoreRead> core::fmt::Debug for DeserializeError<R> {
    fn fmt(&self, _fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        unimplemented!();
    }
}

impl<R: CoreRead> core::fmt::Display for DeserializeError<R> {
    fn fmt(&self, _fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        unimplemented!();
    }
}

impl<R: CoreRead> Error for DeserializeError<R> {
    fn custom<T: core::fmt::Display>(_cause: T) -> Self {
        unimplemented!()
    }
}

fn get_slice_length<R: CoreRead, B: byteorder::ByteOrder + 'static>(
    reader: &mut R,
) -> Result<usize, R::Error> {
    let buf = reader.read_range(2)?;
    let len: SliceLenType = B::read_u16(buf);
    Ok(len as usize)
}

fn get_str_length<R: CoreRead, B: byteorder::ByteOrder + 'static>(
    reader: &mut R,
) -> Result<usize, R::Error> {
    let buf = reader.read_range(2)?;
    let len: StrLenType = B::read_u16(buf);
    Ok(len as usize)
}

fn get_seq_len<R: CoreRead, B: byteorder::ByteOrder + 'static>(
    reader: &mut R,
) -> Result<usize, R::Error> {
    let buf = reader.read_range(2)?;
    let len: SequenceLengthType = B::read_u16(buf);
    Ok(len as usize)
}

pub struct Deserializer<R: CoreRead, B: byteorder::ByteOrder + 'static> {
    reader: R,
    pd: PhantomData<B>,
}

impl<'a, R: CoreRead, B: byteorder::ByteOrder + 'static> serde::Deserializer<'a>
    for &'a mut Deserializer<R, B>
{
    type Error = DeserializeError<R>;

    fn deserialize_any<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    fn deserialize_bool<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let value: u8 = serde::Deserialize::deserialize(self)?;
        match value {
            1 => visitor.visit_bool(true),
            0 => visitor.visit_bool(false),
            value => Err(DeserializeError::InvalidBoolValue(value)),
        }
    }

    fn deserialize_i8<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let val = self.reader.read().map_err(DeserializeError::Read)?;
        visitor.visit_i8(val as i8)
    }

    fn deserialize_i16<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let buffer = self.reader.read_range(2).map_err(DeserializeError::Read)?;
        visitor.visit_i16(B::read_i16(&buffer))
    }

    fn deserialize_i32<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let buffer = self.reader.read_range(4).map_err(DeserializeError::Read)?;
        visitor.visit_i32(B::read_i32(&buffer))
    }

    fn deserialize_i64<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let buffer = self.reader.read_range(8).map_err(DeserializeError::Read)?;
        visitor.visit_i64(B::read_i64(&buffer))
    }

    serde_if_integer128! {
        fn deserialize_i128<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            let buffer = self.reader.read_range(16).map_err(DeserializeError::Read)?;
            visitor.visit_i128(B::read_i128(&buffer))
        }
    }

    fn deserialize_u8<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let val = self.reader.read().map_err(DeserializeError::Read)?;
        visitor.visit_u8(val)
    }

    fn deserialize_u16<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let buffer = self.reader.read_range(2).map_err(DeserializeError::Read)?;
        visitor.visit_u16(B::read_u16(&buffer))
    }

    fn deserialize_u32<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let buffer = self.reader.read_range(4).map_err(DeserializeError::Read)?;
        visitor.visit_u32(B::read_u32(&buffer))
    }

    fn deserialize_u64<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let buffer = self.reader.read_range(8).map_err(DeserializeError::Read)?;
        visitor.visit_u64(B::read_u64(&buffer))
    }

    serde_if_integer128! {
        fn deserialize_u128<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
            let buffer = self.reader.read_range(16).map_err(DeserializeError::Read)?;
            visitor.visit_u128(B::read_u128(&buffer))
        }
    }

    fn deserialize_f32<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let buffer = self.reader.read_range(4).map_err(DeserializeError::Read)?;
        visitor.visit_f32(B::read_f32(&buffer))
    }

    fn deserialize_f64<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let buffer = self.reader.read_range(8).map_err(DeserializeError::Read)?;
        visitor.visit_f64(B::read_f64(&buffer))
    }

    fn deserialize_char<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let mut buf = [0u8; 4];

        // Look at the first byte to see how many bytes must be read
        buf[0] = self.reader.read().map_err(DeserializeError::Read)?;
        let width = utf8_char_width(buf[0]);
        if width == 1 {
            return visitor.visit_char(buf[0] as char);
        }
        if width == 0 {
            return Err(DeserializeError::InvalidCharEncoding);
        }

        for i in 1..width {
            buf[1] = self.reader.read().map_err(DeserializeError::Read)?;
        }
        let res = str::from_utf8(&buf[..width])?
            .chars()
            .next()
            .ok_or(DeserializeError::InvalidCharEncoding)?;
        visitor.visit_char(res)
    }

    fn deserialize_str<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let length = get_str_length::<R, B>(&mut self.reader).map_err(DeserializeError::Read)?;
        let buf = self
            .reader
            .read_range(length)
            .map_err(DeserializeError::Read)?;
        let res = str::from_utf8(buf)?;

        visitor.visit_str(res)
    }

    fn deserialize_string<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let length = get_slice_length::<R, B>(&mut self.reader).map_err(DeserializeError::Read)?;
        let buf = self
            .reader
            .read_range(length)
            .map_err(DeserializeError::Read)?;
        visitor.visit_bytes(buf)
    }

    fn deserialize_byte_buf<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let val = self.reader.read().map_err(DeserializeError::Read)?;
        if val == 0 {
            visitor.visit_none()
        } else if val == 1 {
            visitor.visit_some(self)
        } else {
            Err(DeserializeError::InvalidOptionValue(val))
        }
    }

    fn deserialize_unit<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V: Visitor<'a>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_unit()
    }

    fn deserialize_newtype_struct<V: Visitor<'a>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let len = get_seq_len::<R, B>(&mut self.reader).map_err(DeserializeError::Read)?;
        unimplemented!()
    }

    fn deserialize_tuple<V: Visitor<'a>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    fn deserialize_tuple_struct<V: Visitor<'a>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    fn deserialize_map<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    /// Hint that the `Deserialize` type is expecting a struct with a particular
    /// name and fields.
    fn deserialize_struct<V: Visitor<'a>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    /// Hint that the `Deserialize` type is expecting an enum value with a
    /// particular name and possible variants.
    fn deserialize_enum<V: Visitor<'a>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    /// Hint that the `Deserialize` type is expecting the name of a struct
    /// field or the discriminant of an enum variant.
    fn deserialize_identifier<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    /// Hint that the `Deserialize` type needs to deserialize a value whose type
    /// doesn't matter because it is ignored.
    ///
    /// Deserializers for non-self-describing formats may not support this mode.
    fn deserialize_ignored_any<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

const UTF8_CHAR_WIDTH: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0xBF
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, // 0xDF
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
];

// This function is a copy of experimental function core::str::utf8_char_width
const fn utf8_char_width(b: u8) -> usize {
    UTF8_CHAR_WIDTH[b as usize] as usize
}

/*
// This is the same function as above, but without a lookup table
// In godbolt this resulted in a lot more runtime code, but it's a valid alternative
// https://godbolt.org/z/3DePUa

pub fn utf8_char_width(b: u8) -> usize {
    if b <= 0x7F { 1 }
    else if b <= 0xC1 { 0 }
    else if b <= 0xDF { 2 }
    else if b <= 0xEF { 3 }
    else if b <= 0xF4 { 4 }
    else { 0 }
}

*/
