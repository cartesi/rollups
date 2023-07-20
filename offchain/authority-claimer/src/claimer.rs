// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use async_trait::async_trait;
use tracing::{trace, warn};

use crate::{listener::BrokerListener, sender::ClaimSender};

/// The `AuthorityClaimer` starts an event loop that waits for claim messages
/// from the broker, and then sends the claims to the blockchain.
///
/// It uses a `BrokerListener` for listening for messages from the broker.
///
/// It also uses a `ClaimSender` that interacts with the blockchain and
/// effectively submits the claims.
#[async_trait]
pub trait AuthorityClaimer<'a, L: BrokerListener, S: ClaimSender>
where
    L: 'a + Sync,
    S: 'a,
{
    async fn start(
        &'a self,
        broker_listener: L,
        claim_sender: S,
    ) -> Result<(), AuthorityClaimerError<S, L>> {
        trace!("Starting the authority claimer loop");
        let mut claim_sender = claim_sender;
        loop {
            match broker_listener.listen().await {
                Ok(rollups_claim) => {
                    trace!("Got a claim from the broker: {:?}", rollups_claim);
                    claim_sender = claim_sender
                        .send_claim(rollups_claim)
                        .await
                        .map_err(AuthorityClaimerError::ClaimSenderError)?;
                }
                Err(e) => {
                    warn!("Broker error `{}`", e);
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthorityClaimerError<S: ClaimSender, L: BrokerListener> {
    #[error("claim sender error: {0}")]
    ClaimSenderError(S::Error),

    #[error("broker listener error: {0}")]
    BrokerListenerError(L::Error),
}

// ------------------------------------------------------------------------------------------------
// DefaultAuthorityClaimer
// ------------------------------------------------------------------------------------------------

pub struct DefaultAuthorityClaimer;

impl DefaultAuthorityClaimer {
    pub fn new() -> Self {
        Self
    }
}

impl<'a, L: BrokerListener, S: ClaimSender> AuthorityClaimer<'a, L, S>
    for DefaultAuthorityClaimer
where
    L: 'a + Sync,
    S: 'a,
{
}
