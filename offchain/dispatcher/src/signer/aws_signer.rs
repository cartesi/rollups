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

use async_trait::async_trait;
use rusoto_core::{
    credential::ChainProvider, region::ParseRegionError, HttpClient, Region,
};
use rusoto_kms::KmsClient;
use snafu::{ResultExt, Snafu};
use state_fold_types::ethers::{
    signers::{
        AwsSigner as InnerAwsSigner, AwsSignerError as InnerAwsSignerError,
        Signer,
    },
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address, Signature,
    },
};
use std::str::FromStr;

/// Creates a `KmsClient` instance.
macro_rules! kms {
    ($region: expr) => {
        KmsClient::new_with(
            HttpClient::new().expect("http client TLS error"),
            ChainProvider::new(),
            Region::from_str(&$region).context(InvalidRegionSnafu)?,
        )
    };
}

/// Creates an `InnerAwsSigner` instance.
macro_rules! inner_aws_signer {
    ($kms: expr, $key_id: expr, $chain_id: expr) => {
        InnerAwsSigner::new(&$kms, $key_id.clone(), $chain_id)
            .await
            .context(InnerSnafu)?
    };
}

/// We don't store an `InnerAwsSigner` here because of lifetime and
/// borrow restrictions imposed by the underlying libraries.
#[derive(Debug, Clone)]
pub struct AwsSigner {
    region: String,
    key_id: String,
    chain_id: u64,
    address: Address,
}

#[derive(Debug, Snafu)]
pub enum AwsSignerError {
    #[snafu(display("AWS KMS error: {source}"))]
    Inner { source: InnerAwsSignerError },

    #[snafu(display("Invalid AWS region: {source}"))]
    InvalidRegion { source: ParseRegionError },
}

impl AwsSigner {
    pub async fn new(
        key_id: String,
        chain_id: u64,
        region: String,
    ) -> Result<Self, AwsSignerError> {
        let kms = kms!(&region);
        let aws_signer = inner_aws_signer!(&kms, key_id, chain_id);
        Ok(Self {
            region,
            key_id,
            chain_id,
            address: aws_signer.address(),
        })
    }
}

/// Calls the async `$method` from an `InnerAwsSigner` instance.
/// Reinstantiates the `InnerAwsSigner`.
macro_rules! inner_aws_signer_call {
    ($aws_signer: expr,
     $method: ident,
     $argument: expr) => {
        inner_aws_signer!(
            &kms!($aws_signer.region),
            $aws_signer.key_id,
            $aws_signer.chain_id
        )
        .$method($argument)
        .await
        .context(InnerSnafu)
    };
}

#[async_trait]
impl Signer for AwsSigner {
    type Error = AwsSignerError;

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        inner_aws_signer_call!(self, sign_message, message)
    }

    async fn sign_transaction(
        &self,
        message: &TypedTransaction,
    ) -> Result<Signature, Self::Error> {
        inner_aws_signer_call!(&self, sign_transaction, message)
    }

    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        inner_aws_signer_call!(&self, sign_typed_data, payload)
    }

    fn address(&self) -> Address {
        self.address.clone()
    }

    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        Self {
            key_id: self.key_id.clone(),
            chain_id: chain_id.into(),
            region: self.region.clone(),
            address: self.address.clone(),
        }
    }
}
