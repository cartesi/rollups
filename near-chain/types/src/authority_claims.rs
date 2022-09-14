use crate::FoldableError;

use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{
    ethers::{
        prelude::EthEvent,
        providers::Middleware,
        types::{Address, H256},
    },
    Block,
};

// use contracts::history::*;

use anyhow::Context;
use async_trait::async_trait;
use im::{HashMap, Vector};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// TODO change name
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct ClaimsInitialState {
    history_address: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claim {
    pub claim: H256,

    // Both "closed/inclusive"
    pub start_input_index: usize,
    pub end_input_index: usize,
}

impl From<([u8; 32], u128, u128)> for Claim {
    fn from(x: ([u8; 32], u128, u128)) -> Self {
        Self {
            claim: x.0.into(),
            start_input_index: x.1 as usize,
            end_input_index: x.2 as usize,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DAppHistory {
    pub claims: Vector<Arc<Claim>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct History {
    pub claims_initial_state: Arc<ClaimsInitialState>,
    pub histories: Arc<HashMap<Arc<Address>, Arc<DAppHistory>>>,
}

#[async_trait]
impl Foldable for History {
    type InitialState = Arc<ClaimsInitialState>;
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        _block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let claims_initial_state = initial_state.clone();
        let contract_address = claims_initial_state.history_address;

        let mut histories = HashMap::new();
        fetch_history(access, contract_address, &mut histories).await?;

        Ok(Self {
            claims_initial_state,
            histories: Arc::new(histories),
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block, // TODO: when new version of state-fold gets released, change this to Arc
        // and save on cloning.
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let history_address =
            previous_state.claims_initial_state.history_address;

        if !(fold_utils::contains_address(&block.logs_bloom, &history_address)
            && (fold_utils::contains_topic(
                &block.logs_bloom,
                &contracts::history::NewClaimFilter::signature(),
            )))
        {
            return Ok(previous_state.clone());
        }

        let mut new_histories = (*previous_state.histories).clone();
        fetch_history(access, history_address, &mut new_histories).await?;

        Ok(Self {
            claims_initial_state: previous_state.claims_initial_state.clone(),
            histories: Arc::new(new_histories),
        })
    }
}

async fn fetch_history<M: Middleware + 'static>(
    provider: Arc<M>,
    contract_address: Address,
    histories: &mut HashMap<Arc<Address>, Arc<DAppHistory>>,
) -> Result<(), FoldableError> {
    use contracts::history::*;

    let contract = History::new(contract_address, Arc::clone(&provider));

    // Retrieve `NewClaim` events
    let claims = contract
        .new_claim_filter()
        .query()
        .await
        .context("Error querying for new claim events")?;

    for claim in claims {
        let new_claim: Arc<Claim> = Arc::new(claim.claim.into());
        let dapp_address = Arc::new(claim.dapp);

        histories
            .entry(dapp_address)
            .and_modify(|h| {
                let mut new_history = (**h).clone();
                new_history.claims.push_back(new_claim.clone());
                *h = Arc::new(new_history);
            })
            .or_insert_with(|| {
                Arc::new(DAppHistory {
                    claims: im::vector![new_claim],
                })
            });
    }

    Ok(())
}
