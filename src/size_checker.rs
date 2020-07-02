use crate::config::IntEncoding;
use crate::{config::Options, serialize::SerializeError, traits::CoreWrite};
use core::mem::size_of;
use serde::serde_if_integer128;

pub(crate) struct SizeChecker<O: Options> {
    pub options: O,
    pub total: u64,
}

impl<O: Options> CoreWrite for SizeChecker<O> {
    type Error = ();
    fn write(&mut self, _val: u8) -> Result<(), ()> {
        self.total += 1;
        Ok(())
    }
}

impl<O: Options> SizeChecker<O> {
    fn add_raw(&mut self, len: u64) -> Result<(), SerializeError<()>> {
        self.total += len;
        Ok(())
    }

    fn add_discriminant(&mut self, idx: u32) -> Result<(), SerializeError<()>> {
        let bytes = O::IntEncoding::u32_size(idx);
        self.add_raw(bytes)
    }

    fn add_len(&mut self, len: usize) -> Result<(), SerializeError<()>> {
        let bytes = O::IntEncoding::len_size(len);
        self.add_raw(bytes)
    }
}

macro_rules! impl_size_int {
    ($ser_method:ident($ty:ty) = $size_method:ident()) => {
        fn $ser_method(self, v: $ty) -> Result<(), SerializeError<()>> {
            self.add_raw(O::IntEncoding::$size_method(v))
        }
    };
}

impl<'a, O: Options> serde::Serializer for &'a mut SizeChecker<O> {
    type Ok = ();
    type Error = SerializeError<()>;
    type SerializeSeq = Compound<'a, O>;
    type SerializeTuple = Compound<'a, O>;
    type SerializeTupleStruct = Compound<'a, O>;
    type SerializeTupleVariant = Compound<'a, O>;
    type SerializeMap = Compound<'a, O>;
    type SerializeStruct = Compound<'a, O>;
    type SerializeStructVariant = Compound<'a, O>;

    fn serialize_unit(self) -> Result<(), SerializeError<()>> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<(), SerializeError<()>> {
        Ok(())
    }

    fn serialize_bool(self, _: bool) -> Result<(), SerializeError<()>> {
        self.add_raw(1)
    }

    fn serialize_u8(self, _: u8) -> Result<(), SerializeError<()>> {
        self.add_raw(1)
    }
    fn serialize_i8(self, _: i8) -> Result<(), SerializeError<()>> {
        self.add_raw(1)
    }

    impl_size_int! {serialize_u16(u16) = u16_size()}
    impl_size_int! {serialize_u32(u32) = u32_size()}
    impl_size_int! {serialize_u64(u64) = u64_size()}
    impl_size_int! {serialize_i16(i16) = i16_size()}
    impl_size_int! {serialize_i32(i32) = i32_size()}
    impl_size_int! {serialize_i64(i64) = i64_size()}

    serde_if_integer128! {
        impl_size_int!{serialize_u128(u128) = u128_size()}
        impl_size_int!{serialize_i128(i128) = i128_size()}
    }

    fn serialize_f32(self, _: f32) -> Result<(), SerializeError<()>> {
        self.add_raw(size_of::<f32>() as u64)
    }

    fn serialize_f64(self, _: f64) -> Result<(), SerializeError<()>> {
        self.add_raw(size_of::<f64>() as u64)
    }

    fn serialize_str(self, v: &str) -> Result<(), SerializeError<()>> {
        self.add_len(v.len())?;
        self.add_raw(v.len() as u64)
    }

    fn serialize_char(self, c: char) -> Result<(), SerializeError<()>> {
        self.add_raw(encode_utf8(c).as_slice().len() as u64)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<(), SerializeError<()>> {
        self.add_len(v.len())?;
        self.add_raw(v.len() as u64)
    }

    fn serialize_none(self) -> Result<(), SerializeError<()>> {
        self.add_raw(1)
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<(), SerializeError<()>>
    where
        T: serde::Serialize,
    {
        self.add_raw(1)?;
        v.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, SerializeError<()>> {
        let len = len.ok_or(SerializeError::SequenceMustHaveLength)?;

        self.add_len(len)?;
        Ok(Compound { ser: self })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, SerializeError<()>> {
        Ok(Compound { ser: self })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, SerializeError<()>> {
        Ok(Compound { ser: self })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, SerializeError<()>> {
        self.add_raw(O::IntEncoding::u32_size(variant_index))?;
        Ok(Compound { ser: self })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, SerializeError<()>> {
        let len = len.ok_or(SerializeError::SequenceMustHaveLength)?;

        self.add_len(len)?;
        Ok(Compound { ser: self })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, SerializeError<()>> {
        Ok(Compound { ser: self })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, SerializeError<()>> {
        self.add_discriminant(variant_index)?;
        Ok(Compound { ser: self })
    }

    fn serialize_newtype_struct<V: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        v: &V,
    ) -> Result<(), SerializeError<()>> {
        v.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<(), SerializeError<()>> {
        self.add_discriminant(variant_index)
    }

    fn serialize_newtype_variant<V: serde::Serialize + ?Sized>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &V,
    ) -> Result<(), SerializeError<()>> {
        self.add_discriminant(variant_index)?;
        value.serialize(self)
    }

    fn is_human_readable(&self) -> bool {
        false
    }

    fn collect_str<T: ?Sized>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: core::fmt::Display,
    {
        todo!()
    }
}

pub(crate) struct Compound<'a, S: Options + 'a> {
    ser: &'a mut SizeChecker<S>,
}

impl<'a, O: Options> serde::ser::SerializeSeq for Compound<'a, O> {
    type Ok = ();
    type Error = SerializeError<()>;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), SerializeError<()>>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), SerializeError<()>> {
        Ok(())
    }
}

impl<'a, O: Options> serde::ser::SerializeTuple for Compound<'a, O> {
    type Ok = ();
    type Error = SerializeError<()>;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), SerializeError<()>>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), SerializeError<()>> {
        Ok(())
    }
}

impl<'a, O: Options> serde::ser::SerializeTupleStruct for Compound<'a, O> {
    type Ok = ();
    type Error = SerializeError<()>;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), SerializeError<()>>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), SerializeError<()>> {
        Ok(())
    }
}

impl<'a, O: Options> serde::ser::SerializeTupleVariant for Compound<'a, O> {
    type Ok = ();
    type Error = SerializeError<()>;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), SerializeError<()>>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), SerializeError<()>> {
        Ok(())
    }
}

impl<'a, O: Options + 'a> serde::ser::SerializeMap for Compound<'a, O> {
    type Ok = ();
    type Error = SerializeError<()>;

    #[inline]
    fn serialize_key<K: ?Sized>(&mut self, value: &K) -> Result<(), SerializeError<()>>
    where
        K: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<V: ?Sized>(&mut self, value: &V) -> Result<(), SerializeError<()>>
    where
        V: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), SerializeError<()>> {
        Ok(())
    }
}

impl<'a, O: Options> serde::ser::SerializeStruct for Compound<'a, O> {
    type Ok = ();
    type Error = SerializeError<()>;

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), SerializeError<()>>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), SerializeError<()>> {
        Ok(())
    }
}

impl<'a, O: Options> serde::ser::SerializeStructVariant for Compound<'a, O> {
    type Ok = ();
    type Error = SerializeError<()>;

    #[inline]
    fn serialize_field<T: ?Sized>(
        &mut self,
        _key: &'static str,
        value: &T,
    ) -> Result<(), SerializeError<()>>
    where
        T: serde::ser::Serialize,
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<(), SerializeError<()>> {
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
