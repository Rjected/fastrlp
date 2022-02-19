// Copyright 2020 Parity Technologies
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! benchmarking for rlp

use bytes::BytesMut;
use criterion::{criterion_group, criterion_main, Criterion};
use ethnum::U256;
use fastrlp::*;
use hex_literal::hex;

// This wraps the ethnum U256 so we can implement Encodable and Decodable from the rlp crate for
// comparison during benchmarks
struct WrappedU256(U256);

// This From implementation assumes the bytes being passed represent a u256 in big endian.
impl From<&[u8]> for WrappedU256 {
    fn from(bytes: &[u8]) -> Self {
        if bytes.len() > 32 {
            panic!("Can't convert a byte slice greater than 32 bytes to a U256");
        }
        WrappedU256 {
            0: {
                let mut u256_backing = [0u8; 32];
                u256_backing[32 - bytes.len()..].copy_from_slice(bytes);
                U256::from_be_bytes(u256_backing)
            },
        }
    }
}

impl rlp::Encodable for WrappedU256 {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        let buffer = self.0.to_be_bytes();
        s.encoder().encode_value(&buffer);
    }
}

impl rlp::Decodable for WrappedU256 {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        rlp.decoder().decode_value(|bytes| {
            if !bytes.is_empty() && bytes[0] == 0 {
                Err(rlp::DecoderError::RlpInvalidIndirection)
            } else if bytes.len() <= 32 {
                Ok(WrappedU256::from(bytes))
            } else {
                Err(rlp::DecoderError::RlpIsTooBig)
            }
        })
    }
}

fn bench_encode(c: &mut Criterion) {
    c.bench_function("encode_u64", |b| {
        b.iter(|| {
            let mut out = BytesMut::new();
            let _ = 0x1023_4567_89ab_cdefu64.encode(&mut out);
        })
    });
    c.bench_function("encode_u256", |b| {
        b.iter(|| {
            let mut out = BytesMut::new();
            let uint = U256::from_be_bytes(hex!(
                "8090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0"
            ));
            uint.encode(&mut out);
        })
    });
    #[cfg(feature = "ethereum-types")]
    c.bench_function("encode_eth_types_u256", |b| {
        b.iter(|| {
            let mut out = BytesMut::new();
            let uint = ethereum_types::U256::from_big_endian(
                &hex!("8090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0")[..],
            );
            uint.encode(&mut out);
        })
    });
    c.bench_function("encode_1000_u64", |b| {
        b.iter(|| {
            let mut out = BytesMut::new();
            fastrlp::encode_list(
                (0..1000u64).into_iter().collect::<Vec<_>>().as_slice(),
                &mut out,
            );
        })
    });
}

fn bench_old_encode(c: &mut Criterion) {
    c.bench_function("old_encode_u64", |b| {
        b.iter(|| {
            let _out = rlp::Encodable::rlp_bytes(&0x1023_4567_89ab_cdefu64);
        })
    });
    c.bench_function("old_encode_u256", |b| {
        b.iter(|| {
            let uint = WrappedU256(U256::from_be_bytes(hex!(
                "8090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0"
            )));
            let _out = rlp::Encodable::rlp_bytes(&uint);
        })
    });
    #[cfg(feature = "ethereum-types")]
    c.bench_function("encode_old_eth_types_u256", |b| {
        b.iter(|| {
            let uint = ethereum_types::U256::from_big_endian(
                &hex!("8090a0b0c0d0e0f00910203040506077000000000000000100000000000012f0")[..],
            );
            let _out = rlp::Encodable::rlp_bytes(&uint);
        })
    });
}

fn bench_decode(c: &mut Criterion) {
    c.bench_function("decode_u64", |b| {
        b.iter(|| {
            let data = [0x88, 0x10, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
            let _ = u64::decode(&mut &data[..]).unwrap();
        })
    });
    c.bench_function("decode_u256", |b| {
        b.iter(|| {
            let data = vec![
                0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x09, 0x10, 0x20, 0x30, 0x40,
                0x50, 0x60, 0x77, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x12, 0xf0,
            ];
            let _ = U256::decode(&mut &data[..]).unwrap();
        })
    });
    c.bench_function("decode_1000_u64", |b| {
        let input = (0..1000u64).into_iter().collect::<Vec<_>>();
        let mut data = BytesMut::new();
        fastrlp::encode_list(input.as_slice(), &mut data);
        b.iter(|| {
            let _ = Vec::<u64>::decode(&mut &data[..]).unwrap();
        });
    });
}

fn bench_old_decode(c: &mut Criterion) {
    c.bench_function("decode_old_u64_trusted", |b| {
        b.iter(|| {
            let data = [0x88, 0x10, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
            let _: u64 = rlp::decode(&data[..]).unwrap();
        })
    });
    c.bench_function("decode_old_u64_untrusted", |b| {
        b.iter(|| {
            let data = [0x88, 0x10, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
            let data_rlp = rlp::Rlp::new(&data[..]);
            let _: u64 = <u64 as rlp::Decodable>::decode(&data_rlp).unwrap();
        })
    });
    #[cfg(feature = "ethereum-types")]
    c.bench_function("decode_old_eth_u256_trusted", |b| {
        b.iter(|| {
            let data = vec![
                0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x09, 0x10, 0x20, 0x30, 0x40,
                0x50, 0x60, 0x77, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x12, 0xf0,
            ];
            let _: ethereum_types::U256 = rlp::decode(&data[..]).unwrap();
        })
    });
    #[cfg(feature = "ethereum-types")]
    c.bench_function("decode_old_eth_u256_untrusted", |b| {
        b.iter(|| {
            let data = vec![
                0xa0, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0, 0x09, 0x10, 0x20, 0x30, 0x40,
                0x50, 0x60, 0x77, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x12, 0xf0,
            ];
            let uint_rlp = rlp::Rlp::new(&data[..]);
            let _ = <ethereum_types::U256 as rlp::Decodable>::decode(&uint_rlp).unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_old_encode,
    bench_old_decode,
    bench_encode,
    bench_decode
);
criterion_main!(benches);
