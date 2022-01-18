pub mod types;
pub use setup::{create_rollups_state_fold, RollupsStateFold};

mod epoch_delegate;
pub mod erc20_token_delegate;
pub mod fee_manager_delegate;
pub mod input_delegate;
pub mod output_delegate;
pub mod rollups_delegate;
pub mod validator_manager_delegate;

mod accumulating_epoch_delegate;
mod finalized_epoch_delegate;
mod sealed_epoch_delegate;

pub mod setup;

use accumulating_epoch_delegate::AccumulatingEpochFoldDelegate;
use epoch_delegate::EpochFoldDelegate;
use finalized_epoch_delegate::FinalizedEpochFoldDelegate;
use input_delegate::InputFoldDelegate;
use output_delegate::OutputFoldDelegate;
use rollups_delegate::RollupsFoldDelegate;
use sealed_epoch_delegate::SealedEpochFoldDelegate;
