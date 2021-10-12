import { gql } from "apollo-server-express";

export default gql`
	# Query types
	type ImmutableState {
		id: ID!
		input_duration: String!
		challenge_period: String!
		contract_creation_timestamp: String!
		input_contract_address: String!
		output_contract_address: String!
		validator_contract_address: String!
		dispute_contract_address: String!
		descartesv2_contract_address: String!
	}

	type Input {
		id: ID!
		sender: String!
		timestamp: String!
		payload: [String]!
	}

	type EpochInputState {
		id: ID!
		epoch_number: String!
		# ID of the Input type
		inputs: [ID]!
		input_contract_address: String!
	}

	type FinalizedEpoch {
		id: ID!
		epoch_number: String!
		hash: Int!
		inputs: EpochInputState!
		finalized_block_hash: String!
		finalized_block_number: Int!
	}

	type FinalizedEpochs {
		id: ID!
		finalized_epochs: [FinalizedEpoch]!
		initial_epoch: String!
		descartesv2_contract_address: String!
		input_contract_address: String!
	}

	type AccumulatingEpoch {
		id: ID!
		epoch_number: String!
		inputs: EpochInputState!
		descartesv2_contract_address: String!
		input_contract_address: String!
	}

	enum PhaseState {
		InputAccumulation
		EpochSealedAwaitingFirstClaim
		AwaitingConsensusNoConflict
		AwaitingConsensusAfterConflict
		ConsensusTimeout
		AwaitingDispute
	}

	type IntegerBool {
		integer: Boolean!
	}

	type IntegerInnerObject {
		integer: IntegerBool
	}

	type IntegerObject {
		integer: IntegerInnerObject
	}

	type OutputState {
		id: ID!
		output_address: String!
		outputs: IntegerObject
	}

	# Mutation Inputs
	input ImmutableStateInput {
		input_duration: String!
		challenge_period: String!
		contract_creation_timestamp: String!
		input_contract_address: String!
		output_contract_address: String!
		validator_contract_address: String!
		dispute_contract_address: String!
		descartesv2_contract_address: String!
	}

	input InputData {
		sender: String!
		timestamp: String!
		payload: [String]!
	}

	input EpochInputStateInput {
		epoch_number: String!
		inputs: [InputData]!
		input_contract_address: String!
	}

	input FinalizedEpochInput {
		epoch_number: String!
		hash: Int!
		inputs: EpochInputStateInput!
		finalized_block_hash: String!
		finalized_block_number: Int!
	}

	input FinalizedEpochsInput {
		initial_epoch: String!
		descartesv2_contract_address: String!
		input_contract_address: String!
		finalized_epochs: [FinalizedEpochInput]!
	}
	input AccumulatingEpochInput {
		epoch_number: String!
		descartesv2_contract_address: String!
		input_contract_address: String!
		inputs: EpochInputStateInput!
	}

	input IntegerBoolInput {
		integer: Boolean!
	}

	input IntegerInnerObjectInput {
		integer: IntegerBoolInput!
	}

	input IntegerObjectInput {
		integer: IntegerInnerObjectInput!
	}

	input OutputStateInput {
		output_address: String!
		outputs: IntegerObjectInput!
	}

	type Query {
		constants(first: Int): [ImmutableState]!
		initial_epoch: String!
		finalized_epochs: [FinalizedEpochs]!
		current_epoch: [AccumulatingEpoch]!
		current_phase: [PhaseState]!
		output_state: [OutputState]!
	}

	type Mutation {
		constants(input: [ImmutableStateInput]!): [ImmutableState]!
		initial_epoch(input: String!): String!
		finalized_epochs(input: [FinalizedEpochsInput]!): [FinalizedEpochs]!
		current_epoch(input: AccumulatingEpochInput!): AccumulatingEpoch!
		current_phase(input: PhaseState!): PhaseState!
		output_state(input: OutputStateInput!): OutputState!
	}
`;
