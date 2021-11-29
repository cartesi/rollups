use offchain_core::ethers;

use ethers::abi::{encode, Token};
use ethers::types::{Address, H256, U256, U64};
use im::{HashMap, HashSet, Vector};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

/// Set of inputs at some epoch
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpochInputState {
    pub epoch_number: U256,
    pub inputs: Vector<Input>,
    pub input_contract_address: Address,
}

impl EpochInputState {
    pub fn new(epoch_number: U256, input_contract_address: Address) -> Self {
        Self {
            epoch_number,
            inputs: Vector::new(),
            input_contract_address,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputState {
    pub output_address: Address,
    pub vouchers: HashMap<usize, HashMap<usize, HashMap<usize, bool>>>,
}

/// Set of claims
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    claims: HashMap<H256, HashSet<Address>>,
    first_claim_timestamp: U256,
}

impl Claims {
    pub fn new(claim: H256, sender: Address, timestamp: U256) -> Self {
        let claims = HashMap::unit(claim, HashSet::unit(sender));
        Self {
            claims,
            first_claim_timestamp: timestamp,
        }
    }

    pub fn first_claim_timestamp(&self) -> U256 {
        self.first_claim_timestamp
    }

    pub fn claims(self) -> HashMap<H256, HashSet<Address>> {
        self.claims.clone()
    }

    pub fn claims_ref(&self) -> &HashMap<H256, HashSet<Address>> {
        &self.claims
    }

    pub fn update_with_new_claim(&self, claim: H256, sender: Address) -> Self {
        let sender_set =
            self.claims.clone().entry(claim).or_default().update(sender);
        let claims = self.claims.update(claim, sender_set);
        Self {
            claims,
            first_claim_timestamp: self.first_claim_timestamp,
        }
    }

    pub fn insert_claim(&mut self, claim: H256, sender: Address) {
        self.claims.entry(claim).or_default().insert(sender);
    }

    pub fn get_sender_claim(&self, sender: &Address) -> Option<H256> {
        for (k, v) in self.claims.iter() {
            if v.contains(sender) {
                return Some(*k);
            }
        }
        None
    }

    pub fn get_senders_with_claim(&self, claim: &H256) -> HashSet<Address> {
        self.claims.get(claim).cloned().unwrap_or_default()
    }

    pub fn iter(&self) -> im::hashmap::Iter<H256, HashSet<Address>> {
        self.claims.iter()
    }
}

impl<'a> IntoIterator for &'a Claims {
    type Item = (&'a H256, &'a HashSet<Address>);
    type IntoIter = im::hashmap::Iter<'a, H256, HashSet<Address>>;

    fn into_iter(self) -> Self::IntoIter {
        self.claims.iter()
    }
}

impl IntoIterator for Claims {
    type Item = (H256, HashSet<Address>);
    type IntoIter = im::hashmap::ConsumingIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.claims.into_iter()
    }
}

/// Epoch finalized on the blockchain, vouchers are executable and notices
/// are verfiable/provable
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinalizedEpoch {
    pub epoch_number: U256,
    pub hash: H256,
    pub inputs: EpochInputState,

    /// Hash of block in which epoch was finalized
    pub finalized_block_hash: H256,

    /// Number of block in which epoch was finalized
    pub finalized_block_number: U64,
}

/// Set of finalized epochs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FinalizedEpochs {
    /// Set of `FinalizedEpoch`
    pub finalized_epochs: Vector<FinalizedEpoch>,

    /// The first epoch that will be included in `finalized_epochs`
    pub initial_epoch: U256,

    pub rollups_contract_address: Address,
    pub input_contract_address: Address,
}

impl FinalizedEpochs {
    pub fn new(
        initial_epoch: U256,
        rollups_contract_address: Address,
        input_contract_address: Address,
    ) -> Self {
        Self {
            finalized_epochs: Vector::new(),
            initial_epoch,
            rollups_contract_address,
            input_contract_address,
        }
    }

    pub fn get_epoch(&self, index: usize) -> Option<FinalizedEpoch> {
        if index >= self.initial_epoch.as_usize()
            && index < self.next_epoch().as_usize()
        {
            let actual_index = index - self.initial_epoch.as_usize();
            Some(self.finalized_epochs[actual_index].clone())
        } else {
            None
        }
    }

    pub fn next_epoch(&self) -> U256 {
        self.initial_epoch + self.finalized_epochs.len()
    }

    fn epoch_number_consistent(&self, epoch_number: &U256) -> bool {
        *epoch_number == self.next_epoch()
    }

    /// If `finalized_epoch.epoch_number` is not consistent, this method fails
    /// to insert epoch and returns false.
    pub fn insert_epoch(&mut self, finalized_epoch: FinalizedEpoch) -> bool {
        if !self.epoch_number_consistent(&finalized_epoch.epoch_number) {
            return false;
        }

        self.finalized_epochs.push_back(finalized_epoch);
        true
    }
}

/// Sealed epoch with one or more claims
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpochWithClaims {
    pub epoch_number: U256,
    pub claims: Claims,
    pub inputs: EpochInputState,
    pub rollups_contract_address: Address,
    pub input_contract_address: Address,
}

/// Active epoch currently receiveing inputs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccumulatingEpoch {
    pub epoch_number: U256,
    pub inputs: EpochInputState,
    pub rollups_contract_address: Address,
    pub input_contract_address: Address,
}

impl AccumulatingEpoch {
    pub fn new(
        rollups_contract_address: Address,
        input_contract_address: Address,
        epoch_number: U256,
    ) -> Self {
        Self {
            epoch_number,
            inputs: EpochInputState::new(epoch_number, input_contract_address),
            rollups_contract_address,
            input_contract_address,
        }
    }
}

///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PhaseState {
    /// No claims or disputes going on, the previous epoch was finalized
    /// successfully and the current epoch is still accumulating inputs
    InputAccumulation {},

    /// `current_epoch` is no longer accepting inputs but hasn't yet received
    /// a claim
    EpochSealedAwaitingFirstClaim { sealed_epoch: AccumulatingEpoch },

    /// Epoch has been claimed but a dispute has yet to arise
    AwaitingConsensusNoConflict { claimed_epoch: EpochWithClaims },

    /// Epoch being claimed was previously challenged and there is a standing
    /// claim that can be challenged
    AwaitingConsensusAfterConflict {
        claimed_epoch: EpochWithClaims,
        challenge_period_base_ts: U256,
    },
    /// Consensus was not reached but the last 'challenge_period' is over. Epoch
    /// can be finalized at any time by anyone
    ConsensusTimeout { claimed_epoch: EpochWithClaims },

    /// Unreacheable
    AwaitingDispute { claimed_epoch: EpochWithClaims },
    // TODO: add dispute timeout when disputes are turned on.
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImmutableState {
    /// duration of input accumulation phase in seconds
    pub input_duration: U256,

    /// duration of challenge period in seconds
    pub challenge_period: U256,

    /// timestamp of the contract creation
    pub contract_creation_timestamp: U256,

    /// contract responsible for inputs
    pub input_contract_address: Address,

    /// contract responsible for outputs
    pub output_contract_address: Address,

    /// contract responsible for validators
    pub validator_contract_address: Address,

    /// contract responsible for dispute resolution
    pub dispute_contract_address: Address,

    /// rollups contract address
    pub rollups_contract_address: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollupsState {
    pub constants: ImmutableState,

    pub initial_epoch: U256,
    pub finalized_epochs: FinalizedEpochs,
    pub current_epoch: AccumulatingEpoch,

    pub current_phase: PhaseState,

    pub output_state: OutputState,
}
