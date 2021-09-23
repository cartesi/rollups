use super::*;
use state_fold::{DelegateAccess, StateFold};

use std::sync::Arc;

pub struct SetupConfig {
    pub safety_margin: usize,
}

pub type DescartesStateFold<DA> =
    Arc<StateFold<DescartesV2FoldDelegate<DA>, DA>>;

/// Creates DescartesV2 State Fold
pub fn create_descartes_state_fold<
    DA: DelegateAccess + Send + Sync + 'static,
>(
    access: Arc<DA>,
    config: &SetupConfig,
) -> DescartesStateFold<DA> {
    let epoch_fold = create_epoch(Arc::clone(&access), config);
    let output_fold = create_output(Arc::clone(&access), config);

    let delegate = DescartesV2FoldDelegate::new(
        epoch_fold,
        output_fold,
    );
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type InputStateFold<DA> = Arc<StateFold<InputFoldDelegate, DA>>;
fn create_input<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SetupConfig,
) -> InputStateFold<DA> {
    let delegate = InputFoldDelegate::new();
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type InputContractAddressStateFold<DA> =
    Arc<StateFold<InputContractAddressFoldDelegate, DA>>;
fn create_input_contract_address<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SetupConfig,
) -> InputContractAddressStateFold<DA> {
    let delegate = InputContractAddressFoldDelegate::new();
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

pub type OutputStateFold<DA> = Arc<StateFold<OutputFoldDelegate, DA>>;
pub fn create_output<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SetupConfig,
) -> OutputStateFold<DA> {
    let delegate = OutputFoldDelegate::new();
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type AccumulatingEpochStateFold<DA> =
    Arc<StateFold<AccumulatingEpochFoldDelegate<DA>, DA>>;
fn create_accumulating_epoch<DA: DelegateAccess + Send + Sync + 'static>(
    input_fold: InputStateFold<DA>,
    access: Arc<DA>,
    input_contract_address: InputContractAddressStateFold<DA>,
    config: &SetupConfig,
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
    config: &SetupConfig,
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
    config: &SetupConfig,
) -> FinalizedEpochStateFold<DA> {
    let delegate =
        FinalizedEpochFoldDelegate::new(input_fold, input_contract_address);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type EpochStateFold<DA> = Arc<StateFold<EpochFoldDelegate<DA>, DA>>;
fn create_epoch<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SetupConfig,
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
