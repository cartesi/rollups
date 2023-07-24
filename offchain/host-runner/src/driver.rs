// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use byteorder::{BigEndian, WriteBytesExt};
use std::mem::size_of;

use crate::hash::{Digest, Hash, Hasher, HASH_SIZE};

pub fn compute_voucher_hash(destination: &[u8], payload: &[u8]) -> Hash {
    let mut hasher = Hasher::new();
    write_data(&mut hasher, destination);
    write_u64(&mut hasher, 0x40);
    write_payload(&mut hasher, payload);
    hasher.finalize().into()
}

pub fn compute_notice_hash(payload: &[u8]) -> Hash {
    let mut hasher = Hasher::new();
    write_u64(&mut hasher, 0x20);
    write_payload(&mut hasher, payload);
    hasher.finalize().into()
}

fn write_padding(hasher: &mut Hasher, n: usize) {
    let alignment = n % HASH_SIZE;
    if alignment != 0 {
        for _ in alignment..HASH_SIZE {
            hasher.write_u8(0).expect("cannot fail");
        }
    }
}

fn write_u64(hasher: &mut Hasher, value: u64) {
    write_padding(hasher, size_of::<u64>());
    hasher.write_u64::<BigEndian>(value).expect("cannot fail");
}

fn write_data(hasher: &mut Hasher, data: &[u8]) {
    write_padding(hasher, data.len());
    hasher.update(data);
}

fn write_payload(hasher: &mut Hasher, payload: &[u8]) {
    write_u64(hasher, payload.len() as u64);
    hasher.update(payload);
    write_padding(hasher, payload.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_voucher_hash() {
        let destination =
            hex::decode("5555555555555555555555555555555555555555").unwrap();
        let payload: Vec<u8> = "hello world".as_bytes().into();
        let hash = compute_voucher_hash(&destination, &payload);
        let expected_hash = Hash::decode(
            "61a61380d2a3b5e2b09a5ff259a2e1048da1989bdd6d6ecc69594cfbedc01278",
        );
        assert_eq!(&hash, &expected_hash);
    }

    #[test]
    fn test_update_notice_hash() {
        let payload: Vec<u8> = "hello world".as_bytes().into();
        let hash = compute_notice_hash(&payload);
        let expected_hash = Hash::decode(
            "d9f29a4e347ad89dc70490124ee6975fbc0693c7e72d6bc383673bfd0e8841f2",
        );
        assert_eq!(&hash, &expected_hash);
    }
}
