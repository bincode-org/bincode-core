#[macro_use]
extern crate serde_derive;

use bincode_core::BufferWriter;
use bincode_core::{deserialize, serialize, DefaultOptions};

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
