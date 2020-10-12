use core::marker::PhantomData;

pub(crate) use self::endian::BincodeByteOrder;
pub(crate) use self::int::IntEncoding;
pub(crate) use self::internal::InternalOptions;
pub(crate) use self::limit::SizeLimit;
pub(crate) use self::trailing::TrailingBytes;

pub use self::endian::{BigEndian, LittleEndian, NativeEndian};
pub use self::int::{FixintEncoding, VarintEncoding};
pub use self::limit::{Bounded, Infinite, LimitError};
pub use self::trailing::{AllowTrailing, RejectTrailing};
pub use crate::traits::{CoreReadBytes, SliceReadError};
use crate::{
    deserialize::DeserializeError,
    serialize::SerializeError,
    traits::{CoreRead, CoreWrite},
};

mod endian;
mod int;
mod internal;
mod limit;
mod trailing;

/// The default options for bincode serialization/deserialization.
///
/// ### Defaults
/// By default bincode will use little-endian encoding for multi-byte integers, and will not
/// limit the number of serialized/deserialized bytes.
#[derive(Copy, Clone)]
pub struct DefaultOptions(Infinite);

impl DefaultOptions {
    /// Get a default configuration object.
    ///
    /// ### Default Configuration:
    ///
    /// | Byte limit | Endianness | Int Encoding | Trailing Behavior |
    /// |------------|------------|--------------|-------------------|
    /// | Unlimited  | Little     | Varint       | Reject            |
    pub fn new() -> DefaultOptions {
        DefaultOptions(Infinite)
    }
}

impl Default for DefaultOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl InternalOptions for DefaultOptions {
    type Limit = Infinite;
    type Endian = LittleEndian;
    type IntEncoding = VarintEncoding;
    type Trailing = RejectTrailing;

    #[inline(always)]
    fn limit(&mut self) -> &mut Infinite {
        &mut self.0
    }
}

/// A configuration builder trait whose options Bincode will use
/// while serializing and deserializing.
///
/// ### Options
/// Endianness: The endianness with which multi-byte integers will be read/written.  *default: little endian*
///
/// Limit: The maximum number of bytes that will be read/written in a bincode serialize/deserialize. *default: unlimited*
///
/// Int Encoding: The encoding used for numbers, enum discriminants, and lengths. *default: varint*
///
/// Trailing Behavior: The behavior when there are trailing bytes left over in a slice after deserialization. *default: reject*
///
/// ### Byte Limit Details
/// The purpose of byte-limiting is to prevent Denial-Of-Service attacks whereby malicious attackers get bincode
/// deserialization to crash your process by allocating too much memory or keeping a connection open for too long.
///
/// When a byte limit is set, bincode will return `Err` on any deserialization that goes over the limit, or any
/// serialization that goes over the limit.
/// Sets the byte limit to be unlimited.
/// This is the default.
pub trait Options: InternalOptions + Sized {
    /// Sets the byte limit to be unlimited.
    /// This is the default.
    fn with_no_limit(self) -> WithOtherLimit<Self, Infinite> {
        WithOtherLimit::new(self, Infinite)
    }

    /// Sets the byte limit to `limit`.
    fn with_limit(self, limit: u64) -> WithOtherLimit<Self, Bounded> {
        WithOtherLimit::new(self, Bounded(limit))
    }

    /// Sets the endianness to little-endian
    /// This is the default.
    fn with_little_endian(self) -> WithOtherEndian<Self, LittleEndian> {
        WithOtherEndian::new(self)
    }

    /// Sets the endianness to big-endian
    fn with_big_endian(self) -> WithOtherEndian<Self, BigEndian> {
        WithOtherEndian::new(self)
    }

    /// Sets the endianness to the the machine-native endianness
    fn with_native_endian(self) -> WithOtherEndian<Self, NativeEndian> {
        WithOtherEndian::new(self)
    }

    /// Sets the length encoding to varint
    fn with_varint_encoding(self) -> WithOtherIntEncoding<Self, VarintEncoding> {
        WithOtherIntEncoding::new(self)
    }

    /// Sets the length encoding to be fixed
    fn with_fixint_encoding(self) -> WithOtherIntEncoding<Self, FixintEncoding> {
        WithOtherIntEncoding::new(self)
    }

    /// Sets the deserializer to reject trailing bytes
    fn reject_trailing_bytes(self) -> WithOtherTrailing<Self, RejectTrailing> {
        WithOtherTrailing::new(self)
    }

    /// Sets the deserializer to allow trailing bytes
    fn allow_trailing_bytes(self) -> WithOtherTrailing<Self, AllowTrailing> {
        WithOtherTrailing::new(self)
    }

    /// Returns the size that an object would be if serialized using Bincode with this configuration
    #[inline(always)]
    fn serialized_size<T: ?Sized + serde::Serialize>(
        self,
        t: &T,
    ) -> Result<u64, SerializeError<()>> {
        crate::serialize::serialize_size(t, self)
    }

    /// Serializes an object directly into a `Writer` using this configuration
    ///
    /// If the serialization would take more bytes than allowed by the size limit, an error
    /// is returned and *no bytes* will be written into the `Writer`
    #[inline(always)]
    fn serialize_into<W: CoreWrite, T: ?Sized + serde::Serialize>(
        self,
        w: W,
        t: &T,
    ) -> Result<(), SerializeError<<W as CoreWrite>::Error>> {
        crate::serialize::serialize(t, w, self)
    }

    /// Deserializes a slice of bytes into an instance of `T` using this configuration
    #[inline(always)]
    fn deserialize_bytes<'a, T: serde::Deserialize<'a>>(
        self,
        bytes: &'a [u8],
    ) -> Result<T, DeserializeError<SliceReadError>> {
        crate::deserialize::deserialize(CoreReadBytes(bytes), self)
    }

    /// TODO: document
    #[doc(hidden)]
    #[inline(always)]
    fn deserialize_in_place<'a, R, T>(
        self,
        reader: R,
        place: &'a mut T,
    ) -> Result<(), DeserializeError<<R as CoreRead>::Error>>
    where
        R: CoreRead<'a> + 'a,
        T: serde::de::Deserialize<'a>,
    {
        *place = crate::deserialize::deserialize(reader, self)?;
        Ok(())
    }

    /// Deserializes an object directly from a `Read`er using this configuration
    ///
    /// If this returns an `Error`, `reader` may be in an invalid state.
    #[inline(always)]
    #[cfg(feature = "alloc")]
    fn deserialize_from<'de, R: CoreRead<'de> + 'de, T: serde::de::DeserializeOwned>(
        self,
        reader: R,
    ) -> Result<T, DeserializeError<<R as CoreRead>::Error>> {
        crate::deserialize::deserialize(reader, self)
    }
}

impl<T: InternalOptions> Options for T {}

/// A configuration struct with a user-specified byte limit
#[derive(Clone, Copy)]
pub struct WithOtherLimit<O: Options, L: SizeLimit> {
    _options: O,
    pub(crate) new_limit: L,
}

/// A configuration struct with a user-specified endian order
#[derive(Clone, Copy)]
pub struct WithOtherEndian<O: Options, E: BincodeByteOrder> {
    options: O,
    _endian: PhantomData<E>,
}

/// A configuration struct with a user-specified length encoding
pub struct WithOtherIntEncoding<O: Options, I: IntEncoding> {
    options: O,
    _length: PhantomData<I>,
}

/// A configuration struct with a user-specified trailing bytes behavior.
pub struct WithOtherTrailing<O: Options, T: TrailingBytes> {
    options: O,
    _trailing: PhantomData<T>,
}

impl<O: Options, L: SizeLimit> WithOtherLimit<O, L> {
    #[inline(always)]
    pub(crate) fn new(options: O, limit: L) -> WithOtherLimit<O, L> {
        WithOtherLimit {
            _options: options,
            new_limit: limit,
        }
    }
}

impl<O: Options, E: BincodeByteOrder> WithOtherEndian<O, E> {
    #[inline(always)]
    pub(crate) fn new(options: O) -> WithOtherEndian<O, E> {
        WithOtherEndian {
            options,
            _endian: PhantomData,
        }
    }
}

impl<O: Options, I: IntEncoding> WithOtherIntEncoding<O, I> {
    #[inline(always)]
    pub(crate) fn new(options: O) -> WithOtherIntEncoding<O, I> {
        WithOtherIntEncoding {
            options,
            _length: PhantomData,
        }
    }
}

impl<O: Options, T: TrailingBytes> WithOtherTrailing<O, T> {
    #[inline(always)]
    pub(crate) fn new(options: O) -> WithOtherTrailing<O, T> {
        WithOtherTrailing {
            options,
            _trailing: PhantomData,
        }
    }
}

impl<O: Options, E: BincodeByteOrder + 'static> InternalOptions for WithOtherEndian<O, E> {
    type Limit = O::Limit;
    type Endian = E;
    type IntEncoding = O::IntEncoding;
    type Trailing = O::Trailing;
    #[inline(always)]
    fn limit(&mut self) -> &mut O::Limit {
        self.options.limit()
    }
}

impl<O: Options, L: SizeLimit + 'static> InternalOptions for WithOtherLimit<O, L> {
    type Limit = L;
    type Endian = O::Endian;
    type IntEncoding = O::IntEncoding;
    type Trailing = O::Trailing;
    fn limit(&mut self) -> &mut L {
        &mut self.new_limit
    }
}

impl<O: Options, I: IntEncoding + 'static> InternalOptions for WithOtherIntEncoding<O, I> {
    type Limit = O::Limit;
    type Endian = O::Endian;
    type IntEncoding = I;
    type Trailing = O::Trailing;

    fn limit(&mut self) -> &mut O::Limit {
        self.options.limit()
    }
}

impl<O: Options, T: TrailingBytes + 'static> InternalOptions for WithOtherTrailing<O, T> {
    type Limit = O::Limit;
    type Endian = O::Endian;
    type IntEncoding = O::IntEncoding;
    type Trailing = T;

    fn limit(&mut self) -> &mut O::Limit {
        self.options.limit()
    }
}
