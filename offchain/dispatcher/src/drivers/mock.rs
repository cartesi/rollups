// Copyright Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use async_trait::async_trait;
use im::{hashmap, Vector};
use rollups_events::RollupsClaim;
use snafu::whatever;
use state_fold_types::{
    ethereum_types::{Address, Bloom, H160, H256},
    Block,
};
use std::{
    collections::VecDeque,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
};
use types::foldables::{
    claims::{Claim, DAppClaims, History},
    input_box::{DAppInputBox, Input, InputBox},
};

use crate::{
    machine::{
        rollups_broker::BrokerFacadeError, BrokerReceive, BrokerSend,
        BrokerStatus, RollupStatus,
    },
    sender::SenderError,
};

// ------------------------------------------------------------------------------------------------
// auxiliary functions
// ------------------------------------------------------------------------------------------------

fn new_claims(n: usize) -> Vec<Claim> {
    let mut claims = Vec::new();
    let mut i = 0;
    claims.resize_with(n, || {
        let claim = Claim {
            epoch_hash: H256::random(),
            start_input_index: i,
            end_input_index: i,
            claim_timestamp: i as u64,
        };
        i = i + 1;
        claim
    });
    claims
}

pub fn new_history() -> History {
    History {
        history_address: Arc::new(H160::random()),
        dapp_claims: Arc::new(hashmap! {}),
    }
}

pub fn update_history(
    history: &History,
    dapp_address: Address,
    n: usize,
) -> History {
    let claims = new_claims(n)
        .iter()
        .map(|x| Arc::new(x.clone()))
        .collect::<Vec<_>>();
    let claims = Vector::from(claims);
    let dapp_claims = history
        .dapp_claims
        .update(Arc::new(dapp_address), Arc::new(DAppClaims { claims }));
    History {
        history_address: history.history_address.clone(),
        dapp_claims: Arc::new(dapp_claims),
    }
}

pub fn new_block(timestamp: u32) -> Block {
    Block {
        hash: H256::random(),
        number: 0.into(),
        parent_hash: H256::random(),
        timestamp: timestamp.into(),
        logs_bloom: Bloom::default(),
    }
}

pub fn new_input(timestamp: u32) -> Input {
    Input {
        sender: Arc::new(H160::random()),
        payload: vec![],
        block_added: Arc::new(new_block(timestamp)),
        dapp: Arc::new(H160::random()),
        tx_hash: Arc::new(H256::default()),
    }
}

pub fn new_input_box() -> InputBox {
    InputBox {
        input_box_address: Arc::new(H160::random()),
        dapp_input_boxes: Arc::new(hashmap! {}),
    }
}

pub fn update_input_box(
    input_box: InputBox,
    dapp_address: Address,
    timestamps: Vec<u32>,
) -> InputBox {
    let inputs = timestamps
        .iter()
        .map(|timestamp| Arc::new(new_input(*timestamp)))
        .collect::<Vec<_>>();
    let inputs = Vector::from(inputs);
    let dapp_input_boxes = input_box
        .dapp_input_boxes
        .update(Arc::new(dapp_address), Arc::new(DAppInputBox { inputs }));
    InputBox {
        input_box_address: input_box.input_box_address,
        dapp_input_boxes: Arc::new(dapp_input_boxes),
    }
}

// ------------------------------------------------------------------------------------------------
// Broker
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SendInteraction {
    EnqueuedInput(u64),
    FinishedEpoch(u64),
}

#[derive(Debug)]
pub struct Broker {
    pub rollup_statuses: Mutex<VecDeque<RollupStatus>>,
    pub next_claims: Mutex<VecDeque<RollupsClaim>>,
    pub send_interactions: Mutex<Vec<SendInteraction>>,
    status_error: bool,
    enqueue_input_error: bool,
    finish_epoch_error: bool,
}

impl Broker {
    fn default() -> Self {
        Self {
            rollup_statuses: Mutex::new(VecDeque::new()),
            next_claims: Mutex::new(VecDeque::new()),
            send_interactions: Mutex::new(Vec::new()),
            status_error: false,
            enqueue_input_error: false,
            finish_epoch_error: false,
        }
    }

    pub fn new(
        rollup_statuses: Vec<RollupStatus>,
        next_claims: Vec<RollupsClaim>,
    ) -> Self {
        let mut broker = Self::default();
        broker.rollup_statuses = Mutex::new(rollup_statuses.into());
        broker.next_claims = Mutex::new(next_claims.into());
        broker
    }

    pub fn with_status_error() -> Self {
        let mut broker = Self::default();
        broker.status_error = true;
        broker
    }

    pub fn with_enqueue_input_error() -> Self {
        let mut broker = Self::default();
        broker.enqueue_input_error = true;
        broker
    }

    pub fn with_finish_epoch_error() -> Self {
        let mut broker = Self::default();
        broker.finish_epoch_error = true;
        broker
    }

    fn send_interactions_len(&self) -> usize {
        let mutex_guard = self.send_interactions.lock().unwrap();
        mutex_guard.deref().len()
    }

    fn get_send_interaction(&self, i: usize) -> SendInteraction {
        let mutex_guard = self.send_interactions.lock().unwrap();
        mutex_guard.deref().get(i).unwrap().clone()
    }

    pub fn assert_send_interactions(&self, expected: Vec<SendInteraction>) {
        assert_eq!(
            self.send_interactions_len(),
            expected.len(),
            "{:?}",
            self.send_interactions
        );
        println!("Send interactions:");
        for (i, expected) in expected.iter().enumerate() {
            let send_interaction = self.get_send_interaction(i);
            println!("{:?} - {:?}", send_interaction, expected);
            assert_eq!(send_interaction, *expected);
        }
    }
}

#[async_trait]
impl BrokerStatus for Broker {
    async fn status(&self) -> Result<RollupStatus, BrokerFacadeError> {
        if self.status_error {
            whatever!("status error")
        } else {
            let mut mutex_guard = self.rollup_statuses.lock().unwrap();
            Ok(mutex_guard.deref_mut().pop_front().unwrap())
        }
    }
}

#[async_trait]
impl BrokerReceive for Broker {
    async fn next_claim(
        &self,
    ) -> Result<Option<RollupsClaim>, BrokerFacadeError> {
        let mut mutex_guard = self.next_claims.lock().unwrap();
        Ok(mutex_guard.deref_mut().pop_front())
    }
}

#[async_trait]
impl BrokerSend for Broker {
    async fn enqueue_input(
        &self,
        input_index: u64,
        _: &Input,
    ) -> Result<(), BrokerFacadeError> {
        if self.enqueue_input_error {
            whatever!("enqueue_input error")
        } else {
            let mut mutex_guard = self.send_interactions.lock().unwrap();
            mutex_guard
                .deref_mut()
                .push(SendInteraction::EnqueuedInput(input_index));
            Ok(())
        }
    }

    async fn finish_epoch(
        &self,
        inputs_sent_count: u64,
    ) -> Result<(), BrokerFacadeError> {
        if self.finish_epoch_error {
            whatever!("finish_epoch error")
        } else {
            let mut mutex_guard = self.send_interactions.lock().unwrap();
            mutex_guard
                .deref_mut()
                .push(SendInteraction::FinishedEpoch(inputs_sent_count));
            Ok(())
        }
    }
}

// ------------------------------------------------------------------------------------------------
// TxSender
// ------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Sender {
    pub sent_rollups_claims: Mutex<Vec<(Address, RollupsClaim)>>,
}

impl Sender {
    pub fn new() -> Self {
        Self {
            sent_rollups_claims: Mutex::new(vec![]),
        }
    }

    pub fn count(&self) -> usize {
        self.sent_rollups_claims.lock().unwrap().len()
    }
}

#[async_trait]
impl crate::sender::Sender for Sender {
    async fn submit_claim(
        self,
        dapp_address: Address,
        rollups_claim: RollupsClaim,
    ) -> Result<Self, SenderError> {
        let mut mutex_guard = self.sent_rollups_claims.lock().unwrap();
        mutex_guard.deref_mut().push((dapp_address, rollups_claim));
        drop(mutex_guard);
        Ok(self)
    }
}
