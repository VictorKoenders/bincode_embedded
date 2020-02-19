#![warn(missing_docs)]
#![no_std]

//! Embedded bincode
//!
//! This crate allows [bincode] to be used on embedded devices that run in `#![no_std]`.
//!
//! Currently this is not possible because bincode requires that the given types implement
//! `std::io::Write` or `std::io::Read`, and bincode supports (de)serializing `alloc` types
//! like `Vec` and `String`.
//!
//! This crate is an alternative (but mostly similar) for bincode that works on microcontrollers.
//! It does this by not supporting types like `Vec` and `String`.
//!
//! Types like `&str` and `&[u8]` are supported. This is possible because `CoreRead` has a
//! requirement that the data being read, has to be persisted somewhere. Usually this is done by a
//! fixed-size backing array. The `&str` and `&[u8]` then simply point to a position in that
//! buffer.

mod deserialize;
mod serialize;

pub use deserialize::*;
pub use serialize::*;

/// A target that can be written to. This is similar to `std::io::Write`, but the std trait is not
/// available in `#![no_std]` projects.
///
/// This trait is auto-implemented for [BufferWriter], but can also be implemented to write to an e.g.
/// `embedded_hal::serial::Write`.
pub trait CoreWrite {
    /// The error that this writer can encounter
    type Error: core::fmt::Debug;

    /// Write a single byte to the writer. This is assumed to be blocking, if the underlying writer
    /// is non-blocking, the value should be written to a backing buffer instead.
    fn write(&mut self, val: u8) -> Result<(), Self::Error>;

    /// Flush the writer. This should empty any backing buffer and ensure all data is transferred.
    /// This function should block until all data is flushed.
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Helper function to write multiple bytes to a writer. The default implementation calls
    /// [write] with each byte in the slice.
    fn write_all(&mut self, val: &[u8]) -> Result<(), Self::Error> {
        for byte in val {
            self.write(*byte)?;
        }
        Ok(())
    }
}

/// A target that can be read from. This is similar to `std::io::Read`, but the std trait is not
/// available in `#![no_std]` projects.
///
/// This trait is auto-implemented for `&[u8]`.
///
/// Because the deserialization is done in-place, any object implementing this trait MUST return a
/// persistent reference to the original data. This allows (de)serialization from e.g. `&str` and
/// `&[u8]` without an allocator.
///
/// The easiest way to implement this would be by reading data into a fixed-size array and reading
/// from there.
///
/// This trait does not support async reading yet. Reads are expected to be blocking.
pub trait CoreRead<'a> {
    /// The error that this reader can encounter
    type Error: core::fmt::Debug;

    /// Read a single byte from the current buffer. This is auto-implemented to read a &[u8; 1]
    /// from [read_range] and return the first value.
    ///
    /// This method can be overwritten to allow for more efficient implementations.
    ///
    /// Unlike [read_range], The value returned from this method does not need to be stored in
    /// a persistent buffer. Implementors of this function are free to discard the data that is
    /// returned from this function.
    fn read(&mut self) -> Result<u8, Self::Error> {
        let buff = self.read_range(1)?;
        Ok(buff[0])
    }

    /// Read a byte slice from this reader.
    ///
    /// Because deserialization is done in-place, he value returned MUST be a reference to a
    /// persistent buffer as the returned value can be used for e.g. `&str` and `&[u8]`.
    ///
    /// The returned slice MUST be exactly the size that is requested. The deserializer will
    /// panic when a differently sized slice is returned.
    fn read_range(&mut self, len: usize) -> Result<&'a [u8], Self::Error>;
}

// These are the data types for metadata that is added to serializing and deserializing.
// To change these values, please clone the project and modify them here.
// Make sure to update src/serializer.rs and src/deserializer.rs as well.
// This library shouldn't compile if these data types don't match the size of the data types.
//
// An example error:
//
// Changing
// `pub(crate) type StrLenType = u16;`
// to
// `pub(crate) type StrLenType = u8;`
//
// Raises compiler errors:
// ```
// error[E0308]: mismatched types
//   --> libs/bincode_embedded/src/deserialize.rs:66:27
//    |
// 66 |     let len: StrLenType = B::read_u16(buf);
//    |              ----------   ^^^^^^^^^^^^^^^^ expected `u8`, found `u16`
//    |              |
//    |              expected due to this
//    |
// help: you can convert an `u16` to `u8` and panic if the converted value wouldn't fit
//    |
// 66 |     let len: StrLenType = B::read_u16(buf).try_into().unwrap();
//    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//
// error[E0308]: mismatched types
//   --> libs/bincode_embedded/src/serialize.rs:43:30
//    |
// 43 |     serializer.serialize_u16(str_len as StrLenType)
//    |                              ^^^^^^^^^^^^^^^^^^^^^
//    |                              |
//    |                              expected `u16`, found `u8`
//    |                              help: you can convert an `u8` to `u16`: `(str_len as StrLenType).into()`
// ```
//
// To fix these errors, change:
//   src/deserializer.rs - `read_u16` into `read_u8`
//   src/serializer.rs   - `serialize_u16` into `serializer_u8`

pub(crate) type EnumVariantType = u8;
pub(crate) type UnitVariantType = u8;
pub(crate) type SequenceLengthType = u16;
pub(crate) type StrLenType = u16;
pub(crate) type SliceLenType = u16;
pub(crate) type MapLenType = u8;
pub(crate) type StructVariantType = u8;

/// An implementation of [CoreWrite]. This buffer writer will write data to a backing `&mut [u8]`.
pub struct BufferWriter<'a> {
    buffer: &'a mut [u8],
    index: usize,
}

impl<'a> BufferWriter<'a> {
    /// Create a new writer with a backing buffer.
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, index: 0 }
    }

    /// The bytes count written to the backing buffer.
    pub fn written_len(&self) -> usize {
        self.index
    }

    /// A slice of the buffer that is in this writer. This is equivalent to getting a slice of the
    /// original buffer with the range `..writer.written_len()`.
    /// ```
    /// # let mut buffer: [u8; 0] = [];
    /// # let mut buffer_2: [u8; 0] = [];
    /// # let mut writer = bincode_embedded::BufferWriter::new(&mut buffer_2[..]);
    ///
    /// // These two statements are equivalent
    /// let buffer_slice = &buffer[..writer.written_len()];
    /// let writer_slice = writer.written_buffer();
    ///
    /// assert_eq!(buffer_slice, writer_slice);
    /// ```
    pub fn written_buffer(&self) -> &[u8] {
        &self.buffer[..self.index]
    }
}

/// Errors that can be returned from writing to a [BufferWriter].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BufferWriterError {
    /// The backing buffer of the [BufferWriter] is too small.
    BufferTooSmall,
}

impl CoreWrite for &'_ mut BufferWriter<'_> {
    type Error = BufferWriterError;

    fn write(&mut self, val: u8) -> Result<(), Self::Error> {
        if self.index >= self.buffer.len() {
            return Err(BufferWriterError::BufferTooSmall);
        }
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
