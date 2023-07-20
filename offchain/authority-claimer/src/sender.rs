// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use async_trait::async_trait;
use rollups_events::{DAppMetadata, RollupsClaim};
use snafu::Snafu;
use std::fmt::Debug;

use crate::metrics::AuthorityClaimerMetrics;

/// The `ClaimSender` sends claims to the blockchain.
///
/// It should wait for N blockchain confirmations.
#[async_trait]
pub trait ClaimSender: Sized + Send + Debug {
    type Error: snafu::Error + Send;

    /// The `send_claim` function consumes the `ClaimSender` object
    /// and then returns it to avoid that processes use the claim sender
    /// concurrently.
    async fn send_claim(
        self,
        rollups_claim: RollupsClaim,
    ) -> Result<Self, Self::Error>;
}

// ------------------------------------------------------------------------------------------------
// TxManagerClaimSender
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TxManagerClaimSender;

#[derive(Debug, Snafu)]
pub enum TxManagerClaimSenderError {
    Todo,
}

impl TxManagerClaimSender {
    pub fn new(
        _dapp_metadata: DAppMetadata,
        _metrics: AuthorityClaimerMetrics,
    ) -> Result<TxManagerClaimSender, TxManagerClaimSenderError> {
        todo!()
    }
}

#[async_trait]
impl ClaimSender for TxManagerClaimSender {
    type Error = TxManagerClaimSenderError;

    async fn send_claim(
        self,
        _rollups_claim: RollupsClaim,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}
