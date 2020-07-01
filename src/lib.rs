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

pub mod config;
pub mod deserialize;
pub mod serialize;
pub mod traits;

use self::traits::{CoreRead, CoreWrite};

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
