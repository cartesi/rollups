use ethers::types::{Address, H256, U256, U64};
use im::{HashMap, HashSet, Vector};
use std::sync::Arc;

/// Single input from Input.sol contract
#[derive(Clone, Debug)]
pub struct Input {
    pub sender: Address,       // TODO: Get from calldata.
    pub timestamp: U256,       // TODO: Get from calldata.
    pub payload: Arc<Vec<u8>>, // TODO: Get from calldata.
}

/// Set of inputs at some epoch
#[derive(Clone, Debug)]
pub struct InputState {
    pub epoch_number: U256,
    pub inputs: Vector<Input>,
}

/// Set of claims
#[derive(Clone, Debug)]
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

    pub fn update_claims(&self, claim: H256, sender: Address) -> Self {
        let sender_set = self.claims.entry(claim).or_default().update(sender);
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
        for (k, v) in self.claims {
            if v.contains(sender) {
                return Some(k);
            }
        }
        None
    }

    pub fn get_senders_with_claim(&self, claim: &H256) -> HashSet<Address> {
        self.claims.get(claim).cloned().unwrap_or_default()
    }

    pub fn has_sender_claimed(&self, claim: &H256, sender: &Address) -> bool {
        match self.claims.get(claim) {
            Some(m) => m.contains(sender),
            None => false,
        }
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

///
#[derive(Clone, Debug)]
pub struct FinalizedEpoch {
    pub hash: H256,
    pub epoch_number: U256,
    pub inputs: InputState,

    /// Hash of block in which epoch was finalized
    pub finalized_block_hash: H256,

    /// Number of block in which epoch was finalized
    pub finalized_block_number: U64,
}

///
#[derive(Clone, Debug)]
pub struct FinalizedEpochs {
    /// Set of `FinalizedEpoch`
    pub finalized_epochs: Vector<FinalizedEpoch>,

    /// The first epoch that will be included in `finalized_epochs`
    pub initial_epoch: U256,
}

impl FinalizedEpochs {
    pub fn new(initial_epoch: U256) -> Self {
        Self {
            finalized_epochs: Vector::new(),
            initial_epoch,
        }
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

    pub fn next_epoch(&self) -> U256 {
        self.initial_epoch + self.finalized_epochs.len()
    }

    fn epoch_number_consistent(&self, epoch_number: &U256) -> bool {
        *epoch_number == self.next_epoch()
    }
}

///
#[derive(Clone, Debug)]
pub struct SealedEpoch {
    pub epoch_number: U256,
    pub claims: Option<Claims>,
    pub inputs: InputState,
}

///
#[derive(Clone, Debug)]
pub struct AccumulatingEpoch {
    pub epoch_number: U256,
    pub inputs: InputState,
}

///
#[derive(Clone, Debug)]
pub enum PhaseState {
    InputAccumulation {
        current_epoch: AccumulatingEpoch,
    },

    ExpiredInputAccumulation {
        sealing_epoch: AccumulatingEpoch,
    },

    AwaitingConsensus {
        sealed_epoch: SealedEpoch,
        current_epoch: AccumulatingEpoch,
        round_start: U256,
    },

    ConsensusTimeout {
        sealed_epoch: SealedEpoch,
        current_epoch: AccumulatingEpoch,
    },

    AwaitingDispute {
        sealed_epoch: SealedEpoch,
        current_epoch: AccumulatingEpoch,
    },
    // TODO: add dispute timeout when disputes are turned on.
}

impl PhaseState {
    pub fn consensus_round_start(&self) -> Option<U256> {
        match self {
            PhaseState::AwaitingConsensus {
                round_start,
                sealed_epoch,
                ..
            } => match sealed_epoch.claims {
                None => None,
                Some(c) => {
                    Some(std::cmp::max(*round_start, c.first_claim_timestamp()))
                }
            },
            _ => None,
        }
    }
}
