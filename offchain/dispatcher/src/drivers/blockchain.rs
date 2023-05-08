use anyhow::Result;
use tracing::{info, instrument, trace};

use state_fold_types::ethereum_types::Address;
use types::foldables::claims::History;

use crate::machine::BrokerReceive;
use crate::tx_sender::TxSender;

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

        while let Some(rollups_claim) = broker.next_claim().await? {
            trace!("Got claim `{:?}` from broker", rollups_claim);
            if rollups_claim.epoch_index >= claims_sent {
                info!("Sending claim `{:?}`", rollups_claim);
                tx_sender = tx_sender
                    .submit_claim(self.dapp_address, rollups_claim)
                    .await?;
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
    use rollups_events::{RollupsClaim, HASH_SIZE};
    use state_fold_types::ethereum_types::H160;

    use crate::drivers::mock;

    use super::BlockchainDriver;

    // --------------------------------------------------------------------------------------------
    // new
    // --------------------------------------------------------------------------------------------

    #[test]
    fn new() {
        let dapp_address = H160::default();
        let blockchain_driver = BlockchainDriver::new(dapp_address);
        assert_eq!(blockchain_driver.dapp_address, dapp_address);
    }

    // --------------------------------------------------------------------------------------------
    // claims_sent
    // --------------------------------------------------------------------------------------------

    #[test]
    fn claims_sent_some_0() {
        let dapp_address = H160::random();
        let history = mock::new_history();
        let history = mock::update_history(&history, dapp_address, 0);
        let n = super::claims_sent(&history, &dapp_address);
        assert_eq!(n, 0);
    }

    #[test]
    fn claims_sent_some_1() {
        let dapp_address1 = H160::random();
        let dapp_address2 = H160::random();
        let history = mock::new_history();
        let history = mock::update_history(&history, dapp_address1, 0);
        let history = mock::update_history(&history, dapp_address2, 1);
        let n = super::claims_sent(&history, &dapp_address1);
        assert_eq!(n, 0);
        let n = super::claims_sent(&history, &dapp_address2);
        assert_eq!(n, 1);
    }

    #[test]
    fn claims_sent_some_n() {
        let dapp_address1 = H160::random();
        let dapp_address2 = H160::random();
        let history = mock::new_history();
        let history = mock::update_history(&history, dapp_address1, 5);
        let history = mock::update_history(&history, dapp_address2, 2);
        let n = super::claims_sent(&history, &dapp_address1);
        assert_eq!(n, 5);
        let n = super::claims_sent(&history, &dapp_address2);
        assert_eq!(n, 2);
    }

    #[test]
    fn claims_sent_none() {
        let dapp_address1 = H160::random();
        let dapp_address2 = H160::random();
        let history = mock::new_history();
        let history = mock::update_history(&history, dapp_address1, 1);
        let n = super::claims_sent(&history, &dapp_address2);
        assert_eq!(n, 0);
    }

    // --------------------------------------------------------------------------------------------
    // react
    // --------------------------------------------------------------------------------------------

    async fn test_react(next_claims: Vec<u64>, n: usize) {
        let dapp_address = H160::random();
        let blockchain_driver = BlockchainDriver::new(dapp_address);

        let history = mock::new_history();
        let history = mock::update_history(&history, dapp_address, 5);
        let history = mock::update_history(&history, H160::random(), 2);

        let next_claims = next_claims
            .iter()
            .map(|i| RollupsClaim {
                epoch_hash: [*i as u8; HASH_SIZE].into(),
                epoch_index: *i,
                first_index: *i as u128,
                last_index: *i as u128,
            })
            .collect();
        let broker = mock::Broker::new(vec![], next_claims);
        let tx_sender = mock::TxSender::new();

        let result =
            blockchain_driver.react(&history, &broker, tx_sender).await;
        assert!(result.is_ok());
        let tx_sender = result.unwrap();
        assert_eq!(tx_sender.count(), n);
    }

    #[tokio::test]
    async fn react_no_claim() {
        test_react(vec![], 0).await;
    }

    // broker has 1 (new) claim -- sent 1 claim
    #[tokio::test]
    async fn react_1_new_claim_sent_1_claim() {
        test_react(vec![5], 1).await;
    }

    // broker has 1 (old) claim -- sent 0 claims
    #[tokio::test]
    async fn react_1_old_claim_sent_0_claims() {
        test_react(vec![4], 0).await;
    }

    // broker has 2 claims (1 old, 1 new) -- sent 1 claim
    #[tokio::test]
    async fn react_2_claims_sent_1_claim() {
        test_react(vec![4, 5], 1).await;
    }

    // broker has interleaved old and new claims -- sent 5 new claims
    #[tokio::test]
    async fn react_interleaved_old_new_claims_sent_5_claims() {
        test_react(vec![0, 4, 5, 1, 2, 6, 7, 3, 4, 8, 9], 5).await;
    }
}
