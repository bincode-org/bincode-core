use core::str;

#[cfg(feature = "std")]
use std::error::Error as StdError;

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

    /// Fills the given buffer from the reader.
    /// The input buffer MUST be completely filled. If the reader reaches end-of-file before filling the
    /// buffer an error MUST be returned.
    fn fill(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error>;

    /// Forward a string slice from the reader on to the given visitor.
    ///
    /// If allocations are not available on the system, the bytes forwarded MUST be a reference to a
    /// persistent buffer and forwarded on to `visitor.visit_borrowed_str`.
    ///
    /// The forwarded slice MUST be exactly the size that is requested.
    fn forward_str<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'a>;

    /// Forward a byte slice from the reader on to the given visitor.
    ///
    /// If allocations are not available on the system, the bytes forwarded MUST be a reference to a
    /// persistent buffer and forwarded on to `visitor.visit_borrowed_bytes`.
    ///
    /// The forwarded slice MUST be exactly the size that is requested.
    fn forward_bytes<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'a>;
}

impl<'a> CoreRead<'a> for &'a [u8] {
    type Error = SliceReadError;

    fn fill(&mut self, buffer: &mut [u8]) -> Result<(), Self::Error> {
        if buffer.len() > self.len() {
            return Err(SliceReadError::EndOfSlice);
        }
        buffer.copy_from_slice(&self[..buffer.len()]);
        *self = &self[buffer.len()..];
        Ok(())
    }

    fn forward_bytes<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'a>,
    {
        if len > self.len() {
            return Err(SliceReadError::EndOfSlice);
        }
        let result = &self[..len];
        *self = &self[len..];

        visitor.visit_borrowed_bytes(result)
    }

    fn forward_str<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'a>,
    {
        if len > self.len() {
            return Err(SliceReadError::EndOfSlice);
        }
        let result = &self[..len];
        *self = &self[len..];

        let string = match str::from_utf8(result) {
            Ok(s) => s,
            Err(_) => return Err(SliceReadError::InvalidUtf8),
        };

        visitor.visit_borrowed_str(string)
    }
}

/// An error that is thrown when reading from a slice.
#[derive(Debug)]
pub enum SliceReadError {
    /// Tried reading more bytes than the slice contains.
    EndOfSlice,
    InvalidUtf8,
}

impl serde::de::Error for SliceReadError {
    fn custom<T: core::fmt::Display>(_cause: T) -> Self {
        panic!("Custom error thrown: {}", _cause);
    }
}

impl core::fmt::Display for SliceReadError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "{:?}", self)
    }
}

#[cfg(feature = "std")]
impl StdError for SliceReadError {}
