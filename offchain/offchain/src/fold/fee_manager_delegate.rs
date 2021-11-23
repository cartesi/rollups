use crate::contracts::fee_manager_contract::*;

use super::types::FeeManagerState;

use offchain_core::types::Block;
use state_fold::{
    delegate_access::{FoldAccess, SyncAccess},
    error::*,
    types::*,
    utils as fold_utils,
};

use async_trait::async_trait;
use snafu::ResultExt;

use ethers::prelude::EthEvent;
use ethers::types::{Address, U256};
use ethers_core::types::I256;

use im::HashMap;

/// Fee Manager Delegate
#[derive(Default)]
pub struct FeeManagerDelegate {}

#[async_trait]
impl StateFoldDelegate for FeeManagerDelegate {
    type InitialState = Address;
    type Accumulator = FeeManagerState;
    type State = BlockState<Self::Accumulator>;

    async fn sync<A: SyncAccess + Send + Sync>(
        &self,
        fee_manager_address: &Address,
        block: &Block,
        access: &A,
    ) -> SyncResult<Self::Accumulator, A> {
        let contract: FeeManagerImpl<A> = access
            .build_sync_contract(*fee_manager_address, block.number, FeeManagerImpl::new)
            .await;

        // `fee_manager_created` event
        let events = contract.fee_manager_created_filter().query().await.context(
            SyncContractError {
                err: "Error querying for fee manager created events",
            },
        )?;
        let created_event = events.first().unwrap();

        // `fee_redeemed` events
        let events = contract.fee_redeemed_filter().query().await.context(
            SyncContractError {
                err: "Error querying for fee redeemed events",
            },
        )?;

        let mut validator_redeemed: [Option<(Address, U256)>; 8] = [None; 8];
        let mut validator_redeemed_sums: HashMap<Address, U256> = HashMap::new();

        for (index, ev) in events.iter().enumerate() {
            match validator_redeemed_sums.get(&ev.validator) {
                Some(amount) => validator_redeemed_sums[ev.validator] = amount + ev.amount,
                None => validator_redeemed_sums[ev.validator] = ev.amount,
            }
        }

        for (index, sum) in validator_redeemed_sums.iter().enumerate() {
            validator_redeemed[index] = Some((*sum.0, *sum.1));
        }

        // filter event `Transfer(sender, recipient, amount) at erc20 address
        // https://github.com/OpenZeppelin/openzeppelin-contracts/blob/86bd4d73896afcb35a205456e361436701823c7a/contracts/token/ERC20/ERC20.sol#L238
        // filter recipient as `fee_manager`
        // balance of fee manager = total income - total fees redeemed by validators


        // leftover_balance seems to be overlapped with validator manager delegate

        Ok(FeeManagerState {
            validator_manager_address: created_event.validator_manager_cci,
            erc20_address: created_event.erc20,
            fee_per_claim: created_event.fee_per_claim,
            validator_redeemed,
            fee_manager_balance: U256::zero(),
            leftover_balance: I256::zero(),
        })
    }

    async fn fold<A: FoldAccess + Send + Sync>(
        &self,
        previous_state: &Self::Accumulator,
        block: &Block,
        access: &A,
    ) -> FoldResult<Self::Accumulator, A> {
        let voucher_address = previous_state.voucher_address;

        // If not in bloom copy previous state
        if !(fold_utils::contains_address(&block.logs_bloom, &voucher_address)
            && fold_utils::contains_topic(
                &block.logs_bloom,
                &VoucherExecutedFilter::signature(),
            ))
        {
            return Ok(previous_state.clone());
        }

        let contract = access
            .build_fold_contract(voucher_address, block.hash, VoucherImpl::new)
            .await;

        let events = contract.voucher_executed_filter().query().await.context(
            FoldContractError {
                err: "Error querying for voucher executed events",
            },
        )?;

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

        Ok(VoucherState {
            vouchers,
            voucher_address: voucher_address,
        })
    }

    fn convert(
        &self,
        accumulator: &BlockState<Self::Accumulator>,
    ) -> Self::State {
        accumulator.clone()
    }
}
