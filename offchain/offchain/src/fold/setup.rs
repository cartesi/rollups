use super::*;
use state_fold::{config::SFConfig, DelegateAccess, StateFold};

use crate::fold::erc20_token_delegate::ERC20BalanceFoldDelegate;
use crate::fold::fee_manager_delegate::FeeManagerFoldDelegate;
use crate::fold::validator_manager_delegate::ValidatorManagerFoldDelegate;
use std::sync::Arc;

pub type RollupsStateFold<DA> = Arc<StateFold<RollupsFoldDelegate<DA>, DA>>;

/// Creates Rollups State Fold
pub fn create_rollups_state_fold<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> RollupsStateFold<DA> {
    let epoch_fold = create_epoch(Arc::clone(&access), config);
    let output_fold = create_output(Arc::clone(&access), config);
    let validator_manager_fold =
        create_validator_manager(Arc::clone(&access), config);
    let fee_manager_fold = create_fee_manager(Arc::clone(&access), config);

    let delegate = RollupsFoldDelegate::new(
        epoch_fold,
        output_fold,
        validator_manager_fold,
        fee_manager_fold,
    );
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

pub type OutputStateFold<DA> = Arc<StateFold<OutputFoldDelegate, DA>>;
pub fn create_output<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> OutputStateFold<DA> {
    let delegate = OutputFoldDelegate::default();
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

pub type ValidatorManagerStateFold<DA> =
    Arc<StateFold<ValidatorManagerFoldDelegate, DA>>;
pub fn create_validator_manager<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> ValidatorManagerStateFold<DA> {
    let delegate = ValidatorManagerFoldDelegate::default();
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

pub type FeeManagerStateFold<DA> =
    Arc<StateFold<FeeManagerFoldDelegate<DA>, DA>>;
pub fn create_fee_manager<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> FeeManagerStateFold<DA> {
    let erc20_balance_fold = create_erc20_balance(Arc::clone(&access), config);
    let validator_manager_fold =
        create_validator_manager(Arc::clone(&access), config);
    let delegate =
        FeeManagerFoldDelegate::new(erc20_balance_fold, validator_manager_fold);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

pub type ERC20BalanceStateFold<DA> =
    Arc<StateFold<ERC20BalanceFoldDelegate, DA>>;
pub fn create_erc20_balance<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> ERC20BalanceStateFold<DA> {
    let delegate = ERC20BalanceFoldDelegate::default();
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type AccumulatingEpochStateFold<DA> =
    Arc<StateFold<AccumulatingEpochFoldDelegate<DA>, DA>>;
fn create_accumulating_epoch<DA: DelegateAccess + Send + Sync + 'static>(
    input_fold: InputStateFold<DA>,
    access: Arc<DA>,
    config: &SFConfig,
) -> AccumulatingEpochStateFold<DA> {
    let delegate = AccumulatingEpochFoldDelegate::new(input_fold);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type SealedEpochStateFold<DA> = Arc<StateFold<SealedEpochFoldDelegate<DA>, DA>>;
fn create_sealed_epoch<DA: DelegateAccess + Send + Sync + 'static>(
    input_fold: InputStateFold<DA>,
    access: Arc<DA>,
    config: &SFConfig,
) -> SealedEpochStateFold<DA> {
    let delegate = SealedEpochFoldDelegate::new(input_fold);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type FinalizedEpochStateFold<DA> =
    Arc<StateFold<FinalizedEpochFoldDelegate<DA>, DA>>;
fn create_finalized_epoch<DA: DelegateAccess + Send + Sync + 'static>(
    input_fold: InputStateFold<DA>,
    access: Arc<DA>,
    config: &SFConfig,
) -> FinalizedEpochStateFold<DA> {
    let delegate = FinalizedEpochFoldDelegate::new(input_fold);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}

type EpochStateFold<DA> = Arc<StateFold<EpochFoldDelegate<DA>, DA>>;
fn create_epoch<DA: DelegateAccess + Send + Sync + 'static>(
    access: Arc<DA>,
    config: &SFConfig,
) -> EpochStateFold<DA> {
    let input_fold = create_input(Arc::clone(&access), config);
    let accumulating_fold = create_accumulating_epoch(
        Arc::clone(&input_fold),
        Arc::clone(&access),
        config,
    );
    let sealed_fold = create_sealed_epoch(
        Arc::clone(&input_fold),
        Arc::clone(&access),
        config,
    );
    let finalized_fold = create_finalized_epoch(
        Arc::clone(&input_fold),
        Arc::clone(&access),
        config,
    );

    let delegate =
        EpochFoldDelegate::new(accumulating_fold, sealed_fold, finalized_fold);
    let state_fold = StateFold::new(delegate, access, config.safety_margin);
    Arc::new(state_fold)
}
