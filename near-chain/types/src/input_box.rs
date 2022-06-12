use crate::utils::MergeAscending;
use crate::FoldableError;

use contracts::input_box::*;
use ethers::{
    abi::AbiDecode,
    contract::LogMeta,
    prelude::EthEvent,
    providers::Middleware,
    types::{Address, Bytes, TxHash, H256, U256, U64},
};
use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{
    ethers::{self, prelude::Transaction},
    Block,
};

use anyhow::{ensure, Context};
use im::Vector;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::Arc;

use async_trait::async_trait;

const DIRTECT_INPUT_LOG_NUMBER: u64 = 0;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub sender: Address,
    pub payload: Vec<u8>,
    pub value: U256,
    pub transaction_hash: TxHash,
    pub block_hash: H256,
    pub timestamp: U256,
    pub block_number: U64,
    pub transaction_index: U64,
    pub log_index: U256,
}

impl Ord for Input {
    fn cmp(&self, other: &Self) -> Ordering {
        self.block_number
            .cmp(&other.block_number)
            .then(self.transaction_index.cmp(&other.transaction_index))
            .then(self.log_index.cmp(&other.log_index))
    }
}

impl PartialOrd for Input {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputList {
    pub input_box_contract_address: Address,
    pub inputs: Vector<Input>,
}

#[async_trait]
impl Foldable for InputList {
    type InitialState = Address;
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        _block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let input_box_contract_address = *initial_state;

        Ok(Self {
            input_box_contract_address,
            inputs: fetch_inputs(access, input_box_contract_address, None)
                .await?,
        })
    }

    async fn fold<M: Middleware + 'static>(
        previous_state: &Self,
        block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<FoldMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let input_box_contract_address =
            previous_state.input_box_contract_address;

        if !(fold_utils::contains_address(
            &block.logs_bloom,
            &input_box_contract_address,
        ) && (fold_utils::contains_topic(
            &block.logs_bloom,
            &DirectInputAddedFilter::signature(),
        ) || fold_utils::contains_topic(
            &block.logs_bloom,
            &IndirectInputAddedFilter::signature(),
        ))) {
            return Ok(previous_state.clone());
        }

        let new_inputs = fetch_inputs(
            access,
            input_box_contract_address,
            Some(block.clone()),
        )
        .await?;

        let mut inputs = previous_state.inputs.clone();
        inputs.append(new_inputs);

        Ok(Self {
            input_box_contract_address,
            inputs,
        })
    }
}

async fn fetch_inputs<M: Middleware + 'static>(
    provider: Arc<M>,
    input_box_contract_address: Address,
    block_opt: Option<Block>,
) -> Result<Vector<Input>, FoldableError> {
    let contract =
        InputBox::new(input_box_contract_address, Arc::clone(&provider));

    // Retrieve `DirectInputAdded` events
    let direct_inputs_futures: Vec<_> = contract
        .direct_input_added_filter()
        .query_with_meta()
        .await
        .context("Sync error querying for direct input added events in sync")?
        .into_iter()
        .map(|(_, meta)| {
            Input::build_direct_input(
                Arc::clone(&provider),
                meta.transaction_hash,
                block_opt.clone(),
            )
        })
        .collect();

    // Retrieve `IndirectInputAdded` events
    let indirect_inputs_futures: Vec<_> = contract
        .indirect_input_added_filter()
        .query_with_meta()
        .await
        .context("Sync error querying for direct input added events in sync")?
        .into_iter()
        .map(|(e, meta)| {
            Input::build_indirect_input(Arc::clone(&provider), e, meta)
        })
        .collect();

    let (direct_inputs_results, indirect_inputs_results) = futures::join!(
        futures::future::join_all(direct_inputs_futures),
        futures::future::join_all(indirect_inputs_futures),
    );

    let direct_inputs: Result<Vec<Input>, _> =
        direct_inputs_results.into_iter().collect();

    let indirect_inputs: Result<Vec<Input>, _> =
        indirect_inputs_results.into_iter().collect();

    let inputs = MergeAscending::new(
        direct_inputs?.into_iter(),
        indirect_inputs?.into_iter(),
    )
    .collect();

    Ok(inputs)
}

impl Input {
    async fn build_direct_input<M: Middleware + 'static>(
        provider: Arc<M>,
        tx_hash: TxHash,
        block_opt: Option<Block>,
    ) -> Result<Self, FoldableError> {
        let tx: MinedDirectInputTx = provider
            .get_transaction(tx_hash)
            .await
            .context(format!("Could not query tx_hash `{:?}`", tx_hash))?
            .context(format!(
                "Transaction with hash `{:?}` not found",
                tx_hash
            ))?
            .try_into()?;

        let block = match block_opt {
            Some(b) => b,
            None => provider
                .get_block(tx.block_hash)
                .await
                .context(format!(
                    "Could not query block `{:?}`",
                    tx.block_hash
                ))?
                .context(format!(
                    "Block with hash `{:?}` not found",
                    tx.block_hash
                ))?
                .try_into()
                .context("Could not convert Block")?,
        };

        Input::try_new_direct_input(tx, &block)
    }

    async fn build_indirect_input<M: Middleware + 'static>(
        provider: Arc<M>,
        event: IndirectInputAddedFilter,
        meta: LogMeta,
    ) -> Result<Self, FoldableError> {
        let block = provider
            .get_block(meta.block_hash)
            .await
            .context(format!("Could not query block `{:?}`", meta.block_hash))?
            .context(format!(
                "Block with hash `{:?}` not found",
                meta.block_hash
            ))?
            .try_into()
            .context("Could not convert Block")?;

        meta_consistent_with_block(&meta, &block)?;

        Ok(Self {
            sender: event.sender,
            payload: event.input.to_vec(),
            value: event.value,
            transaction_hash: meta.transaction_hash,
            timestamp: block.timestamp,
            block_hash: meta.block_hash,
            block_number: meta.block_number,
            transaction_index: meta.transaction_index,
            log_index: meta.log_index,
        })
    }

    fn try_new_direct_input(
        tx: MinedDirectInputTx,
        block: &Block,
    ) -> Result<Self, FoldableError> {
        tx.consistent_with_block(block)?;

        let payload = AddDirectInputCall::decode(tx.input)
            .context(format!(
                "Transaction with hash `{:?}` was not a direct input",
                tx.hash
            ))?
            .input
            .to_vec();

        Ok(Self {
            sender: tx.from,
            payload,
            value: tx.value,
            transaction_hash: tx.hash,
            timestamp: block.timestamp,
            block_hash: block.hash,
            block_number: block.number,
            transaction_index: tx.transaction_index,
            log_index: U256::from(DIRTECT_INPUT_LOG_NUMBER),
        })
    }
}

struct MinedDirectInputTx {
    hash: H256,
    block_hash: H256,
    block_number: U64,
    transaction_index: U64,
    from: Address,
    to: Address,
    value: U256,
    input: Bytes,
}

impl TryFrom<Transaction> for MinedDirectInputTx {
    // TODO: maybe better error type?
    type Error = FoldableError;

    fn try_from(tx: Transaction) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: tx.hash,

            block_hash: tx
                .block_hash
                .context("`MinedDirectInputTx` conversion failed: tx `block_hash` is None; maybe transaction wasn't mined yet?")?,

            block_number: tx
                .block_number
                .context("`MinedDirectInputTx` conversion failed: tx `block_number` is None; maybe transaction wasn't mined yet?")?,

            transaction_index: tx.transaction_index.context("`MinedDirectInputTx` conversion failed: tx `transaction_index` is None; maybe transaction wasn't mined yet?")?,

            from: tx.from,

            to: tx.to.context("`MinedDirectInputTx` conversion failed: tx `to` is None; maybe transaction was contract creation and not `AddDirectInput`?")?,

            value: tx.value,

            input: tx.input,
        })
    }
}

impl MinedDirectInputTx {
    fn consistent_with_block(
        &self,
        block: &Block,
    ) -> Result<(), anyhow::Error> {
        ensure!(
            self.block_hash == block.hash,
            "Sanity check failed: tx and block `block_hash` do not match"
        );

        ensure!(
            self.block_number == block.number,
            "Sanity check failed: tx and block `block_number` do not match"
        );

        Ok(())
    }
}

fn meta_consistent_with_block(
    meta: &LogMeta,
    block: &Block,
) -> Result<(), anyhow::Error> {
    ensure!(
        meta.block_hash == block.hash,
        "Sanity check failed: meta and block `block_hash` do not match"
    );

    ensure!(
        meta.block_number == block.number,
        "Sanity check failed: meta and block `block_number` do not match"
    );

    Ok(())
}
