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
