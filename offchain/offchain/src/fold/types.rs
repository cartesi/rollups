use offchain_core::ethers;

use ethers::abi::{encode, Token};
use ethers::types::{Address, H256, U256, U64};
use im::{HashMap, HashSet, Vector};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::sync::Arc;

pub const MAX_NUM_VALIDATORS: usize = 8;

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
    pub dapp_contract_address: Address,
}

impl EpochInputState {
    pub fn new(epoch_number: U256, dapp_contract_address: Address) -> Self {
        Self {
            epoch_number,
            inputs: Vector::new(),
            dapp_contract_address,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputState {
    pub dapp_contract_address: Address,
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

    pub dapp_contract_address: Address,
}

impl FinalizedEpochs {
    pub fn new(initial_epoch: U256, dapp_contract_address: Address) -> Self {
        Self {
            finalized_epochs: Vector::new(),
            initial_epoch,
            dapp_contract_address,
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
    pub dapp_contract_address: Address,
}

/// Active epoch currently receiveing inputs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccumulatingEpoch {
    pub epoch_number: U256,
    pub inputs: EpochInputState,
    pub dapp_contract_address: Address,
}

impl AccumulatingEpoch {
    pub fn new(dapp_contract_address: Address, epoch_number: U256) -> Self {
        Self {
            epoch_number,
            inputs: EpochInputState::new(epoch_number, dapp_contract_address),
            dapp_contract_address,
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

impl Default for PhaseState {
    fn default() -> Self {
        Self::InputAccumulation {}
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImmutableState {
    /// duration of input accumulation phase in seconds
    pub input_duration: U256,

    /// duration of challenge period in seconds
    pub challenge_period: U256,

    /// timestamp of the contract creation
    pub contract_creation_timestamp: U256,

    /// decentralized application contract address
    pub dapp_contract_address: Address,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NumClaims {
    pub validator_address: Address,
    pub num_claims_mades: U256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidatorManagerState {
    // each tuple containing (validator_address, #claims_made_so_far)
    // note that when a validator gets removed, the corresponding option
    // becomes `None` and this `None` can appear anywhere in the array
    pub num_claims: [Option<NumClaims>; MAX_NUM_VALIDATORS],
    // validators that have claimed in the current unfinalized epoch
    pub claiming: Vec<Address>,
    // validators that lost the disputes
    pub validators_removed: Vec<Address>,
    pub num_finalized_epochs: U256,
    pub dapp_contract_address: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ERC20BalanceState {
    pub erc20_address: Address,
    pub owner_address: Address,
    pub balance: U256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BankState {
    pub bank_address: Address,
    pub dapp_address: Address,
    pub balance: U256,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NumRedeemed {
    pub validator_address: Address,
    pub num_claims_redeemed: U256,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeeManagerState {
    pub dapp_contract_address: Address,
    pub bank_address: Address,
    pub fee_per_claim: U256, // only the current value
    // Tuple containing (validator, #claims_redeemed_so_far)
    pub num_redeemed: [Option<NumRedeemed>; MAX_NUM_VALIDATORS],
    pub bank_balance: U256,
    // Uncommitted balance equals the balance of bank contract minus
    // the amount of to-be-redeemed fees
    // un-finalized claims are not considered
    pub uncommitted_balance: i128,
}

pub struct FeeIncentiveStrategy {
    pub num_buffer_epochs: usize,
    pub num_claims_triger_redeem: usize,
    pub minimum_required_fee: U256,
}

impl Default for FeeIncentiveStrategy {
    fn default() -> Self {
        FeeIncentiveStrategy {
            // ideally fee manager should have enough uncommitted balance for at least 4 epochs
            num_buffer_epochs: 4,
            // when the number of redeemable claims reaches this value, call `redeem`
            num_claims_triger_redeem: 4,
            // zero means an altruistic validator
            minimum_required_fee: U256::zero(),
        }
    }
}

impl FeeManagerState {
    pub fn should_redeem(
        &self,
        validator_manager_state: &ValidatorManagerState,
        validator_address: Address,
        strategy: &FeeIncentiveStrategy,
    ) -> bool {
        let num_claims_triger_redeem =
            U256::from(strategy.num_claims_triger_redeem);

        let num_redeemable_claims = self
            .num_redeemable_claims(validator_manager_state, validator_address);

        num_redeemable_claims >= num_claims_triger_redeem
    }

    pub fn num_redeemable_claims(
        &self,
        validator_manager_state: &ValidatorManagerState,
        validator_address: Address,
    ) -> U256 {
        // number of total claims for the validator
        let num_claims = validator_manager_state.num_claims;
        let mut validator_claims = U256::zero();
        for i in 0..MAX_NUM_VALIDATORS {
            // find validator address in `num_claims`
            if let Some(num_claims_struct) = &num_claims[i] {
                if num_claims_struct.validator_address == validator_address {
                    validator_claims = num_claims_struct.num_claims_mades;
                    break;
                }
            }
        }

        // number of redeemed claims for the validator
        let num_redeemed = self.num_redeemed;
        let mut validator_redeemed = U256::zero();
        for i in 0..MAX_NUM_VALIDATORS {
            // find validator address in `num_redeemed`
            if let Some(num_redeemed_struct) = &num_redeemed[i] {
                if num_redeemed_struct.validator_address == validator_address {
                    validator_redeemed =
                        num_redeemed_struct.num_claims_redeemed;
                    break;
                }
            }
        }

        assert!(
            validator_claims >= validator_redeemed,
            "validator_claims should be no less than validator_redeemed"
        );

        validator_claims - validator_redeemed
    }

    pub fn sufficient_uncommitted_balance(
        &self,
        validator_manager_state: &ValidatorManagerState,
        strategy: &FeeIncentiveStrategy,
    ) -> bool {
        if strategy.minimum_required_fee == U256::zero() {
            return true;
        }

        if self.fee_per_claim < strategy.minimum_required_fee {
            return false;
        }

        let validators_removed =
            validator_manager_state.validators_removed.len();

        assert!(
            MAX_NUM_VALIDATORS >= validators_removed,
            "current_num_validators out of range"
        );

        let current_num_validators =
            (MAX_NUM_VALIDATORS - validators_removed) as i128;

        let fee_per_claim =
            i128::try_from(self.fee_per_claim.as_u128()).unwrap();

        let balance_buffer = fee_per_claim
            * current_num_validators
            * (strategy.num_buffer_epochs as i128);

        self.uncommitted_balance >= balance_buffer
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RollupsState {
    pub constants: ImmutableState,

    pub initial_epoch: U256,
    pub finalized_epochs: FinalizedEpochs,
    pub current_epoch: AccumulatingEpoch,

    pub current_phase: PhaseState,

    pub output_state: OutputState,
    pub validator_manager_state: ValidatorManagerState,
    pub fee_manager_state: FeeManagerState,
}
