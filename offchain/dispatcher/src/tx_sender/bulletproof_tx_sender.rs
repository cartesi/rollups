use super::TxSender;

use contracts::authority::Authority;

use eth_tx_manager::{
    database::FileSystemDatabase as TxDatabase,
    gas_oracle::DefaultGasOracle as GasOracle,
    time::DefaultTime,
    transaction::{Priority, Transaction, Value},
    TransactionManager,
};

use state_fold_types::ethers::{
    prelude::NameOrAddress,
    providers::{Middleware, MockProvider, Provider},
    types::Address,
};

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{instrument, trace};

#[derive(Debug)]
pub struct BulletproofTxSender<M>
where
    M: Middleware + Send + Sync + 'static,
{
    tx_manager: TransactionManager<M, GasOracle, TxDatabase, DefaultTime>,
    constants: Constants,
}

#[derive(Debug)]
struct Constants {
    confirmations: usize,
    priority: Priority,
    sender_address: Address,
    authority_consensus: Authority<Provider<MockProvider>>,
}

impl<M> BulletproofTxSender<M>
where
    M: Middleware + Send + Sync + 'static,
{
    pub fn new(
        tx_manager: TransactionManager<M, GasOracle, TxDatabase, DefaultTime>,
        confirmations: usize,
        priority: Priority,
        sender_address: Address,
        dapp_address: Address,
    ) -> Self {
        let constants = {
            let (provider, _mock) = Provider::mocked();
            let provider = Arc::new(provider);
            let authority_consensus = Authority::new(dapp_address, provider);

            Constants {
                confirmations,
                priority,
                sender_address,
                authority_consensus,
            }
        };

        Self {
            tx_manager,
            constants,
        }
    }
}

#[async_trait]
impl<M> TxSender for BulletproofTxSender<M>
where
    M: Middleware + Send + Sync + 'static,
{
    #[instrument(level = "trace")]
    async fn send_claim_tx(self, claim: &[u8; 32]) -> Result<Self> {
        let claim_tx = {
            let call = self
                .constants
                .authority_consensus
                .submit_claim(claim.into())
                .from(self.constants.sender_address);

            Transaction {
                from: *call.tx.from().expect("tx `from` should not be null"),
                to: match call.tx.to().expect("tx `to` should not be null") {
                    NameOrAddress::Address(a) => *a,
                    _ => panic!("expected address, found ENS name"),
                },
                value: Value::Nothing,
                call_data: call.tx.data().cloned(),
            }
        };

        trace!("Built claim transaction: `{:?}`", claim_tx);

        let (tx_manager, receipt) = {
            let tx_manager = self.tx_manager;
            tx_manager
                .send_transaction(
                    claim_tx,
                    self.constants.confirmations,
                    self.constants.priority,
                )
                .await?
        };

        trace!("Claim transaction confirmed: `{:?}`", receipt);

        Ok(Self {
            tx_manager,
            constants: self.constants,
        })
    }
}
