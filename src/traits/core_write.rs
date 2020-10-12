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

#[cfg(feature = "alloc")]
impl<'a> CoreWrite for &'a mut alloc::vec::Vec<u8> {
    type Error = ();
    fn write(&mut self, val: u8) -> Result<(), ()> {
        self.push(val);
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl CoreWrite for alloc::vec::Vec<u8> {
    type Error = ();
    fn write(&mut self, val: u8) -> Result<(), ()> {
        self.push(val);
        Ok(())
    }
}
