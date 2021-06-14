mod contracts;

pub mod descartesv2_delegate;
pub mod epoch_delegate;
pub mod input_delegate;

pub mod accumulating_epoch_delegate;
pub mod finalized_epoch_delegate;
pub mod sealed_epoch_delegate;

pub mod setup;
pub mod types;

pub use accumulating_epoch_delegate::AccumulatingEpochFoldDelegate;
pub use descartesv2_delegate::DescartesV2FoldDelegate;
pub use epoch_delegate::EpochFoldDelegate;
pub use finalized_epoch_delegate::FinalizedEpochFoldDelegate;
pub use input_delegate::InputFoldDelegate;
pub use sealed_epoch_delegate::SealedEpochFoldDelegate;
