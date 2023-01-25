use crate::machine::{BrokerSend, BrokerStatus};

use anyhow::Result;
use types::foldables::input_box::Input;

#[derive(Debug)]
pub struct Context {
    inputs_sent_count: u64,
    last_event_is_finish_epoch: bool,
    last_timestamp: u64,

    // constants
    genesis_timestamp: u64,
    epoch_length: u64,
}

impl Context {
    pub async fn new(
        genesis_timestamp: u64,
        epoch_length: u64,
        broker: &impl BrokerStatus,
    ) -> Result<Self> {
        let status = broker.status().await?;

        Ok(Self {
            inputs_sent_count: status.inputs_sent_count,
            last_event_is_finish_epoch: status.last_event_is_finish_epoch,
            last_timestamp: genesis_timestamp,
            genesis_timestamp,
            epoch_length,
        })
    }

    pub fn inputs_sent_count(&self) -> u64 {
        self.inputs_sent_count
    }

    pub async fn finish_epoch_if_needed(
        &mut self,
        event_timestamp: u64,
        broker: &impl BrokerSend,
    ) -> Result<()> {
        if self.should_finish_epoch(event_timestamp) {
            self.finish_epoch(event_timestamp, broker).await?;
        }
        Ok(())
    }

    pub async fn enqueue_input(
        &mut self,
        input: &Input,
        broker: &impl BrokerSend,
    ) -> Result<()> {
        broker.enqueue_input(self.inputs_sent_count, input).await?;
        self.increment_input();
        Ok(())
    }
}

impl Context {
    fn calculate_epoch_for(&self, timestamp: u64) -> u64 {
        assert!(timestamp >= self.genesis_timestamp);
        (timestamp - self.genesis_timestamp) / self.epoch_length
    }

    // This logic works because we call this function with `event_timestamp` being equal to the
    // timestamp of each individual input, rather than just the latest from the blockchain.
    fn should_finish_epoch(&self, event_timestamp: u64) -> bool {
        if self.last_event_is_finish_epoch {
            return false;
        }
        let current_epoch = self.calculate_epoch_for(self.last_timestamp);
        let event_epoch = self.calculate_epoch_for(event_timestamp);
        event_epoch > current_epoch
    }

    async fn finish_epoch(
        &mut self,
        event_timestamp: u64,
        broker: &impl BrokerSend,
    ) -> Result<()> {
        broker.finish_epoch(self.inputs_sent_count).await?;
        self.increment_epoch(event_timestamp);
        Ok(())
    }

    fn increment_input(&mut self) {
        self.inputs_sent_count += 1;
        self.last_event_is_finish_epoch = false;
    }

    fn increment_epoch(&mut self, event_timestamp: u64) {
        self.last_timestamp = event_timestamp;
        self.last_event_is_finish_epoch = true;
    }
}
