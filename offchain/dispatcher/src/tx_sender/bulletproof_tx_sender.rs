use super::TxSender;

use contracts::{
    fee_manager_facet::FeeManagerFacet, rollups_facet::RollupsFacet,
};

use tx_manager::{
    database::FileSystemDatabase as TxDatabase,
    gas_oracle::ETHGasStationOracle as GasOracle,
    time::DefaultTime,
    transaction::{Priority, Transaction, Value},
    TransactionManager,
};

use state_fold_types::ethers::{
    prelude::NameOrAddress,
    providers::{Middleware, MockProvider, Provider},
    types::{Address, H256, U256},
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
    rollups_facet: RollupsFacet<Provider<MockProvider>>,
    fee_manager_facet: FeeManagerFacet<Provider<MockProvider>>,
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

            let rollups_facet =
                RollupsFacet::new(dapp_address, Arc::clone(&provider));

            let fee_manager_facet =
                FeeManagerFacet::new(dapp_address, Arc::clone(&provider));

            Constants {
                confirmations,
                priority,
                sender_address,
                rollups_facet,
                fee_manager_facet,
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
    async fn send_claim_tx(
        self,
        claim: H256,
        _epoch_number: U256,
    ) -> Result<Self> {
        let claim_tx = {
            let call = self
                .constants
                .rollups_facet
                .claim(claim.to_fixed_bytes())
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

    #[instrument(level = "trace")]
    async fn send_finalize_tx(self, _epoch_number: U256) -> Result<Self> {
        let finalize_tx = {
            let call = self
                .constants
                .rollups_facet
                .finalize_epoch()
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

        trace!("Built finalize transaction: `{:?}`", finalize_tx);

        let (tx_manager, receipt) = {
            let tx_manager = self.tx_manager;
            tx_manager
                .send_transaction(
                    finalize_tx,
                    self.constants.confirmations,
                    self.constants.priority,
                )
                .await?
        };

        trace!("Finalize transaction confirmed: `{:?}`", receipt);

        Ok(Self {
            tx_manager,
            constants: self.constants,
        })
    }

    #[instrument(level = "trace")]
    async fn send_redeem_tx(self) -> Result<Self> {
        let redeem_tx = {
            let call = self
                .constants
                .fee_manager_facet
                .redeem_fee(self.constants.sender_address)
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

        trace!("Built redeem transaction: `{:?}`", redeem_tx);

        let (tx_manager, receipt) = {
            let tx_manager = self.tx_manager;
            tx_manager
                .send_transaction(
                    redeem_tx,
                    self.constants.confirmations,
                    self.constants.priority,
                )
                .await?
        };

        trace!("Redeem transaction confirmed: `{:?}`", receipt);

        Ok(Self {
            tx_manager,
            constants: self.constants,
        })
    }
}
