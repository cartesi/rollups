// Copyright Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use hex::FromHexError;
use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum DecodeError {
    #[snafu(display(
        "Failed to decode ethereum binary string {} (expected 0x prefix)",
        s
    ))]
    InvalidPrefix { s: String },
    #[snafu(display("Failed to decode ethereum binary string {} ({})", s, e))]
    FromHex { s: String, e: FromHexError },
}

/// Convert binary array to Ethereum binary format
pub fn encode_ethereum_binary(bytes: &[u8]) -> String {
    String::from("0x") + &hex::encode(bytes)
}

/// Convert string in Ethereum binary format to binary array
pub fn decode_ethereum_binary(s: &str) -> Result<Vec<u8>, DecodeError> {
    snafu::ensure!(s.starts_with("0x"), InvalidPrefixSnafu { s });
    hex::decode(&s[2..]).map_err(|e| DecodeError::FromHex {
        s: s.to_string(),
        e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        assert_eq!(
            encode_ethereum_binary(&[0x01, 0x20, 0xFF, 0x00]).as_str(),
            "0x0120ff00"
        );
    }

    #[test]
    fn test_encode_with_empty() {
        assert_eq!(encode_ethereum_binary(&[]).as_str(), "0x");
    }

    #[test]
    fn test_decode_with_uppercase() {
        assert_eq!(
            decode_ethereum_binary("0x0120FF00").unwrap(),
            vec![0x01, 0x20, 0xFF, 0x00]
        );
    }

    #[test]
    fn test_decode_with_lowercase() {
        assert_eq!(decode_ethereum_binary("0xff").unwrap(), vec![0xFF]);
    }

    #[test]
    fn test_decode_with_invalid_prefix() {
        let err = decode_ethereum_binary("0X0120FF00").unwrap_err();
        assert_eq!(
            err.to_string().as_str(),
            "Failed to decode ethereum binary string 0X0120FF00 (expected 0x prefix)",
        );
    }

    #[test]
    fn test_decode_with_invalid_number() {
        let err = decode_ethereum_binary("0xZZ").unwrap_err();
        assert_eq!(
            err.to_string().as_str(),
            "Failed to decode ethereum binary string 0xZZ (Invalid character 'Z' at position 0)"
        );
    }

    #[test]
    fn test_decode_with_odd_number_of_chars() {
        let err = decode_ethereum_binary("0xA").unwrap_err();
        assert_eq!(
            err.to_string().as_str(),
            "Failed to decode ethereum binary string 0xA (Odd number of digits)"
        );
    }
}
