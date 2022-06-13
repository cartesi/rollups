use crate::{
    accumulating_epoch::AccumulatingEpoch, input::EpochInputState,
    FoldableError,
};
use anyhow::{Context, Error};
use async_trait::async_trait;
use contracts::rollups_facet::*;
use ethers::{
    prelude::EthEvent,
    providers::Middleware,
    types::{Address, H256, U256},
};
use im::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum SealedEpochState {
    SealedEpochNoClaims { sealed_epoch: AccumulatingEpoch },
    SealedEpochWithClaims { claimed_epoch: EpochWithClaims },
}

impl SealedEpochState {
    pub fn epoch_number(&self) -> U256 {
        match self {
            SealedEpochState::SealedEpochNoClaims { sealed_epoch } => {
                sealed_epoch.epoch_number
            }
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => {
                claimed_epoch.epoch_number
            }
        }
    }

    pub fn dapp_contract_address(&self) -> Address {
        match self {
            SealedEpochState::SealedEpochNoClaims { sealed_epoch } => {
                sealed_epoch.dapp_contract_address
            }
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => {
                claimed_epoch.dapp_contract_address
            }
        }
    }

    pub fn claims(&self) -> Option<Claims> {
        match self {
            SealedEpochState::SealedEpochNoClaims { .. } => None,
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => {
                Some(claimed_epoch.claims.clone())
            }
        }
    }

    pub fn inputs(&self) -> EpochInputState {
        match self {
            SealedEpochState::SealedEpochNoClaims { sealed_epoch } => {
                sealed_epoch.inputs.clone()
            }
            SealedEpochState::SealedEpochWithClaims { claimed_epoch } => {
                claimed_epoch.inputs.clone()
            }
        }
    }
}

/// Sealed epoch with one or more claims
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpochWithClaims {
    pub epoch_number: U256,
    pub claims: Claims,
    pub inputs: EpochInputState,
    pub dapp_contract_address: Address,
}

/// Set of claims
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    claims: HashMap<H256, HashSet<Address>>,
    first_claim_timestamp: U256,
}

impl Claims {
    pub fn new(claim: H256, sender: Address, timestamp: U256) -> Self {
        let claims = HashMap::unit(claim, HashSet::unit(sender));
        Self {
            claims,
            first_claim_timestamp: timestamp,
        }
    }

    pub fn first_claim_timestamp(&self) -> U256 {
        self.first_claim_timestamp
    }

    pub fn claims(self) -> HashMap<H256, HashSet<Address>> {
        self.claims.clone()
    }

    pub fn claims_ref(&self) -> &HashMap<H256, HashSet<Address>> {
        &self.claims
    }

    pub fn update_with_new_claim(&self, claim: H256, sender: Address) -> Self {
        let sender_set =
            self.claims.clone().entry(claim).or_default().update(sender);
        let claims = self.claims.update(claim, sender_set);
        Self {
            claims,
            first_claim_timestamp: self.first_claim_timestamp,
        }
    }

    pub fn insert_claim(&mut self, claim: H256, sender: Address) {
        self.claims.entry(claim).or_default().insert(sender);
    }

    pub fn get_sender_claim(&self, sender: &Address) -> Option<H256> {
        for (k, v) in self.claims.iter() {
            if v.contains(sender) {
                return Some(*k);
            }
        }
        None
    }

    pub fn get_senders_with_claim(&self, claim: &H256) -> HashSet<Address> {
        self.claims.get(claim).cloned().unwrap_or_default()
    }

    pub fn iter(&self) -> im::hashmap::Iter<H256, HashSet<Address>> {
        self.claims.iter()
    }
}

impl<'a> IntoIterator for &'a Claims {
    type Item = (&'a H256, &'a HashSet<Address>);
    type IntoIter = im::hashmap::Iter<'a, H256, HashSet<Address>>;

    fn into_iter(self) -> Self::IntoIter {
        self.claims.iter()
    }
}

impl IntoIterator for Claims {
    type Item = (H256, HashSet<Address>);
    type IntoIter = im::hashmap::ConsumingIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.claims.into_iter()
    }
}

#[async_trait]
impl Foldable for SealedEpochState {
    type InitialState = (Address, U256);
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        block: &Block,
        env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let (dapp_contract_address, epoch_number) = initial_state.clone();

        let contract =
            RollupsFacet::new(dapp_contract_address, Arc::clone(&access));

        // Inputs of epoch
        let inputs = EpochInputState::get_state_for_block(
            &(dapp_contract_address, epoch_number),
            block,
            env,
        )
        .await?
        .state;

        // Get all claim events of epoch
        let claim_events = contract
            .claim_filter()
            .topic1(epoch_number.clone())
            .query_with_meta()
            .await
            .context("Error querying for rollups claims")?;

        // If there are no claim, state is SealedEpochNoClaims
        if claim_events.is_empty() {
            return Ok(SealedEpochState::SealedEpochNoClaims {
                sealed_epoch: AccumulatingEpoch {
                    epoch_number,
                    inputs,
                    dapp_contract_address,
                },
            });
        }

        let mut claims: Option<Claims> = None;
        for (claim_event, meta) in claim_events {
            claims = Some(match claims {
                None => {
                    // If first claim, get timestamp
                    let timestamp = access
                        .get_block(meta.block_hash)
                        .await
                        .map_err(|e| FoldableError::from(Error::from(e)))?
                        .context("Block not found")?
                        .timestamp;

                    Claims::new(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                        timestamp,
                    )
                }

                Some(mut c) => {
                    c.insert_claim(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                    );
                    c
                }
            });
        }

        Ok(SealedEpochState::SealedEpochWithClaims {
            claimed_epoch: EpochWithClaims {
                epoch_number,
                inputs,
                claims: claims.unwrap(),
                dapp_contract_address,
            },
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        // Check if there was (possibly) some log emited on this block.
        // As finalized epochs' inputs will not change, we can return early
        // without querying the input StateFold.
        let dapp_contract_address =
            previous_state.dapp_contract_address().clone();
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &dapp_contract_address,
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &previous_state.epoch_number(),
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &ClaimFilter::signature(),
        )) {
            return Ok(previous_state.clone());
        }

        let epoch_number = previous_state.epoch_number().clone();
        let contract = RollupsFacet::new(dapp_contract_address, access);

        // Get claim events of epoch at this block hash
        let claim_events = contract
            .claim_filter()
            .topic1(epoch_number.clone())
            .query_with_meta()
            .await
            .context("Error querying for rollups claims")?;

        // if there are no new claims, return epoch's old claims (might be empty)
        if claim_events.is_empty() {
            return Ok(previous_state.clone());
        }

        let mut claims: Option<Claims> = previous_state.claims();

        for (claim_event, _) in claim_events {
            claims = Some(match claims {
                None => {
                    // If this is the first claim in epoch, get block timestamp
                    // and create new claim
                    let timestamp = block.timestamp;
                    Claims::new(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                        timestamp,
                    )
                }

                Some(mut c) => {
                    // else there are other claims, timestamp is uninmportant
                    c.insert_claim(
                        claim_event.epoch_hash.into(),
                        claim_event.claimer,
                    );
                    c
                }
            });
        }

        // don't need to re-update inputs because epoch is sealed
        Ok(SealedEpochState::SealedEpochWithClaims {
            claimed_epoch: EpochWithClaims {
                epoch_number,
                inputs: previous_state.inputs(),
                claims: claims.unwrap(),
                dapp_contract_address,
            },
        })
    }
}
