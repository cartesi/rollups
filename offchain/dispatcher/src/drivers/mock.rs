use anyhow::Result;
use async_trait::async_trait;
use std::{collections::VecDeque, ops::DerefMut, sync::Mutex};
use types::foldables::input_box::Input;

use crate::machine::{
    BrokerReceive, BrokerSend, BrokerStatus, RollupClaim, RollupStatus,
};

// ------------------------------------------------------------------------------------------------
// Broker
// ------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Broker {
    pub rollup_statuses: Mutex<VecDeque<RollupStatus>>,
    pub next_claims: Mutex<VecDeque<RollupClaim>>,
}

impl Broker {
    pub fn new(
        rollup_statuses: Vec<RollupStatus>,
        next_claims: Vec<RollupClaim>,
    ) -> Self {
        Self {
            rollup_statuses: Mutex::new(rollup_statuses.into()),
            next_claims: Mutex::new(next_claims.into()),
        }
    }
}

#[async_trait]
impl BrokerStatus for Broker {
    async fn status(&self) -> Result<RollupStatus> {
        let mut mutex_guard = self.rollup_statuses.lock().unwrap();
        Ok(mutex_guard.deref_mut().pop_front().unwrap())
    }
}

#[async_trait]
impl BrokerReceive for Broker {
    async fn next_claim(&self) -> Result<Option<RollupClaim>> {
        let mut mutex_guard = self.next_claims.lock().unwrap();
        Ok(mutex_guard.deref_mut().pop_front())
    }
}

#[async_trait]
impl BrokerSend for Broker {
    async fn enqueue_input(
        &self,
        input_index: u64,
        input: &Input,
    ) -> Result<()> {
        // TODO
        Ok(())
    }

    async fn finish_epoch(&self, inputs_sent_count: u64) -> Result<()> {
        // TODO
        Ok(())
    }
}

// ------------------------------------------------------------------------------------------------
// TxSender
// ------------------------------------------------------------------------------------------------

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
