use bytes::{Bytes, BytesMut};
use ethnum::U256;
use fastrlp::*;
use hex_literal::hex;

#[derive(Debug, PartialEq, RlpEncodable, RlpDecodable)]
struct Item {
    a: Bytes,
}

#[derive(Debug, PartialEq, RlpEncodable, RlpDecodable, RlpMaxEncodedLen)]
struct Test4Numbers {
    a: u8,
    b: u64,
    c: U256,
    d: U256,
}

#[derive(Debug, PartialEq, RlpEncodableWrapper, RlpDecodableWrapper)]
pub struct W(Test4Numbers);

fn encoded<T: Encodable>(t: &T) -> BytesMut {
    let mut out = BytesMut::new();
    t.encode(&mut out);
    out
}

#[test]
fn test_encode_item() {
    let item = Item {
        a: b"dog".to_vec().into(),
    };

    let expected = vec![0xc4, 0x83, b'd', b'o', b'g'];
    let out = encoded(&item);
    assert_eq!(&*out, expected);

    let decoded = Decodable::decode(&mut &*expected).expect("decode failure");
    assert_eq!(item, decoded);

    let item = Test4Numbers {
        a: 0x05,
        b: 0xdeadbeefbaadcafe,
        c: U256::from_be_bytes(hex!(
            "56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"
        )),
        d: U256::from_be_bytes(hex!(
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        )),
    };

    let expected = hex!("f84c0588deadbeefbaadcafea056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470").to_vec();
    let out = encoded(&item);
    assert_eq!(&*out, expected);

    let out = fastrlp::encode_fixed_size(&item);
    assert_eq!(&*out, expected);

    let decoded = Decodable::decode(&mut &*expected).unwrap();
    assert_eq!(item, decoded);

    assert_eq!(encoded(&W(item)), expected);
    assert_eq!(W::decode(&mut &*expected).unwrap().0, decoded);
}
