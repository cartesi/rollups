pub mod config;
pub mod rollups_broker;

use types::foldables::input_box::Input;

use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug)]
pub struct RollupStatus {
    pub inputs_sent_count: u64,
    pub last_event_is_finish_epoch: bool,
}

impl Default for RollupStatus {
    fn default() -> Self {
        RollupStatus {
            inputs_sent_count: 0,
            last_event_is_finish_epoch: false,
        }
    }
}

#[derive(Debug)]
pub struct RollupClaim {
    pub hash: [u8; 32],
    pub number: u64,
}

#[async_trait]
pub trait BrokerStatus: std::fmt::Debug {
    async fn status(&self) -> Result<RollupStatus>;
}

#[async_trait]
pub trait BrokerSend: std::fmt::Debug {
    async fn enqueue_input(
        &self,
        input_index: u64,
        input: &Input,
    ) -> Result<()>;
    async fn finish_epoch(&self, inputs_sent_count: u64) -> Result<()>;
}

#[async_trait]
pub trait BrokerReceive: std::fmt::Debug {
    async fn next_claim(&self) -> Result<Option<RollupClaim>>;
}
