use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{instrument, trace};

use eth_tx_manager::{
    database::FileSystemDatabase as TxDatabase,
    gas_oracle::DefaultGasOracle as GasOracle,
    time::DefaultTime,
    transaction::{Priority, Transaction, Value},
    TransactionManager,
};

use contracts::{authority::Authority, history::Claim};
use rollups_events::RollupsClaim;
use state_fold_types::{
    ethabi::Token,
    ethers::{
        self,
        abi::AbiEncode,
        prelude::NameOrAddress,
        providers::{Middleware, MockProvider, Provider},
        types::{Address, Bytes},
    },
};

use super::TxSender;

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
    async fn submit_claim(
        self,
        dapp_address: Address,
        rollups_claim: RollupsClaim,
    ) -> Result<Self> {
        let claim_tx = {
            let call = self
                .constants
                .authority_consensus
                .submit_claim(
                    SubmittableClaim(dapp_address, rollups_claim).into(),
                )
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
