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

use crate::{
    config::DispatcherConfig,
    drivers::Context,
    machine::BrokerStatus,
    tx_sender::{BulletproofTxSender, TxSender},
};

use eth_tx_manager::{
    config::TxManagerConfig, database::FileSystemDatabase,
    gas_oracle::DefaultGasOracle,
    manager::Configuration as ManagerConfiguration, Priority,
    TransactionManager,
};
use state_client_lib::{
    config::SCConfig, error::StateServerError, BlockServer,
    GrpcStateFoldClient, StateServer,
};
use state_fold_types::{
    ethers::{
        middleware::SignerMiddleware,
        providers::{Http, HttpRateLimitRetryPolicy, Provider, RetryClient},
        signers::Signer,
        types::Address,
    },
    BlockStreamItem,
};
use tracing::warn;

use types::foldables::authority::{RollupsInitialState, RollupsState};

use anyhow::{anyhow, Result};
use std::sync::Arc;
use tokio_stream::{Stream, StreamExt};
use tonic::transport::Channel;
use url::Url;

const BUFFER_LEN: usize = 256;

const MAX_RETRIES: u32 = 10;
const INITIAL_BACKOFF: u64 = 1000;

pub async fn create_state_server(
    config: &SCConfig,
) -> Result<
    impl StateServer<InitialState = RollupsInitialState, State = RollupsState>
        + BlockServer,
> {
    let channel = Channel::from_shared(config.grpc_endpoint.to_owned())?
        .connect()
        .await?;

    Ok(GrpcStateFoldClient::new_from_channel(channel))
}

pub async fn create_block_subscription(
    client: &impl BlockServer,
    confirmations: usize,
) -> Result<
    impl Stream<Item = Result<BlockStreamItem, StateServerError>>
        + Send
        + std::marker::Unpin,
> {
    let s = client.subscribe_blocks(confirmations).await?;

    let s = {
        use futures::StreamExt;
        s.ready_chunks(BUFFER_LEN)
    };

    let s = s.filter_map(|mut x| {
        if x.len() == BUFFER_LEN {
            None
        } else {
            let a = x.pop();
            a
        }
    });

    Ok(s)
}

pub async fn create_tx_sender(
    config: &TxManagerConfig,
    consensus_address: Address,
    priority: Priority,
) -> Result<impl TxSender> {
    let tx_manager = {
        let provider = {
            let http = Http::new(Url::parse(&config.provider_http_endpoint)?);

            let retry_client = RetryClient::new(
                http,
                Box::new(HttpRateLimitRetryPolicy),
                MAX_RETRIES,
                INITIAL_BACKOFF,
            );

            let provider = Provider::new(retry_client);

            Arc::new(SignerMiddleware::new(provider, config.wallet.clone()))
        };

        let tx_manager = match TransactionManager::new(
            provider.clone(),
            DefaultGasOracle::new(),
            FileSystemDatabase::new(config.database_path.to_owned()),
            config.into(),
            ManagerConfiguration::default(),
        )
        .await
        {
            Ok((m, _)) => m,

            Err(eth_tx_manager::Error::NonceTooLow { .. }) => {
                warn!("Nonce too low! Clearing tx database");

                TransactionManager::force_new(
                    provider,
                    DefaultGasOracle::new(),
                    FileSystemDatabase::new(config.database_path.to_owned()),
                    config.into(),
                    ManagerConfiguration::default(),
                )
                .await?
            }

            Err(e) => return Err(anyhow!(e)),
        };

        tx_manager
    };

    Ok(BulletproofTxSender::new(
        tx_manager,
        config.default_confirmations,
        priority,
        config.wallet.address(),
        consensus_address,
    ))
}

pub async fn create_context(
    config: &DispatcherConfig,
    block_server: &impl BlockServer,
    broker: &impl BrokerStatus,
) -> Result<Context> {
    let genesis_timestamp: u64 = block_server
        .query_block(config.dapp_deployment.deploy_block_hash)
        .await?
        .timestamp
        .as_u64();
    let epoch_length = config.epoch_duration;
    let context = Context::new(genesis_timestamp, epoch_length, broker).await?;

    Ok(context)
}
