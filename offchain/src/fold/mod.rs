pub mod types;
pub use setup::{create_descartes_state_fold, DescartesStateFold, SetupConfig};

mod contracts;

mod descartesv2_delegate;
mod epoch_delegate;
mod input_delegate;

mod accumulating_epoch_delegate;
mod finalized_epoch_delegate;
mod sealed_epoch_delegate;

mod setup;

use accumulating_epoch_delegate::AccumulatingEpochFoldDelegate;
use descartesv2_delegate::DescartesV2FoldDelegate;
use epoch_delegate::EpochFoldDelegate;
use finalized_epoch_delegate::FinalizedEpochFoldDelegate;
use input_delegate::InputFoldDelegate;
use sealed_epoch_delegate::SealedEpochFoldDelegate;
