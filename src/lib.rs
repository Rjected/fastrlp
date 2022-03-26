#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod decode;
pub mod encode;
mod types;

pub use bytes::BufMut;

pub use decode::{Decodable, DecodeError, Rlp};
pub use encode::{
    const_add, encode_fixed_size, encode_list, length_of_length, list_length, Encodable,
    MaxEncodedLen, MaxEncodedLenAssoc,
};
pub use types::*;

#[cfg(feature = "derive")]
pub use fastrlp_derive::{
    RlpDecodable, RlpDecodableWrapper, RlpEncodable, RlpEncodableWrapper, RlpMaxEncodedLen,
};
