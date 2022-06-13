use crate::FoldableError;
use anyhow::Context;
use async_trait::async_trait;
use contracts::output_facet::*;
use ethers::{
    prelude::EthEvent,
    providers::Middleware,
    types::{Address, U256},
};
use im::HashMap;
use serde::{Deserialize, Serialize};
use state_fold::{
    utils as fold_utils, FoldMiddleware, Foldable, StateFoldEnvironment,
    SyncMiddleware,
};
use state_fold_types::{ethers, Block};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputState {
    pub dapp_contract_address: Address,
    pub vouchers: HashMap<usize, HashMap<usize, HashMap<usize, bool>>>,
}

/// voucher_position = voucher_index * 2 ** 128 + input_index * 2 ** 64 + epoch
/// We always assume indices have at most 8 bytes, as does rust
fn convert_voucher_position_to_indices(
    voucher_position: U256,
) -> (usize, usize, usize) {
    let mut pos_bytes = [0u8; 32];
    voucher_position.to_big_endian(&mut pos_bytes);

    let mut voucher_index_bytes = [0u8; 8];
    voucher_index_bytes.copy_from_slice(&pos_bytes[8..16]);

    let mut input_index_bytes = [0u8; 8];
    input_index_bytes.copy_from_slice(&pos_bytes[16..24]);

    let mut epoch_bytes = [0u8; 8];
    epoch_bytes.copy_from_slice(&pos_bytes[24..32]);

    (
        usize::from_be_bytes(voucher_index_bytes),
        usize::from_be_bytes(input_index_bytes),
        usize::from_be_bytes(epoch_bytes),
    )
}

#[async_trait]
impl Foldable for OutputState {
    type InitialState = Address;
    type Error = FoldableError;
    type UserData = ();

    async fn sync<M: Middleware + 'static>(
        initial_state: &Self::InitialState,
        _block: &Block,
        _env: &StateFoldEnvironment<M, Self::UserData>,
        access: Arc<SyncMiddleware<M>>,
    ) -> Result<Self, Self::Error> {
        let dapp_contract_address = *initial_state;

        let contract = OutputFacet::new(dapp_contract_address, access);
        let events = contract
            .voucher_executed_filter()
            .query()
            .await
            .context("Error querying for voucher executed events")?;

        let mut vouchers: HashMap<usize, HashMap<usize, HashMap<usize, bool>>> =
            HashMap::new();

        for ev in events {
            let (voucher_index, input_index, epoch_index) =
                convert_voucher_position_to_indices(ev.voucher_position);
            vouchers
                .entry(voucher_index)
                .or_insert_with(|| HashMap::new())
                .entry(input_index)
                .or_insert_with(|| HashMap::new())
                .entry(epoch_index)
                .or_insert_with(|| true);
        }

        Ok(OutputState {
            vouchers,
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
            &VoucherExecutedFilter::signature(),
        )) {
            return Ok(previous_state.clone());
        }

        let contract = OutputFacet::new(dapp_contract_address, access);
        let events = contract
            .voucher_executed_filter()
            .query()
            .await
            .context("Error querying for voucher executed events")?;

        let mut vouchers = previous_state.vouchers.clone();

        for ev in events {
            let (voucher_index, input_index, epoch_index) =
                convert_voucher_position_to_indices(ev.voucher_position);
            vouchers
                .entry(voucher_index)
                .or_insert_with(|| HashMap::new())
                .entry(input_index)
                .or_insert_with(|| HashMap::new())
                .entry(epoch_index)
                .or_insert_with(|| true);
        }

        Ok(OutputState {
            vouchers,
            dapp_contract_address,
        })
    }
}
