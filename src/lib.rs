#![no_std]

mod deserialize;
mod serialize;

pub use deserialize::*;
pub use serialize::*;

pub trait CoreWrite {
    type Error: core::fmt::Debug;
    fn write(&mut self, val: u8) -> Result<(), Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write_all(&mut self, val: &[u8]) -> Result<(), Self::Error> {
        for byte in val {
            self.write(*byte)?;
        }
        Ok(())
    }
}

pub trait CoreRead<'a> {
    type Error;
    fn read(&mut self) -> Result<u8, Self::Error> {
        let buff = self.read_range(1)?;
        Ok(buff[0])
    }

    fn read_range(&mut self, len: usize) -> Result<&'a [u8], Self::Error>;
}

pub(crate) type EnumVariantType = u8;
pub(crate) type UnitVariantType = u8;
pub(crate) type SequenceLengthType = u16;
pub(crate) type StrLenType = u16;
pub(crate) type SliceLenType = u16;
pub(crate) type MapLenType = u8;
pub(crate) type StructVariantType = u8;

pub struct BufferWriter<'a> {
    buffer: &'a mut [u8],
    index: usize,
}

impl<'a> BufferWriter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, index: 0 }
    }

    pub fn written_buffer(&self) -> &[u8] {
        &self.buffer[..self.index]
    }
}

impl CoreWrite for &'_ mut BufferWriter<'_> {
    type Error = ();
    fn write(&mut self, val: u8) -> Result<(), Self::Error> {
        self.buffer[self.index] = val;
        self.index += 1;
        Ok(())
    }
}

impl CoreWrite for BufferWriter<'_> {
    type Error = ();
    fn write(&mut self, val: u8) -> Result<(), Self::Error> {
        self.buffer[self.index] = val;
        self.index += 1;
        Ok(())
    }
}

impl<'a> CoreRead<'a> for &'a [u8] {
    type Error = ();

    fn read_range(&mut self, len: usize) -> Result<&'a [u8], Self::Error> {
        let result = &self[..len];
        *self = &self[len..];
        Ok(result)
    }
}
