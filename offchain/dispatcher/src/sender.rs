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

use anyhow::Result;
use async_trait::async_trait;
use contracts::{authority::Authority, history::Claim};
use eth_tx_manager::{
    database::FileSystemDatabase as Database,
    gas_oracle::DefaultGasOracle as GasOracle,
    manager::Configuration,
    time::DefaultTime as Time,
    transaction::{Priority, Transaction, Value},
    Chain, Error as TransactionManagerError, TransactionManager,
};
use rollups_events::RollupsClaim;
use state_fold_types::{
    ethabi::Token,
    ethers::{
        self,
        abi::AbiEncode,
        middleware::SignerMiddleware,
        providers::{
            Http, HttpRateLimitRetryPolicy, MockProvider, Provider, RetryClient,
        },
        signers::Signer,
        types::{Address, Bytes, NameOrAddress},
    },
};
use std::sync::Arc;
use tracing::{instrument, trace, warn};
use url::Url;

use crate::{config::DispatcherConfig, signer::ConditionalSigner};

// We added this trait for dependency injection and ease of testing.
#[async_trait]
pub trait Sender: std::fmt::Debug + Sized {
    async fn submit_claim(
        self,
        dapp_address: Address,
        rollups_claim: RollupsClaim,
    ) -> Result<Self>;
}

struct SubmittableClaim(Address, RollupsClaim);

impl From<SubmittableClaim> for Bytes {
    fn from(submittable_claim: SubmittableClaim) -> Self {
        let SubmittableClaim(dapp_address, claim) = submittable_claim;
        let claim = Claim {
            epoch_hash: claim.epoch_hash.into_inner(),
            first_index: claim.first_index,
            last_index: claim.last_index,
        };
        ethers::abi::encode(&[
            Token::Address(dapp_address),
            Token::FixedBytes(claim.encode()),
        ])
        .into()
    }
}

const MAX_RETRIES: u32 = 10;
const INITIAL_BACKOFF: u64 = 1000;

/// Instantiates the tx-manager calling `new` or `force_new`.
macro_rules! tx_manager {
    ($new: ident, $middleware: expr, $database_path: expr, $chain: expr) => {
        TransactionManager::$new(
            $middleware.clone(),
            GasOracle::new(),
            Database::new($database_path.clone()),
            $chain,
            Configuration::default(),
        )
        .await
    };
}

type Middleware =
    Arc<SignerMiddleware<Provider<RetryClient<Http>>, ConditionalSigner>>;

type TxManager = TransactionManager<Middleware, GasOracle, Database, Time>;

#[derive(Debug)]
pub struct ClaimSender {
    tx_manager: TxManager,
    confirmations: usize,
    priority: Priority,
    from: Address,
    authority: Authority<Provider<MockProvider>>,
}

// Private.
impl ClaimSender {
    /// Creates the (layered) middleware instance to be sent to the tx-manager.
    fn middleware(
        conditional_signer: ConditionalSigner,
        provider_url: String,
    ) -> Result<Middleware> {
        let base_layer = Http::new(Url::parse(&provider_url)?);
        let retry_layer = Provider::new(RetryClient::new(
            base_layer,
            Box::new(HttpRateLimitRetryPolicy),
            MAX_RETRIES,
            INITIAL_BACKOFF,
        ));
        let signer_layer =
            SignerMiddleware::new(retry_layer, conditional_signer);
        Ok(Arc::new(signer_layer))
    }

    /// Creates the tx-manager instance.
    /// NOTE: does not try to reinstantiate the tx-manager more than once.
    async fn tx_manager(
        conditional_signer: &ConditionalSigner,
        provider_url: String,
        database_path: String,
        chain: Chain,
    ) -> Result<TxManager> {
        let middleware =
            Self::middleware(conditional_signer.clone(), provider_url)?;
        let result = tx_manager!(new, middleware, database_path, chain);
        let tx_manager =
            if let Err(TransactionManagerError::NonceTooLow { .. }) = result {
                warn!("Nonce too low! Clearing the tx-manager database.");
                tx_manager!(force_new, middleware, database_path, chain)?
            } else {
                let (tx_manager, receipt) = result?;
                trace!("Database claim transaction confirmed: `{:?}`", receipt);
                tx_manager
            };
        Ok(tx_manager)
    }
}

// Public.
impl ClaimSender {
    pub async fn new(config: &DispatcherConfig) -> Result<Self> {
        let chain: Chain = (&config.tx_config).into();

        let conditional_signer =
            ConditionalSigner::new(chain.id, &config.auth_config)
                .await
                .expect("Failed to initialize the transaction signer");

        let tx_manager = Self::tx_manager(
            &conditional_signer,
            config.tx_config.provider_http_endpoint.clone(),
            config.tx_config.database_path.clone(),
            chain,
        )
        .await?;

        let authority = {
            let (provider, _mock) = Provider::mocked();
            let provider = Arc::new(provider);
            Authority::new(
                config.rollups_deployment.authority_address,
                provider,
            )
        };

        Ok(Self {
            tx_manager,
            confirmations: config.tx_config.default_confirmations,
            priority: config.priority,
            from: conditional_signer.address(),
            authority,
        })
    }
}

#[async_trait]
impl Sender for ClaimSender {
    #[instrument(level = "trace")]
    async fn submit_claim(
        self,
        dapp_address: Address,
        rollups_claim: RollupsClaim,
    ) -> Result<Self> {
        let transaction = {
            let submittable_claim =
                SubmittableClaim(dapp_address, rollups_claim);
            let call = self
                .authority
                .submit_claim(submittable_claim.into())
                .from(self.from);
            let to = match call.tx.to().expect("tx `to` should not be null") {
                NameOrAddress::Address(a) => *a,
                _ => panic!("expected address, found ENS name"),
            };
            Transaction {
                from: self.from,
                to,
                value: Value::Nothing,
                call_data: call.tx.data().cloned(),
            }
        };

        trace!("Built claim transaction: `{:?}`", transaction);

        let (tx_manager, receipt) = self
            .tx_manager
            .send_transaction(transaction, self.confirmations, self.priority)
            .await?;
        trace!("Claim transaction confirmed: `{:?}`", receipt);

        Ok(Self { tx_manager, ..self })
    }
}
