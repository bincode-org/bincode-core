#[macro_use]
extern crate serde_derive;

use bincode_core::BufferWriter;
use bincode_core::{deserialize, serialize, DefaultOptions};
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TestStruct {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: u128,

    opt: Option<u8>,
    buff: [u8; 3],
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum SimpleEnum {
    A,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UnitStruct;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct NewTypeStruct(u8);

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ComplexEnum {
    A(u8),
    B(u8, i8),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TupleStruct(u8, i8);

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct SimpleStruct {
    a: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct ComplexStruct {
    a: SimpleStruct,
    b: SimpleEnum,
    c: ComplexEnum,
}

#[test]
fn simple_struct() {
    let s = TestStruct {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e: 5,
        opt: Some(6),
        buff: [7, 8, 9],
    };

    let mut buffer = [0u8; 100];
    let mut writer = BufferWriter::new(&mut buffer);
    let options = DefaultOptions::new();
    serialize(&s, &mut writer, options).unwrap();
    println!("Buffer: {:?}", writer.written_buffer());

    // type         size
    // u8           1
    // u16          1
    // u32          1
    // u64          1
    // u128         1
    // Option<u8>   1 + 1
    // [u8; 3]      3 (fixed array so no length)
    assert_eq!(1 + 1 + 1 + 1 + 1 + 1 + 1 + 3, writer.written_len());

    let deserialized: TestStruct = deserialize(&buffer[..], options).unwrap();
    assert_eq!(s, deserialized);
}

#[test]
fn simple_tuple() {
    let s = (1u16, 2u32, &b"test"[..], "tesT");

    let mut buffer = [0u8; 100];
    let mut writer = BufferWriter::new(&mut buffer);
    let options = DefaultOptions::new();
    serialize(&s, &mut writer, options).unwrap();
    println!("Buffer: {:?}", writer.written_buffer());

    // type         size
    // u16          1
    // u32          1
    // &[u8]        1 (len) + 4 (byte content)
    // &str         1 (len) + 4 (str content)
    assert_eq!(1 + 1 + 1 + 4 + 1 + 4, writer.written_len());

    let deserialized: (u16, u32, &[u8], &str) = deserialize(&buffer[..], options).unwrap();
    assert_eq!(s, deserialized);
}

macro_rules! simple_test {
    ($name:ident($prim: ty), val: $val: expr, size: $size: expr) => {
        #[test]
        fn $name() {
            let s: $prim = $val;
            let mut buffer = [0u8; 100];
            let mut writer = BufferWriter::new(&mut buffer);
            let options = DefaultOptions::new();
            serialize(&s, &mut writer, options).unwrap();
            println!("Buffer: {:?}", writer.written_buffer());

            assert_eq!($size, writer.written_len());

            let deserialized: $prim = deserialize(&buffer[..], options).unwrap();
            assert_eq!(s, deserialized);
        }
    };
}

simple_test!(test_bool(bool), val: true, size: 1);
simple_test!(test_i8(i8), val: -1, size: 1);
simple_test!(test_i16(i16), val: -2, size: 1);
simple_test!(test_i32(i32), val: -3, size: 1);
simple_test!(test_i64(i64), val: -4, size: 1);
simple_test!(test_i128(i128), val: -5, size: 1);
simple_test!(test_isize(isize), val: -6, size: 1);
simple_test!(test_u8(u8), val: 1, size: 1);
simple_test!(test_u16(u16), val: 2, size: 1);
simple_test!(test_u32(u32), val: 3, size: 1);
simple_test!(test_u64(u64), val: 4, size: 1);
simple_test!(test_u128(u128), val: 5, size: 1);
simple_test!(test_usize(usize), val: 6, size: 1);
simple_test!(test_f32(f32), val: 1.0, size: 4);
simple_test!(test_f64(f64), val: -1.0, size: 8);
simple_test!(test_char(char), val: 'a', size: 1);
// Units should be zero size
simple_test!(test_unit(()), val: (), size: 0);
simple_test!(test_phantom_data(PhantomData<()>), val: PhantomData, size: 0);
simple_test!(test_unit_struct(UnitStruct), val: UnitStruct, size: 0);
// String has length (1 byte) + content (4 bytes)
simple_test!(test_string(&str), val: "Test", size: 5);
// Slice has length (1 byte) + content (1 byte)
simple_test!(test_slice(&[u8]), val: &[1], size: 2);
// Option type (1 byte) + content (1 byte)
simple_test!(test_option_some(Option<u8>), val: Some(1), size: 2);
// Option (None) type (1 byte)
simple_test!(test_option_none(Option<u8>), val: None, size: 1);
// Enum variant (1 bytes)
simple_test!(test_enum_variant(SimpleEnum), val: SimpleEnum::A, size: 1);
// Newtype struct content (1 byte)
simple_test!(test_newtype_struct(NewTypeStruct), val: NewTypeStruct(1), size: 1);
// Newtype enum variant (1 byte) + content (1 byte)
simple_test!(test_newtype_enum_variant(ComplexEnum), val: ComplexEnum::A(1), size: 2);
// Tuple enum variant (1 byte) + content (2 bytes)
simple_test!(test_tuple_enum_variant(ComplexEnum), val: ComplexEnum::B(1, -1), size: 3);
// Tuple content (2 bytes)
simple_test!(test_tuple((u8, i8)), val: (1, -1), size: 2);
// Tuple struct (2 bytes)
simple_test!(test_tuple_struct(TupleStruct), val: TupleStruct(1, -1), size: 2);
// Simple struct (1 bytes)
simple_test!(test_simple_struct(SimpleStruct), val: SimpleStruct{ a: 1 }, size: 1);
// Complex struct - a (1 byte) + b: (1 byte) + c: (1 byte + 1 byte)
simple_test!(test_complex_struct(ComplexStruct), val: ComplexStruct{ a: SimpleStruct { a: 1 }, b: SimpleEnum::A, c: ComplexEnum::A(1) }, size: 4);
