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

/// The `AwsSigner` (re)implements the `Signer` trait for the `InnerAwsSigner`.
///
/// We do not use an `InnerAwsSigner` directly because of lifetime and
/// borrow restrictions imposed by the underlying libraries.
///
/// Instead, we instantiate a new `InnerAwsSigner` every time we call
/// a function from `Signer`.
#[derive(Debug, Clone)]
pub struct AwsSigner {
    region: String,
    key_id: String,
    chain_id: u64,
    address: Address,
}

#[derive(Debug, Snafu)]
pub enum AwsSignerError {
    #[snafu(display("AWS KMS error"))]
    Inner { source: InnerAwsSignerError },

    #[snafu(display("Invalid AWS region"))]
    InvalidRegion { source: ParseRegionError },
}

/// Creates a `KmsClient` instance.
fn create_kms(region: &String) -> Result<KmsClient, AwsSignerError> {
    Ok(KmsClient::new_with(
        HttpClient::new().expect("http client TLS error"),
        ChainProvider::new(),
        Region::from_str(region).context(InvalidRegionSnafu)?,
    ))
}

async fn create_inner_aws_signer<'a>(
    kms: &'a KmsClient,
    key_id: &String,
    chain_id: u64,
) -> Result<InnerAwsSigner<'a>, AwsSignerError> {
    InnerAwsSigner::new(&kms, key_id.clone(), chain_id)
        .await
        .context(InnerSnafu)
}

impl AwsSigner {
    pub async fn new(
        key_id: String,
        chain_id: u64,
        region: String,
    ) -> Result<Self, AwsSignerError> {
        let kms = create_kms(&region)?;
        let aws_signer =
            create_inner_aws_signer(&kms, &key_id, chain_id).await?;
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
        create_inner_aws_signer(
            &create_kms(&$aws_signer.region)?,
            &$aws_signer.key_id,
            $aws_signer.chain_id,
        )
        .await?
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
