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

#[cfg(test)]
mod tests {
    use im::{hashmap, Vector};
    use state_fold_types::ethereum_types::{Address, H160};
    use std::sync::Arc;
    use types::foldables::claims::{Claim, DAppClaims, History};

    use super::BlockchainDriver;

    #[test]
    fn test_new() {
        let dapp_address = H160::default();
        let blockchain_driver = BlockchainDriver::new(dapp_address);
        assert_eq!(blockchain_driver.dapp_address, dapp_address);
    }

    /* ========================================================================================= */

    mod mock {
        use anyhow::Result;
        use async_trait::async_trait;
        use std::{ops::DerefMut, sync::Mutex};

        use crate::machine::{BrokerReceive, RollupClaim};

        #[derive(Debug)]
        pub struct Broker {
            pub next_claims: Mutex<Vec<Option<RollupClaim>>>,
        }

        impl Broker {
            pub fn new(next_claims: Vec<Option<RollupClaim>>) -> Self {
                Self {
                    next_claims: Mutex::new(next_claims),
                }
            }
        }

        #[async_trait]
        impl BrokerReceive for Broker {
            async fn next_claim(&self) -> Result<Option<RollupClaim>> {
                let mut mutex_guard = self.next_claims.lock().unwrap();
                Ok(mutex_guard.deref_mut().pop().unwrap())
            }
        }

        #[derive(Debug)]
        pub struct TxSender {
            pub sent_claims: Mutex<Vec<[u8; 32]>>,
        }

        impl TxSender {
            pub fn new() -> Self {
                Self {
                    sent_claims: Mutex::new(vec![]),
                }
            }
        }

        #[async_trait]
        impl crate::tx_sender::TxSender for TxSender {
            async fn send_claim_tx(self, claim: &[u8; 32]) -> Result<Self> {
                let mut mutex_guard = self.sent_claims.lock().unwrap();
                mutex_guard.deref_mut().push(*claim);
                drop(mutex_guard);
                Ok(self)
            }
        }
    }

    fn new_history(history_address: Address) -> History {
        History {
            history_address: Arc::new(history_address),
            dapp_claims: Arc::new(hashmap! {}),
        }
    }

    fn update_history(
        history: &History,
        dapp_address: Address,
        claims: Vec<Claim>,
    ) -> History {
        let claims = claims
            .iter()
            .map(|x| Arc::new(x.clone()))
            .collect::<Vec<_>>();
        let claims = Vector::from(claims);
        let dapp_claims = history
            .dapp_claims
            .update(Arc::new(dapp_address), Arc::new(DAppClaims { claims }));
        History {
            history_address: history.history_address.clone(),
            dapp_claims: Arc::new(dapp_claims),
        }
    }

    /* ========================================================================================= */

    #[tokio::test]
    async fn test_react() {
        let dapp_address = H160::zero();

        let history = new_history(H160::default());
        let history =
            update_history(&history, dapp_address, vec![] /* TODO */);

        let broker = mock::Broker::new(vec![] /* TODO */);
        let tx_sender = mock::TxSender::new();

        let blockchain_driver = BlockchainDriver::new(dapp_address);
        let result =
            blockchain_driver.react(&history, &broker, tx_sender).await;
        assert!(result.is_ok());
    }
}
