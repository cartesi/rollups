use super::*;
use state_fold::{config::SFConfig, DelegateAccess, StateFold};

use std::sync::Arc;
use crate::fold::fee_manager_delegate::FeeManagerFoldDelegate;

pub type RollupsStateFold<DA> =
    Arc<StateFold<RollupsFoldDelegate<DA>, DA>>;

/// Creates Rollups State Fold
pub fn create_rollups_state_fold<
    DA: DelegateAccess + Send + Sync + 'static,
>(
    access: Arc<DA>,
    config: &SFConfig,
) -> RollupsStateFold<DA> {
    let epoch_fold = create_epoch(Arc::clone(&access), config);
    let output_fold = create_output(Arc::clone(&access), config);
    let fee_manager_fold = create_fee_manager(Arc::clone(&access), config);

    let delegate = RollupsFoldDelegate::new(epoch_fold, output_fold, fee_manager_fold);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

pub type InputStateFold<DA> = Arc<StateFold<InputFoldDelegate, DA>>;
pub fn create_input<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> InputStateFold<DA> {
    let delegate = InputFoldDelegate::default();
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type InputContractAddressStateFold<DA> =
    Arc<StateFold<InputContractAddressFoldDelegate, DA>>;
fn create_input_contract_address<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> InputContractAddressStateFold<DA> {
    let delegate = InputContractAddressFoldDelegate::default();
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

pub type OutputStateFold<DA> = Arc<StateFold<OutputFoldDelegate, DA>>;
pub fn create_output<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> OutputStateFold<DA> {
    let delegate = OutputFoldDelegate::default();
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

pub type FeeManagerStateFold<DA> = Arc<StateFold<FeeManagerFoldDelegate, DA>>;
pub fn create_fee_manager<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> FeeManagerStateFold<DA> {
    let delegate = FeeManagerFoldDelegate::default();
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type AccumulatingEpochStateFold<DA> =
    Arc<StateFold<AccumulatingEpochFoldDelegate<DA>, DA>>;
fn create_accumulating_epoch<DA: DelegateAccess + Send + Sync + 'static>(
    input_fold: InputStateFold<DA>,
    access: Arc<DA>,
    input_contract_address: InputContractAddressStateFold<DA>,
    config: &SFConfig,
) -> AccumulatingEpochStateFold<DA> {
    let delegate =
        AccumulatingEpochFoldDelegate::new(input_fold, input_contract_address);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type SealedEpochStateFold<DA> = Arc<StateFold<SealedEpochFoldDelegate<DA>, DA>>;
fn create_sealed_epoch<DA: DelegateAccess + Send + Sync + 'static>(
    input_fold: InputStateFold<DA>,
    access: Arc<DA>,
    input_contract_address: InputContractAddressStateFold<DA>,
    config: &SFConfig,
) -> SealedEpochStateFold<DA> {
    let delegate =
        SealedEpochFoldDelegate::new(input_fold, input_contract_address);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type FinalizedEpochStateFold<DA> =
    Arc<StateFold<FinalizedEpochFoldDelegate<DA>, DA>>;
fn create_finalized_epoch<DA: DelegateAccess + Send + Sync + 'static>(
    input_fold: InputStateFold<DA>,
    access: Arc<DA>,
    input_contract_address: InputContractAddressStateFold<DA>,
    config: &SFConfig,
) -> FinalizedEpochStateFold<DA> {
    let delegate =
        FinalizedEpochFoldDelegate::new(input_fold, input_contract_address);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type EpochStateFold<DA> = Arc<StateFold<EpochFoldDelegate<DA>, DA>>;
fn create_epoch<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> EpochStateFold<DA> {
    let input_fold = create_input(Arc::clone(&access), config);
    let input_contract_address =
        create_input_contract_address(Arc::clone(&access), config);
    let accumulating_fold = create_accumulating_epoch(
        Arc::clone(&input_fold),
        Arc::clone(&access),
        Arc::clone(&input_contract_address),
        config,
    );
    let sealed_fold = create_sealed_epoch(
        Arc::clone(&input_fold),
        Arc::clone(&access),
        Arc::clone(&input_contract_address),
        config,
    );
    let finalized_fold = create_finalized_epoch(
        Arc::clone(&input_fold),
        Arc::clone(&access),
        Arc::clone(&input_contract_address),
        config,
    );

    let delegate =
        EpochFoldDelegate::new(accumulating_fold, sealed_fold, finalized_fold);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}
