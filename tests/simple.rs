#[macro_use]
extern crate serde_derive;

use bincode_embedded::*;

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
fn simple() {
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
    serialize::<_, _, byteorder::NetworkEndian>(&s, &mut writer).unwrap();
    println!("Buffer: {:?}", writer.written_buffer());

    let deserialized: TestStruct =
        deserialize::<_, _, byteorder::NetworkEndian>(&buffer[..]).unwrap();
    assert_eq!(s, deserialized);
}
