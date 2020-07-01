use super::*;
use config::InternalOptions;
use serde::{ser::*, serde_if_integer128};

/// Serialize a given `T` type into a given `CoreWrite` writer with the given `B` byte order.
///
/// `T` can be any value that derives `serde::Serialize`.
///
/// `W` can be any value that implements [CoreWrite]. This can e.g. be a fixed-size array, or a
/// serial writer.
///
/// `B` can be any type that implements [byteorder::ByteOrder]. This includes:
/// - BigEndian
/// - LittleEndian
/// - NetworkEndian.
pub fn serialize<T: serde::Serialize, W: CoreWrite, O: InternalOptions>(
    value: &T,
    writer: W,
    options: O,
) -> Result<(), SerializeError<W>> {
    let mut serializer = Serializer::<W, O> { writer, options };
    value.serialize(&mut serializer)
}

/// Return the size that serializing a given `T` type would need to be stored. This is an optimized version of:
/// ```ignore
/// let mut writer = ...;
/// serialize(value, &mut writer, options);
/// let len = writer.bytes_written();
/// ```
/// But without actually writing to memory
pub fn serialize_size<T: serde::Serialize, W: CoreWrite, O: InternalOptions>(
    value: &T,
    options: O,
) -> Result<usize, SerializeError<W>> {
    unimplemented!()
}

/// Any error that can be thrown while serializing a type
pub enum SerializeError<W: CoreWrite> {
    /// Generic write error. See the inner `CoreWrite::Error` for more info
    Write(W::Error),

    /// A sequence (e.g. `&str` or `&[u8]`) was requested to serialize, but it has no length.
    SequenceMustHaveLength,
}

impl<W: CoreWrite> core::fmt::Debug for SerializeError<W> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            SerializeError::Write(w) => write!(fmt, "Write error {:?}", w),
            SerializeError::SequenceMustHaveLength => write!(fmt, "Sequence does not have length"),
        }
    }
}

impl<W: CoreWrite> core::fmt::Display for SerializeError<W> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, fmt)
    }
}

impl<W: CoreWrite> serde::ser::Error for SerializeError<W> {
    fn custom<T: core::fmt::Display>(_cause: T) -> Self {
        // Custom errors not supported
        panic!("Custom error: {}", _cause);
    }
}

/// A serializer that can serialize any value that implements `serde::Serialize` into a given
/// [CoreWrite] writer.
pub struct Serializer<W: CoreWrite, O: InternalOptions> {
    writer: W,
    options: O,
}

impl<'a, W: CoreWrite, O: InternalOptions> serde::Serializer for &'a mut Serializer<W, O> {
    type Ok = ();
    type Error = SerializeError<W>;
    type SerializeSeq = Compound<'a, W, O>;
    type SerializeTuple = Compound<'a, W, O>;
    type SerializeTupleStruct = Compound<'a, W, O>;
    type SerializeTupleVariant = Compound<'a, W, O>;
    type SerializeMap = Compound<'a, W, O>;
    type SerializeStruct = Compound<'a, W, O>;
    type SerializeStructVariant = Compound<'a, W, O>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.writer.write(v as u8).map_err(SerializeError::Write)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.writer.write(v as u8).map_err(SerializeError::Write)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 2];
        O::Endian::write_i16(&mut buf, v);
        self.writer.write_all(&buf).map_err(SerializeError::Write)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 4];
        O::Endian::write_i32(&mut buf, v);
        self.writer.write_all(&buf).map_err(SerializeError::Write)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 8];
        O::Endian::write_i64(&mut buf, v);
        self.writer.write_all(&buf).map_err(SerializeError::Write)
    }

    serde_if_integer128! {
        fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
            let mut buf = [0u8; 16];
            O::Endian::write_i128(&mut buf, v);
            self.writer.write_all(&buf).map_err(SerializeError::Write)
        }
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.writer.write(v).map_err(SerializeError::Write)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 2];
        O::Endian::write_u16(&mut buf, v);
        self.writer.write_all(&buf).map_err(SerializeError::Write)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 4];
        O::Endian::write_u32(&mut buf, v);
        self.writer.write_all(&buf).map_err(SerializeError::Write)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 8];
        O::Endian::write_u64(&mut buf, v);
        self.writer.write_all(&buf).map_err(SerializeError::Write)
    }

    serde_if_integer128! {
        fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
            let mut buf = [0u8; 16];
            O::Endian::write_u128(&mut buf, v);
            self.writer.write_all(&buf).map_err(SerializeError::Write)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 4];
        O::Endian::write_f32(&mut buf, v);
        self.writer.write_all(&buf).map_err(SerializeError::Write)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let mut buf = [0u8; 8];
        O::Endian::write_f64(&mut buf, v);
        self.writer.write_all(&buf).map_err(SerializeError::Write)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(encode_utf8(v).as_slice())
            .map_err(SerializeError::Write)
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        O::serialize_len(&mut self, v.len())?;
        self.writer
            .write_all(v.as_bytes())
            .map_err(SerializeError::Write)
    }

    fn serialize_bytes(mut self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        O::serialize_len(&mut self, v.len())?;
        self.writer.write_all(v).map_err(SerializeError::Write)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write(0).map_err(SerializeError::Write)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        self.writer.write(1).map_err(SerializeError::Write)?;
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        mut self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        O::serialize_u32(&mut self, variant_index)?;
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        mut self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        O::serialize_u32(&mut self, variant_index)?;
        value.serialize(self)
    }

    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        O::serialize_len(&mut self, len)?;
        Ok(Compound(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(Compound(self))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Compound(self))
    }

    fn serialize_tuple_variant(
        mut self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        O::serialize_u32(variant_index)?;
        Ok(Compound(self))
    }

    fn serialize_map(mut self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        O::serialize_usize(len)?;
        Ok(Compound(self))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(Compound(self))
    }

    fn serialize_struct_variant(
        mut self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        O::serialize_u32(variant_index)?;
        Ok(Compound(self))
    }

    fn collect_str<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: core::fmt::Display,
    {
        panic!("Unimplemented: Serialize::collect_str")
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

/// Internal struct needed for serialization.
pub struct Compound<'a, W: CoreWrite, O: InternalOptions>(&'a mut Serializer<W, O>);

impl<'a, W: CoreWrite, O: InternalOptions> SerializeSeq for Compound<'a, W, O> {
    type Ok = ();
    type Error = SerializeError<W>;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W: CoreWrite, O: InternalOptions> SerializeTuple for Compound<'a, W, O> {
    type Ok = ();
    type Error = SerializeError<W>;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W: CoreWrite, B: byteorder::ByteOrder + 'static> SerializeTupleStruct
    for Compound<'a, W, B>
{
    type Ok = ();
    type Error = SerializeError<W>;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W: CoreWrite, B: byteorder::ByteOrder + 'static> SerializeTupleVariant
    for Compound<'a, W, B>
{
    type Ok = ();
    type Error = SerializeError<W>;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W: CoreWrite, B: byteorder::ByteOrder + 'static> SerializeMap for Compound<'a, W, B> {
    type Ok = ();
    type Error = SerializeError<W>;

    #[inline]
    fn serialize_key<K: ?Sized>(&mut self, value: &K) -> Result<(), Self::Error>
    where
        K: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<V: ?Sized>(&mut self, value: &V) -> Result<(), Self::Error>
    where
        V: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W: CoreWrite, B: byteorder::ByteOrder + 'static> SerializeStruct for Compound<'a, W, B> {
    type Ok = ();
    type Error = SerializeError<W>;

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, W: CoreWrite, B: byteorder::ByteOrder + 'static> SerializeStructVariant
    for Compound<'a, W, B>
{
    type Ok = ();
    type Error = SerializeError<W>;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

const TAG_CONT: u8 = 0b1000_0000;
const TAG_TWO_B: u8 = 0b1100_0000;
const TAG_THREE_B: u8 = 0b1110_0000;
const TAG_FOUR_B: u8 = 0b1111_0000;
const MAX_ONE_B: u32 = 0x80;
const MAX_TWO_B: u32 = 0x800;
const MAX_THREE_B: u32 = 0x10000;

fn encode_utf8(c: char) -> EncodeUtf8 {
    let code = c as u32;
    let mut buf = [0; 4];
    let pos = if code < MAX_ONE_B {
        buf[3] = code as u8;
        3
    } else if code < MAX_TWO_B {
        buf[2] = (code >> 6 & 0x1F) as u8 | TAG_TWO_B;
        buf[3] = (code & 0x3F) as u8 | TAG_CONT;
        2
    } else if code < MAX_THREE_B {
        buf[1] = (code >> 12 & 0x0F) as u8 | TAG_THREE_B;
        buf[2] = (code >> 6 & 0x3F) as u8 | TAG_CONT;
        buf[3] = (code & 0x3F) as u8 | TAG_CONT;
        1
    } else {
        buf[0] = (code >> 18 & 0x07) as u8 | TAG_FOUR_B;
        buf[1] = (code >> 12 & 0x3F) as u8 | TAG_CONT;
        buf[2] = (code >> 6 & 0x3F) as u8 | TAG_CONT;
        buf[3] = (code & 0x3F) as u8 | TAG_CONT;
        0
    };
    EncodeUtf8 { buf, pos }
}

struct EncodeUtf8 {
    buf: [u8; 4],
    pos: usize,
}

impl EncodeUtf8 {
    fn as_slice(&self) -> &[u8] {
        &self.buf[self.pos..]
    }
}
