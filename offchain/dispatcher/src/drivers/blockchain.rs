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
    use rand::Rng;
    use state_fold_types::ethereum_types::{Address, H160, H256};
    use std::sync::Arc;
    use types::foldables::claims::{Claim, DAppClaims, History};

    use crate::machine::RollupClaim;

    use super::BlockchainDriver;

    /* ========================================================================================= */

    #[test]
    fn test_new() {
        let dapp_address = H160::default();
        let blockchain_driver = BlockchainDriver::new(dapp_address);
        assert_eq!(blockchain_driver.dapp_address, dapp_address);
    }

    /* ========================================================================================= */

    fn random_claim() -> Claim {
        let mut rng = rand::thread_rng();
        let start_input_index = rng.gen();
        Claim {
            epoch_hash: H256::random(),
            start_input_index,
            end_input_index: start_input_index + 5,
            claim_timestamp: rng.gen(),
        }
    }

    fn random_claims(n: usize) -> Vec<Claim> {
        let mut claims = Vec::new();
        claims.resize_with(n, || random_claim());
        claims
    }

    fn new_history() -> History {
        History {
            history_address: Arc::new(H160::random()),
            dapp_claims: Arc::new(hashmap! {}),
        }
    }

    fn update_history(
        history: &History,
        dapp_address: Address,
        n: usize,
    ) -> History {
        let claims = random_claims(n)
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

    #[test]
    fn test_claims_sent_some_0() {
        let dapp_address = H160::random();
        let history = new_history();
        let history = update_history(&history, dapp_address, 0);
        let n = super::claims_sent(&history, &dapp_address);
        assert_eq!(n, 0);
    }

    #[test]
    fn test_claims_sent_some_1() {
        let dapp_address1 = H160::random();
        let dapp_address2 = H160::random();
        let history = new_history();
        let history = update_history(&history, dapp_address1, 0);
        let history = update_history(&history, dapp_address2, 1);
        let n = super::claims_sent(&history, &dapp_address1);
        assert_eq!(n, 0);
        let n = super::claims_sent(&history, &dapp_address2);
        assert_eq!(n, 1);
    }

    #[test]
    fn test_claims_sent_some_n() {
        let dapp_address1 = H160::random();
        let dapp_address2 = H160::random();
        let history = new_history();
        let history = update_history(&history, dapp_address1, 5);
        let history = update_history(&history, dapp_address2, 2);
        let n = super::claims_sent(&history, &dapp_address1);
        assert_eq!(n, 5);
        let n = super::claims_sent(&history, &dapp_address2);
        assert_eq!(n, 2);
    }

    #[test]
    fn test_claims_sent_none() {
        let dapp_address1 = H160::random();
        let dapp_address2 = H160::random();
        let history = new_history();
        let history = update_history(&history, dapp_address1, 1);
        let n = super::claims_sent(&history, &dapp_address2);
        assert_eq!(n, 0);
    }

    /* ========================================================================================= */

    mod mock {
        use anyhow::Result;
        use async_trait::async_trait;
        use std::{collections::VecDeque, ops::DerefMut, sync::Mutex};

        use crate::machine::{BrokerReceive, RollupClaim};

        #[derive(Debug)]
        pub struct Broker {
            pub next_claims: Mutex<VecDeque<Option<RollupClaim>>>,
        }

        impl Broker {
            pub fn new(next_claims: Vec<Option<RollupClaim>>) -> Self {
                Self {
                    next_claims: Mutex::new(next_claims.into()),
                }
            }
        }

        #[async_trait]
        impl BrokerReceive for Broker {
            async fn next_claim(&self) -> Result<Option<RollupClaim>> {
                let mut mutex_guard = self.next_claims.lock().unwrap();
                Ok(mutex_guard.deref_mut().pop_front().unwrap_or_default())
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

            pub fn count(&self) -> usize {
                self.sent_claims.lock().unwrap().len()
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

    fn new_rollup_claim(number: u64) -> RollupClaim {
        let mut rng = rand::thread_rng();
        let hash = (0..32).map(|_| rng.gen()).collect::<Vec<u8>>();
        assert_eq!(hash.len(), 32);
        let hash: [u8; 32] = hash.try_into().unwrap();
        RollupClaim { hash, number }
    }

    fn new_history_for_react_tests(
        dapp_address1: Address,
        dapp_address2: Address,
    ) -> History {
        let history = new_history();
        let history = update_history(&history, dapp_address1, 5);
        let history = update_history(&history, dapp_address2, 2);
        history
    }

    /* ========================================================================================= */

    #[tokio::test]
    async fn test_react_no_claim() {
        let dapp_address = H160::random();
        let other_dapp_address = H160::random();
        let history =
            new_history_for_react_tests(dapp_address, other_dapp_address);

        let broker = mock::Broker::new(vec![None]);
        let tx_sender = mock::TxSender::new();
        let blockchain_driver = BlockchainDriver::new(dapp_address);

        let result =
            blockchain_driver.react(&history, &broker, tx_sender).await;
        assert!(result.is_ok());
        let tx_sender = result.unwrap();
        assert_eq!(tx_sender.count(), 0);
    }

    // broker has 1 (new) claim -- sent 1 claim
    #[tokio::test]
    async fn test_react_1_new_claim_sent_1_claim() {
        let dapp_address = H160::random();
        let other_dapp_address = H160::random();
        let history =
            new_history_for_react_tests(dapp_address, other_dapp_address);

        let broker = mock::Broker::new(vec![Some(new_rollup_claim(6))]);
        let tx_sender = mock::TxSender::new();
        let blockchain_driver = BlockchainDriver::new(dapp_address);

        let tx_sender =
            blockchain_driver.react(&history, &broker, tx_sender).await;
        assert!(tx_sender.is_ok());
        assert_eq!(tx_sender.unwrap().count(), 1);
    }

    // broker has 1 (old) claim -- sent 0 claims
    #[tokio::test]
    async fn test_react_1_old_claim_sent_0_claims() {
        let dapp_address = H160::random();
        let other_dapp_address = H160::random();
        let history =
            new_history_for_react_tests(dapp_address, other_dapp_address);

        let broker = mock::Broker::new(vec![Some(new_rollup_claim(5))]);
        let tx_sender = mock::TxSender::new();
        let blockchain_driver = BlockchainDriver::new(dapp_address);

        let tx_sender =
            blockchain_driver.react(&history, &broker, tx_sender).await;
        assert!(tx_sender.is_ok());
        assert_eq!(tx_sender.unwrap().count(), 0);
    }

    // broker has 2 claims (1 old, 1 new) -- sent 1 claim
    #[tokio::test]
    async fn test_react_2_claims_sent_1_claim() {
        let dapp_address = H160::random();
        let other_dapp_address = H160::random();
        let history =
            new_history_for_react_tests(dapp_address, other_dapp_address);

        let broker = mock::Broker::new(vec![
            Some(new_rollup_claim(5)),
            Some(new_rollup_claim(6)),
        ]);
        let tx_sender = mock::TxSender::new();
        let blockchain_driver = BlockchainDriver::new(dapp_address);

        let tx_sender =
            blockchain_driver.react(&history, &broker, tx_sender).await;
        assert!(tx_sender.is_ok());
        assert_eq!(tx_sender.unwrap().count(), 1);
    }

    // broker has interleaved old and new claims -- sent 5 new claims
    #[tokio::test]
    async fn test_react_interleaved_old_new_claims_sent_5_claims() {
        let dapp_address = H160::random();
        let other_dapp_address = H160::random();
        let history =
            new_history_for_react_tests(dapp_address, other_dapp_address);

        let broker = mock::Broker::new(vec![
            Some(new_rollup_claim(1)),
            Some(new_rollup_claim(5)),
            Some(new_rollup_claim(6)),
            Some(new_rollup_claim(2)),
            Some(new_rollup_claim(3)),
            Some(new_rollup_claim(7)),
            Some(new_rollup_claim(8)),
            Some(new_rollup_claim(4)),
            Some(new_rollup_claim(5)), // duplicate
            Some(new_rollup_claim(9)),
            Some(new_rollup_claim(10)),
        ]);
        let tx_sender = mock::TxSender::new();
        let blockchain_driver = BlockchainDriver::new(dapp_address);

        let tx_sender =
            blockchain_driver.react(&history, &broker, tx_sender).await;
        assert!(tx_sender.is_ok());
        assert_eq!(tx_sender.unwrap().count(), 5);
    }
}
