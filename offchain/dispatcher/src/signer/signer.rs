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
use snafu::{ResultExt, Snafu};
use state_fold_types::ethers::{
    signers::{
        coins_bip39::English, LocalWallet, MnemonicBuilder, Signer, WalletError,
    },
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address, Signature,
    },
};

use crate::{
    auth::AuthConfig,
    signer::aws_signer::{AwsSigner, AwsSignerError},
};

/// The `ConditionalSigner` is implementing dynamic dispatch by hand
/// for objects that implement the `Sender` trait.
///
/// We had to do this because we cannot create a `Box<dyn Signer>` and
/// using parametrics types would move this complexity to the `main_loop`
/// function, which is undesirable.
#[derive(Debug, Clone)]
pub enum ConditionalSigner {
    LocalWallet(LocalWallet),
    Aws(AwsSigner),
}

#[derive(Debug, Snafu)]
pub enum ConditionalSignerError {
    #[snafu(display("Local Wallet error: {source}"))]
    LocalWallet { source: WalletError },

    #[snafu(display("{source}"))]
    Aws { source: AwsSignerError },
}

impl ConditionalSigner {
    pub async fn new(
        chain_id: u64,
        auth_config: &AuthConfig,
    ) -> Result<Self, ConditionalSignerError> {
        match auth_config.clone() {
            AuthConfig::Mnemonic(mnemonic, account_index) => {
                const DEFAULT_ACCOUNT_INDEX: u32 = 0;
                let index = account_index.unwrap_or(DEFAULT_ACCOUNT_INDEX);
                let wallet = MnemonicBuilder::<English>::default()
                    .phrase(mnemonic.clone().as_str())
                    .index(index)
                    .context(LocalWalletSnafu)?
                    .build()
                    .context(LocalWalletSnafu)?
                    .with_chain_id(chain_id);
                Ok(ConditionalSigner::LocalWallet(wallet))
            }
            AuthConfig::AWS(key_id, region) => {
                AwsSigner::new(key_id, chain_id, region)
                    .await
                    .map(ConditionalSigner::Aws)
                    .context(AwsSnafu)
            }
        }
    }
}

#[async_trait]
impl Signer for ConditionalSigner {
    type Error = ConditionalSignerError;

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
        &self,
        message: S,
    ) -> Result<Signature, Self::Error> {
        match &self {
            Self::LocalWallet(local_wallet) => local_wallet
                .sign_message(message)
                .await
                .context(LocalWalletSnafu),
            Self::Aws(aws_signer) => {
                aws_signer.sign_message(message).await.context(AwsSnafu)
            }
        }
    }

    async fn sign_transaction(
        &self,
        message: &TypedTransaction,
    ) -> Result<Signature, Self::Error> {
        match &self {
            Self::LocalWallet(local_wallet) => local_wallet
                .sign_transaction(message)
                .await
                .context(LocalWalletSnafu),
            Self::Aws(aws_signer) => {
                aws_signer.sign_transaction(message).await.context(AwsSnafu)
            }
        }
    }

    async fn sign_typed_data<T: Eip712 + Send + Sync>(
        &self,
        payload: &T,
    ) -> Result<Signature, Self::Error> {
        match &self {
            Self::LocalWallet(local_wallet) => local_wallet
                .sign_typed_data(payload)
                .await
                .context(LocalWalletSnafu),
            Self::Aws(aws_signer) => {
                aws_signer.sign_typed_data(payload).await.context(AwsSnafu)
            }
        }
    }

    fn address(&self) -> Address {
        match &self {
            Self::LocalWallet(local_wallet) => local_wallet.address(),
            Self::Aws(aws_signer) => aws_signer.address(),
        }
    }

    fn chain_id(&self) -> u64 {
        match &self {
            Self::LocalWallet(local_wallet) => local_wallet.chain_id(),
            Self::Aws(aws_signer) => aws_signer.chain_id(),
        }
    }

    fn with_chain_id<T: Into<u64>>(self, chain_id: T) -> Self {
        match &self {
            Self::LocalWallet(local_wallet) => {
                Self::LocalWallet(local_wallet.clone().with_chain_id(chain_id))
            }
            Self::Aws(aws_signer) => {
                Self::Aws(aws_signer.clone().with_chain_id(chain_id))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use state_fold_types::ethers::{
        signers::Signer,
        types::{
            transaction::{eip2718::TypedTransaction, eip2930::AccessList},
            Address, Eip1559TransactionRequest,
        },
    };

    use crate::auth::AuthConfig;

    use super::ConditionalSigner;

    const CHAIN_ID: u64 = 1;
    const MNEMONIC: &str =
        "indoor dish desk flag debris potato excuse depart ticket judge file exit";
    const KEY_ID: &str = "3a5dd2e1-5d01-43a1-89a9-deb5d6db190b";
    const REGION: &str = "us-east-1";

    #[tokio::test]
    async fn new_local_wallet_from_auth_config() {
        let conditional_signer = new_local_wallet_conditional_signer().await;
        if let ConditionalSigner::Aws { .. } = conditional_signer {
            panic!("expected LocalWallet conditional signer")
        }
    }

    #[tokio::test]
    #[ignore]
    async fn new_aws_signer_from_auth_config() {
        let conditional_signer = new_aws_conditional_signer().await;
        if let ConditionalSigner::LocalWallet(_) = conditional_signer {
            panic!("expected Aws conditional signer")
        }
    }

    #[tokio::test]
    async fn local_wallet_sign_transaction() {
        let conditional_signer = new_local_wallet_conditional_signer().await;
        let message = TypedTransaction::Eip1559(eip1559_request());
        let result = conditional_signer.sign_transaction(&message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn aws_signer_sign_transaction() {
        let conditional_signer = new_aws_conditional_signer().await;
        let message = TypedTransaction::Eip1559(eip1559_request());
        let result = conditional_signer.sign_transaction(&message).await;
        assert!(result.is_ok());
    }

    // --------------------------------------------------------------------------------------------
    // Auxiliary
    // --------------------------------------------------------------------------------------------

    fn eip1559_request() -> Eip1559TransactionRequest {
        Eip1559TransactionRequest::new()
            .from(Address::default())
            .to(Address::default())
            .gas(555)
            .value(1337)
            .data(vec![1, 2, 3])
            .nonce(1)
            .access_list(AccessList::default())
            .max_priority_fee_per_gas(10)
            .max_fee_per_gas(20)
            .chain_id(CHAIN_ID)
    }

    async fn new_conditional_signer(
        auth_config: AuthConfig,
    ) -> ConditionalSigner {
        ConditionalSigner::new(CHAIN_ID, &auth_config)
            .await
            .expect("could not instantiate the signer")
    }

    async fn new_local_wallet_conditional_signer() -> ConditionalSigner {
        new_conditional_signer(AuthConfig::Mnemonic(
            MNEMONIC.to_string(),
            Some(1),
        ))
        .await
    }

    async fn new_aws_conditional_signer() -> ConditionalSigner {
        new_conditional_signer(AuthConfig::AWS(
            KEY_ID.to_string(),
            REGION.to_string(),
        ))
        .await
    }
}
