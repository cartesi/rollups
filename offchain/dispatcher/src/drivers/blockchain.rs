use crate::machine::BrokerReceive;
use crate::tx_sender::TxSender;

use state_fold_types::ethereum_types::Address;
use types::foldables::claims::History;

use anyhow::Result;

use tracing::{info, instrument, trace};

#[derive(Debug)]
pub struct BlockchainDriver {
    dapp_address: Address,
}

impl BlockchainDriver {
    pub fn new(dapp_address: Address) -> Self {
        Self { dapp_address }
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn react<TS: TxSender + Sync + Send>(
        &self,
        history: &History,
        broker: &impl BrokerReceive,
        mut tx_sender: TS,
    ) -> Result<TS> {
        let claims_sent = claims_sent(history, &self.dapp_address);
        trace!(?claims_sent);

        while let Some(claim) = broker.next_claim().await? {
            trace!("Got claim `{:?}` from broker", claim);

            if claim.number > claims_sent {
                info!("Sending claim `{:?}`", claim);
                tx_sender = tx_sender.send_claim_tx(&claim.hash).await?;
            }
        }

        Ok(tx_sender)
    }
}

fn claims_sent(history: &History, dapp_address: &Address) -> u64 {
    match history.dapp_claims.get(dapp_address) {
        Some(c) => c.claims.len() as u64,
        None => 0,
    }
}
