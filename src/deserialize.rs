use super::*;
use config::{BincodeByteOrder, IntEncoding, LimitError, Options, SizeLimit};
use core::str::Utf8Error;
use core::{marker::PhantomData, str};
use serde::{de::*, serde_if_integer128};

// #[cfg(feature = "alloc")]
// use alloc::{string::String, vec::Vec};

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
/// # use bincode_core::{deserialize, DefaultOptions};
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
/// let options = DefaultOptions::new();
/// let val: SomeStruct = deserialize(&buffer[..], options).unwrap();
/// assert_eq!(val, SomeStruct { a: 3, b: 6 });
/// ```
pub fn deserialize<'a, T: Deserialize<'a>, R: CoreRead<'a> + 'a, O: Options>(
    reader: R,
    options: O,
) -> Result<T, DeserializeError<'a, R>> {
    let mut deserializer = Deserializer {
        reader,
        options,
        _lifetime: PhantomData,
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

    /// Limit error reached. See the inner exception for more info.
    LimitError(LimitError),

    /// Could not cast from type `from_type` to type `to_type`. Usually this means that the data is encoded with a different version or protocol.
    InvalidCast {
        /// The base type that was being casted from
        from_type: &'static str,

        /// The target type that was being casted to
        to_type: &'static str,
    },

    /// Invalid UTF8 encoding while trying to parse a `&str` or `String`
    InvalidUtf8Encoding(Utf8Error),

    /// Invalid value (u128 range): you may have a version or configuration disagreement?
    InvalidValueRange,

    /// Byte 255 is treated as an extension point; it should not be encoding anything. Do you have a mismatched bincode version or configuration?
    ExtensionPoint,
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
            DeserializeError::LimitError(e) => write!(fmt, "Limit error {:?}", e),
            DeserializeError::InvalidCast { from_type, to_type } => {
                write!(fmt, "Could not cast from {:?} to {:?}", from_type, to_type)
            }
            DeserializeError::InvalidUtf8Encoding(error) => write!(
                fmt,
                "Invalid UTF8 encoding: {:?}", error
            ),
            DeserializeError::InvalidValueRange => write!(
                fmt,
                "Invalid value (u128 range): you may have a version or configuration disagreement?"
            ),
            DeserializeError::ExtensionPoint => write!(
                fmt,
                "Byte 255 is treated as an extension point; it should not be encoding anything. Do you have a mismatched bincode version or configuration?"
            ),
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

/// A deserializer that can be used to deserialize any `serde::Deserialize` type from a given
/// [CoreRead] reader.
pub struct Deserializer<'a, R: CoreRead<'a> + 'a, O: Options> {
    reader: R,
    options: O,
    _lifetime: PhantomData<&'a ()>,
}

macro_rules! impl_deserialize_literal {
    ($name:ident : $ty:ty = $read:ident()) => {
        #[inline]
        pub(crate) fn $name(&mut self) -> Result<$ty, DeserializeError<'a, R>> {
            self.read_literal_type::<$ty>()?;
            let buffer = self
                .reader
                .read_range(core::mem::size_of::<$ty>())
                .map_err(DeserializeError::Read)?;
            Ok(<<O::Endian as BincodeByteOrder>::Endian as byteorder::ByteOrder>::$read(&buffer))
        }
    };
}

impl<'a, R: CoreRead<'a> + 'a, O: Options> Deserializer<'a, R, O> {
    pub(crate) fn deserialize_byte(&mut self) -> Result<u8, DeserializeError<'a, R>> {
        self.read_literal_type::<u8>()?;
        self.reader.read().map_err(DeserializeError::Read)
    }

    impl_deserialize_literal! { deserialize_literal_u16 : u16 = read_u16() }
    impl_deserialize_literal! { deserialize_literal_u32 : u32 = read_u32() }
    impl_deserialize_literal! { deserialize_literal_u64 : u64 = read_u64() }

    serde_if_integer128! {
        impl_deserialize_literal! { deserialize_literal_u128 : u128 = read_u128() }
    }

    fn read_bytes(&mut self, count: u64) -> Result<(), DeserializeError<'a, R>> {
        self.options
            .limit()
            .add(count)
            .map_err(DeserializeError::LimitError)
    }

    fn read_literal_type<T>(&mut self) -> Result<(), DeserializeError<'a, R>> {
        self.read_bytes(core::mem::size_of::<T>() as u64)
    }

    /*
    #[cfg(feature = "alloc")]
    fn read_vec(&mut self) -> Result<Vec<u8>, DeserializeError<'a, R>> {
        let len = O::IntEncoding::deserialize_len(self)?;
        self.read_bytes(len as u64)?;
        self.reader.read_vec(len).map_err(DeserializeError::Read)
    }

    #[cfg(feature = "alloc")]
    fn read_string(&mut self) -> Result<String, DeserializeError<'a, R>> {
        let vec = self.read_vec()?;
        String::from_utf8(vec)
            .map_err(|e| DeserializeError::InvalidUtf8Encoding(e.utf8_error()).into())
    }
    */
}

macro_rules! impl_deserialize_int {
    ($name:ident = $visitor_method:ident ($dser_method:ident)) => {
        #[inline]
        fn $name<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'a>,
        {
            visitor.$visitor_method(O::IntEncoding::$dser_method(self)?)
        }
    };
}

impl<'a, 'b, R: CoreRead<'a> + 'a, O: Options> serde::Deserializer<'a>
    for &'b mut Deserializer<'a, R, O>
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
        visitor.visit_i8(self.deserialize_byte()? as i8)
    }

    fn deserialize_u8<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_u8(self.deserialize_byte()? as u8)
    }

    impl_deserialize_int!(deserialize_u16 = visit_u16(deserialize_u16));
    impl_deserialize_int!(deserialize_u32 = visit_u32(deserialize_u32));
    impl_deserialize_int!(deserialize_u64 = visit_u64(deserialize_u64));
    impl_deserialize_int!(deserialize_i16 = visit_i16(deserialize_i16));
    impl_deserialize_int!(deserialize_i32 = visit_i32(deserialize_i32));
    impl_deserialize_int!(deserialize_i64 = visit_i64(deserialize_i64));

    serde_if_integer128! {
        impl_deserialize_int!(deserialize_u128 = visit_u128(deserialize_u128));
        impl_deserialize_int!(deserialize_i128 = visit_i128(deserialize_i128));
    }

    fn deserialize_f32<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let buffer = self.reader.read_range(4).map_err(DeserializeError::Read)?;
        visitor.visit_f32(
            <<O::Endian as BincodeByteOrder>::Endian as byteorder::ByteOrder>::read_f32(&buffer),
        )
    }

    fn deserialize_f64<V: Visitor<'a>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        let buffer = self.reader.read_range(8).map_err(DeserializeError::Read)?;
        visitor.visit_f64(
            <<O::Endian as BincodeByteOrder>::Endian as byteorder::ByteOrder>::read_f64(&buffer),
        )
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

    fn deserialize_str<V: Visitor<'a>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        let length = O::IntEncoding::deserialize_len(&mut self)?; // .map_err(DeserializeError::Read)?;
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

    fn deserialize_bytes<V: Visitor<'a>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        let length = O::IntEncoding::deserialize_len(&mut self)?; // .map_err(DeserializeError::Read)?;
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

    fn deserialize_seq<V: Visitor<'a>>(mut self, visitor: V) -> Result<V::Value, Self::Error> {
        let len = O::IntEncoding::deserialize_len(&mut self)?; // .map_err(DeserializeError::Read)?;
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple<V: Visitor<'a>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        struct Access<'a, 'b, R: CoreRead<'a> + 'a, O: Options> {
            deserializer: &'b mut Deserializer<'a, R, O>,
            len: usize,
        }

        impl<'a, 'b, R: CoreRead<'a> + 'a, O: Options> serde::de::SeqAccess<'a> for Access<'a, 'b, R, O> {
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

        let access: Access<'a, 'b, R, O> = Access {
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
        struct Access<'a, 'b, R: CoreRead<'a> + 'a, O: Options> {
            deserializer: &'b mut Deserializer<'a, R, O>,
            len: usize,
        }

        impl<'a, 'b, R: CoreRead<'a> + 'a, O: Options> serde::de::MapAccess<'a> for Access<'a, 'b, R, O> {
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
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        impl<'de, 'a, R: 'a, O> serde::de::EnumAccess<'de> for &'a mut Deserializer<'de, R, O>
        where
            R: CoreRead<'de>,
            O: Options,
        {
            type Error = DeserializeError<'de, R>;
            type Variant = Self;

            fn variant_seed<V>(
                self,
                seed: V,
            ) -> Result<(V::Value, Self::Variant), DeserializeError<'de, R>>
            where
                V: serde::de::DeserializeSeed<'de>,
            {
                let idx: u32 = O::IntEncoding::deserialize_u32(self)?;
                let val: Result<_, DeserializeError<'de, R>> =
                    seed.deserialize(idx.into_deserializer());
                Ok((val?, self))
            }
        }

        visitor.visit_enum(self)
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

impl<'de, 'a, R, O> serde::de::VariantAccess<'de> for &'a mut Deserializer<'de, R, O>
where
    R: CoreRead<'de>,
    O: Options,
{
    type Error = DeserializeError<'de, R>;

    fn unit_variant(self) -> Result<(), DeserializeError<'de, R>> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, DeserializeError<'de, R>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        serde::de::DeserializeSeed::deserialize(seed, self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, DeserializeError<'de, R>>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DeserializeError<'de, R>>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
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
