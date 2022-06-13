use crate::FoldableError;
use anyhow::Context;
use async_trait::async_trait;
use contracts::input_facet::*;
use ethers::{
    abi::{encode, Token},
    contract::LogMeta,
    prelude::EthEvent,
    providers::Middleware,
    types::{Address, U256, U64},
};
use im::Vector;
use serde::{Deserialize, Serialize};
use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::sync::Arc;

/// Set of inputs at some epoch
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpochInputState {
    pub epoch_number: U256,
    pub inputs: Vector<Input>,
    pub dapp_contract_address: Address,
}

impl EpochInputState {
    pub fn new(epoch_number: U256, dapp_contract_address: Address) -> Self {
        Self {
            epoch_number,
            inputs: Vector::new(),
            dapp_contract_address,
        }
    }
}

/// Single input from Input.sol contract
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    pub sender: Address, // TODO: Get from calldata.
    pub block_number: U64,
    pub timestamp: U256,       // TODO: Get from calldata.
    pub payload: Arc<Vec<u8>>, // TODO: Get from calldata.
}

impl Input {
    /// Onchain metadata is abi.encode(msg.sender, block.timestamp)
    pub fn get_metadata(&self) -> Vec<u8> {
        let bytes = encode(&[
            Token::Address(self.sender),
            Token::Uint(self.timestamp.into()),
        ]);

        // This encoding must have 64 bytes:
        // 20 bytes plus 12 zero padding for address,
        // and 32 for timestamp.
        // This is only the case because we're using `encode`
        // and not `encodePacked`.
        assert_eq!(bytes.len(), 64);
        bytes
    }
}

#[async_trait]
impl Foldable for EpochInputState {
    type InitialState = (Address, U256);
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        _block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let (dapp_contract_address, epoch_number) = initial_state.clone();

        let contract = InputFacet::new(dapp_contract_address, access);

        // Retrieve `InputAdded` events
        let events = contract
            .input_added_filter()
            .topic1(epoch_number)
            .query_with_meta()
            .await
            .context("Error querying for input added events")?;

        let mut inputs: Vector<Input> = Vector::new();
        for ev in events {
            inputs.push_back(ev.into());
        }

        Ok(EpochInputState {
            epoch_number,
            inputs,
            dapp_contract_address,
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let dapp_contract_address = previous_state.dapp_contract_address;
        // If not in bloom copy previous state
        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &dapp_contract_address,
        ) && fold_utils::contains_topic(
            &block.logs_bloom,
            &InputAddedFilter::signature(),
        )) {
            return Ok(previous_state.clone());
        }

        let contract = InputFacet::new(dapp_contract_address, access);

        let events = contract
            .input_added_filter()
            .topic1(previous_state.epoch_number)
            .query()
            .await
            .context("Error querying for input added events")?;

        let mut inputs = previous_state.inputs.clone();
        for ev in events {
            inputs.push_back((ev, block).into());
        }

        Ok(EpochInputState {
            epoch_number: previous_state.epoch_number,
            inputs,
            dapp_contract_address,
        })
    }
}

impl From<(InputAddedFilter, LogMeta)> for Input {
    fn from(log: (InputAddedFilter, LogMeta)) -> Self {
        let ev = log.0;
        Self {
            sender: ev.sender,
            payload: Arc::new(ev.input.to_vec()),
            timestamp: ev.timestamp,

            block_number: log.1.block_number,
        }
    }
}

impl From<(InputAddedFilter, &Block)> for Input {
    fn from(log: (InputAddedFilter, &Block)) -> Self {
        let ev = log.0;
        Self {
            sender: ev.sender,
            payload: Arc::new(ev.input.to_vec()),
            timestamp: ev.timestamp,

            block_number: log.1.number,
        }
    }
}
