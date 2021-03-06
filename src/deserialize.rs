use super::*;
use core::{marker::PhantomData, str};
use serde::{de::*, serde_if_integer128};

/// Deserialize a given object from the given [CoreRead] object.
///
/// Rust will detect the first two generic arguments automatically. The third generic argument
/// must be a valid `byteorder::ByteOrder` type. Normally this can be implemented like this:
///
/// `let val: Type = deserialize::<_, _, byteorder::NetworkEndian>(&reader)?;`
///
/// or
///
/// `let val = deserialize::<Type, _, byteorder::NetworkEndian>(&reader)?;`
///
/// ```
/// # extern crate serde_derive;
/// # use serde_derive::Deserialize;
/// # use bincode_embedded::deserialize;
///
/// #[derive(Deserialize, PartialEq, Debug)]
/// pub struct SomeStruct {
///     a: u8,
///     b: u8,
/// }
/// let buffer: [u8; 2] = [
///     3, // a
///     6, // b
/// ];
/// let val = deserialize::<SomeStruct, _, byteorder::NetworkEndian>(&buffer[..]).unwrap();
/// assert_eq!(val, SomeStruct { a: 3, b: 6 });
/// ```
pub fn deserialize<
    'a,
    T: Deserialize<'a>,
    R: CoreRead<'a> + 'a,
    B: byteorder::ByteOrder + 'static,
>(
    reader: R,
) -> Result<T, DeserializeError<'a, R>> {
    let mut deserializer = Deserializer::<'a, R, B> {
        reader,
        pd: PhantomData,
    };
    T::deserialize(&mut deserializer)
}

/// Errors that can occur while deserializing
pub enum DeserializeError<'a, R: CoreRead<'a>> {
    /// Failed to read from the provided `CoreRead`. The inner exception is given.
    Read(R::Error),

    /// Invalid bool value. Only `0` and `1` are valid values.
    InvalidBoolValue(u8),

    /// Invalid character encoding while trying to deserialize a `&str`.
    InvalidCharEncoding,

    /// UTF8 error while trying to deserialize a `&str`
    Utf8(str::Utf8Error),

    /// Invalid value for the `Option` part of `Option<T>`. Only `0` and `1` are accepted values.
    InvalidOptionValue(u8),
}

impl<'a, R: CoreRead<'a>> From<str::Utf8Error> for DeserializeError<'a, R> {
    fn from(err: str::Utf8Error) -> Self {
        Self::Utf8(err)
    }
}

impl<'a, R: CoreRead<'a>> core::fmt::Debug for DeserializeError<'a, R> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            DeserializeError::Read(e) => write!(fmt, "{:?}", e),
            DeserializeError::InvalidBoolValue(v) => {
                write!(fmt, "Unknown bool value, got {}, expected 0 or 1", v)
            }
            DeserializeError::InvalidCharEncoding => write!(fmt, "Invalid character encoding"),
            DeserializeError::Utf8(e) => write!(
                fmt,
                "Could not deserialize the value as a value UTF8 string: {:?}",
                e
            ),
            DeserializeError::InvalidOptionValue(e) => {
                write!(fmt, "Invalid Option value, got {}, expected 0 or 1", e)
            }
        }
    }
}

impl<'a, R: CoreRead<'a>> core::fmt::Display for DeserializeError<'a, R> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

impl<'a, R: CoreRead<'a>> Error for DeserializeError<'a, R> {
    fn custom<T: core::fmt::Display>(_cause: T) -> Self {
        panic!("Custom error thrown: {}", _cause);
    }
}

fn get_slice_length<'a, R: CoreRead<'a>, B: byteorder::ByteOrder + 'static>(
    reader: &mut R,
) -> Result<usize, R::Error> {
    let buf = reader.read_range(2)?;
    let len: SliceLenType = B::read_u16(buf);
    Ok(len as usize)
}

fn get_str_length<'a, R: CoreRead<'a>, B: byteorder::ByteOrder + 'static>(
    reader: &mut R,
) -> Result<usize, R::Error> {
    let buf = reader.read_range(2)?;
    let len: StrLenType = B::read_u16(buf);
    Ok(len as usize)
}

fn get_seq_len<'a, R: CoreRead<'a>, B: byteorder::ByteOrder + 'static>(
    reader: &mut R,
) -> Result<usize, R::Error> {
    let buf = reader.read_range(2)?;
    let len: SequenceLengthType = B::read_u16(buf);
    Ok(len as usize)
}

/// A deserializer that can be used to deserialize any `serde::Deserialize` type from a given
/// [CoreRead] reader.
pub struct Deserializer<'a, R: CoreRead<'a> + 'a, B: byteorder::ByteOrder + 'static> {
    reader: R,
    pd: PhantomData<&'a B>,
}

impl<'a, 'b, R: CoreRead<'a> + 'a, B: byteorder::ByteOrder + 'static> serde::Deserializer<'a>
    for &'b mut Deserializer<'a, R, B>
{
    type Error = DeserializeError<'a, R>;

    fn deserialize_any<V: Visitor<'a>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        panic!("Deserialize any not supported")
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

        for byte in buf.iter_mut().take(width).skip(1) {
            *byte = self.reader.read().map_err(DeserializeError::Read)?;
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

        visitor.visit_borrowed_str(res)
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
        visitor.visit_borrowed_bytes(buf)
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
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple<V: Visitor<'a>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        struct Access<'a, 'b, R: CoreRead<'a> + 'a, B: byteorder::ByteOrder + 'static> {
            deserializer: &'b mut Deserializer<'a, R, B>,
            len: usize,
        }

        impl<'a, 'b, R: CoreRead<'a> + 'a, B: byteorder::ByteOrder + 'static>
            serde::de::SeqAccess<'a> for Access<'a, 'b, R, B>
        {
            type Error = DeserializeError<'a, R>;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
            where
                T: serde::de::DeserializeSeed<'a>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    let value =
                        serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(value))
                } else {
                    Ok(None)
                }
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        let access: Access<'a, 'b, R, B> = Access {
            deserializer: self,
            len,
        };

        visitor.visit_seq(access)
    }

    fn deserialize_tuple_struct<V: Visitor<'a>>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        struct Access<'a, 'b, R: CoreRead<'a> + 'a, B: byteorder::ByteOrder + 'static> {
            deserializer: &'b mut Deserializer<'a, R, B>,
            len: usize,
        }

        impl<'a, 'b, R: CoreRead<'a> + 'a, B: byteorder::ByteOrder + 'static>
            serde::de::MapAccess<'a> for Access<'a, 'b, R, B>
        {
            type Error = DeserializeError<'a, R>;

            fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
            where
                K: serde::de::DeserializeSeed<'a>,
            {
                if self.len > 0 {
                    self.len -= 1;
                    let key =
                        serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                    Ok(Some(key))
                } else {
                    Ok(None)
                }
            }

            fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::DeserializeSeed<'a>,
            {
                let value = serde::de::DeserializeSeed::deserialize(seed, &mut *self.deserializer)?;
                Ok(value)
            }

            fn size_hint(&self) -> Option<usize> {
                Some(self.len)
            }
        }

        let len = serde::Deserialize::deserialize(&mut *self)?;

        visitor.visit_map(Access {
            deserializer: self,
            len,
        })
    }

    /// Hint that the `Deserialize` type is expecting a struct with a particular
    /// name and fields.
    fn deserialize_struct<V: Visitor<'a>>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_tuple(fields.len(), visitor)
    }

    /// Hint that the `Deserialize` type is expecting an enum value with a
    /// particular name and possible variants.
    fn deserialize_enum<V: Visitor<'a>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error> {
        unimplemented!()
    }

    /// Hint that the `Deserialize` type is expecting the name of a struct
    /// field or the discriminant of an enum variant.
    fn deserialize_identifier<V: Visitor<'a>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        panic!("Deserialize_identifier not supported")
    }

    /// Hint that the `Deserialize` type needs to deserialize a value whose type
    /// doesn't matter because it is ignored.
    ///
    /// Deserializers for non-self-describing formats may not support this mode.
    fn deserialize_ignored_any<V: Visitor<'a>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        panic!("Deserialize_ignored_any not supported")
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
