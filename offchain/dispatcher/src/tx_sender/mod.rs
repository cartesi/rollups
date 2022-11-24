mod bulletproof_tx_sender;

pub use bulletproof_tx_sender::BulletproofTxSender;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TxSender: std::fmt::Debug + Sized {
    async fn send_claim_tx(self, claim: &[u8; 32]) -> Result<Self>;
}
