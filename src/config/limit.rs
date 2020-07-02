/// A trait for stopping serialization and deserialization when a certain limit has been reached.
pub trait SizeLimit {
    /// Tells the SizeLimit that a certain number of bytes has been
    /// read or written.  Returns Err if the limit has been exceeded.
    fn add(&mut self, n: u64) -> Result<(), LimitError>;
    /// Returns the hard limit (if one exists)
    fn limit(&self) -> Option<u64>;
}

/// Reached an error regarding the size limit that was passed to the options.
#[non_exhaustive]
pub enum LimitError {
    /// Reached the limit of the given size
    LimitReached,
}

impl core::fmt::Debug for LimitError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            LimitError::LimitReached => write!(fmt, "Limit reached"),
        }
    }
}

/// A SizeLimit that restricts serialized or deserialized messages from
/// exceeding a certain byte length.
#[derive(Copy, Clone)]
pub struct Bounded(pub u64);

/// A SizeLimit without a limit!
/// Use this if you don't care about the size of encoded or decoded messages.
#[derive(Copy, Clone)]
pub struct Infinite;

impl SizeLimit for Bounded {
    #[inline(always)]
    fn add(&mut self, n: u64) -> Result<(), LimitError> {
        if self.0 >= n {
            self.0 -= n;
            Ok(())
        } else {
            Err(LimitError::LimitReached)
        }
    }

    #[inline(always)]
    fn limit(&self) -> Option<u64> {
        Some(self.0)
    }
}

impl SizeLimit for Infinite {
    #[inline(always)]
    fn add(&mut self, _: u64) -> Result<(), LimitError> {
        Ok(())
    }

    #[inline(always)]
    fn limit(&self) -> Option<u64> {
        None
    }
}
