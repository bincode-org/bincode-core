use crate::traits::CoreWrite;

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
    /// # let mut writer = bincode_core::BufferWriter::new(&mut buffer_2[..]);
    ///
    /// // These two statements are equivalent
    /// let buffer_slice = &buffer[..writer.written_len() as usize];
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
    type Error = BufferWriterError;
    fn write(&mut self, val: u8) -> Result<(), Self::Error> {
        if self.buffer.is_empty() {
            return Err(BufferWriterError::BufferTooSmall);
        }
        self.buffer[self.index] = val;
        self.index += 1;
        Ok(())
    }
}
