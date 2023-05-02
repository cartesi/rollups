mod bulletproof_tx_sender;

pub use bulletproof_tx_sender::BulletproofTxSender;

use anyhow::Result;
use async_trait::async_trait;
use rollups_events::RollupsClaim;
use state_fold_types::ethers::types::Address;

#[async_trait]
pub trait TxSender: std::fmt::Debug + Sized {
    async fn submit_claim(
        self,
        dapp_address: Address,
        rollups_claim: RollupsClaim,
    ) -> Result<Self>;
}
