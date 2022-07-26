mod bulletproof_tx_sender;

pub use bulletproof_tx_sender::BulletproofTxSender;

use anyhow::Result;
use async_trait::async_trait;
use state_fold_types::ethers::types::{H256, U256};

#[async_trait]
pub trait TxSender: std::fmt::Debug + Sized {
    async fn send_claim_tx(
        self,
        claim: H256,
        epoch_number: U256,
    ) -> Result<Self>;

    async fn send_finalize_tx(self, epoch_number: U256) -> Result<Self>;

    async fn send_redeem_tx(self, validator_redeemed: U256) -> Result<Self>;
}
