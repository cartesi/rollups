import db from "../db/models";
import { expect } from "chai";

import { graphqlTestCall } from "./graphqlTestCall";
// import { User } from "./entity/User";

// Mutation Calls
const constantsMutation = `
  mutation createImmutableState {
		constants(
			input: [
				{
					input_duration: "1234567890"
					challenge_period: "1234567890"
					contract_creation_timestamp: "1234567890"
					input_contract_address: "An Address"
					output_contract_address: "An Address"
					validator_contract_address: "An Address"
					dispute_contract_address: "An Address"
					descartesv2_contract_address: "An Address"
				}
				{
					input_duration: "15"
					challenge_period: "16"
					contract_creation_timestamp: "17"
					input_contract_address: "An Address"
					output_contract_address: "An Address"
					validator_contract_address: "An Address"
					dispute_contract_address: "An Address"
					descartesv2_contract_address: "An Address"
				}
			]
		) {
			id
			input_duration
			challenge_period
			contract_creation_timestamp
			input_contract_address
			output_contract_address
			validator_contract_address
			dispute_contract_address
			descartesv2_contract_address
		}
	}
`;

const finalizedEpochsMutation = `
  mutation crateFinalizedEpochs {
		finalized_epochs(
			input: [
				{
					initial_epoch: "33"
					descartesv2_contract_address: "An address"
					input_contract_address: "Another address"
					finalized_epochs: [
						{
							epoch_number:"100"
							hash: 200
							finalized_block_hash: "A hash"
							finalized_block_number: 300
							inputs: {
								epoch_number: "400"
								inputs: [
									{
										sender: "A sender"
										timestamp: "A timestamp"
										payload: "A payload"
									}
								]
								input_contract_address: "An Address"
							}
						}
					]
				}
			]
		) {
			id
			finalized_epochs {
				id
				epoch_number
				hash
				inputs {
					id
					epoch_number
					inputs
					input_contract_address
				}
				finalized_block_hash
				finalized_block_number
			}
			initial_epoch
			descartesv2_contract_address
			input_contract_address
			
		}
	}
`;

const currentEpochMutation = `
  mutation crateAccumulatingEpoch {
		current_epoch (input: {
			epoch_number: "1000"
			descartesv2_contract_address: "An Address"
			inputs: {
				epoch_number:"2000"
				input_contract_address: "Another Address again"
				inputs: [
					{
						sender: "Sender 1"
						timestamp: "Timestamp 1"
						payload: "Payload 1"
					},
					{
						sender: "Sender 2"
						timestamp: "Timestamp 2"
						payload: "Payload 2"
					}
				]
			}
			input_contract_address: "Another Address"
		}) {
			id
			epoch_number
			inputs {
				id
				epoch_number
				inputs
				input_contract_address
			}
			descartesv2_contract_address
			input_contract_address
		}
	}
`;

const outputStateMutation = `
	mutation createOutpuState {
		output_state(input: {
			output_address: "The output address"
			outputs: {
				integer:{
					integer: {
						integer:false
					}
				}
			}
		})
	}
`;

const descartesMutation = `
	mutation createDescartes {
		DescartesState(
			input: {
				constants: [
					{
						input_duration: "1234567890"
						challenge_period: "1234567890"
						contract_creation_timestamp: "1234567890"
						input_contract_address: "An Address"
						output_contract_address: "An Address"
						validator_contract_address: "An Address"
						dispute_contract_address: "An Address"
						descartesv2_contract_address: "An Address"
					}
					{
						input_duration: "15"
						challenge_period: "16"
						contract_creation_timestamp: "17"
						input_contract_address: "An Address"
						output_contract_address: "An Address"
						validator_contract_address: "An Address"
						dispute_contract_address: "An Address"
						descartesv2_contract_address: "An Address"
					}
				]
				initial_epoch: "1234567890"
				finalized_epochs: [
					{
						initial_epoch: "33"
						descartesv2_contract_address: "An address"
						input_contract_address: "Another address"
						finalized_epochs: [
							{
								epoch_number: "100"
								hash: 200
								finalized_block_hash: "A hash"
								finalized_block_number: 300
								inputs: {
									epoch_number: "400"
									inputs: [
										{
											sender: "A sender"
											timestamp: "A timestamp"
											payload: "A payload"
										}
									]
									input_contract_address: "An Address"
								}
							}
						]
					}
				]
				current_epoch: {
					epoch_number: "1000"
					descartesv2_contract_address: "An Address"
					inputs: {
						epoch_number: "2000"
						input_contract_address: "Another Address again"
						inputs: [
							{
								sender: "Sender 1"
								timestamp: "Timestamp 1"
								payload: "Payload 1"
							}
							{
								sender: "Sender 2"
								timestamp: "Timestamp 2"
								payload: "Payload 2"
							}
						]
					}
					input_contract_address: "Another Address"
				}
				current_phase: InputAccumulation
				output_state: {
					output_address: "The output address"
					outputs: { integer: { integer: { integer: false } } }
				}
			}
		) {
			block_hash
			constants {
				id
				input_duration
				challenge_period
				input_contract_address
				contract_creation_timestamp
				output_contract_address
				validator_contract_address
				dispute_contract_address
				descartesv2_contract_address
			}
			initial_epoch
			finalized_epochs {
				id
				finalized_epochs {
					id
					epoch_number
					hash
					inputs {
						id
						epoch_number
						inputs
						input_contract_address
					}
					finalized_block_hash
					finalized_block_number
				}
				initial_epoch
				descartesv2_contract_address
				input_contract_address
			}
			current_epoch {
				id
				epoch_number
				inputs {
					id
					epoch_number
					inputs
					input_contract_address
				}
				descartesv2_contract_address
				input_contract_address
			}
			output_state {
				id
				output_address
				outputs {
					integer {
						integer {
							integer
						}
					}
				}
			}
		}
	}
`;

// Query Calls
const constantsQuery = `
  query constants {
		constants {
			id
			input_duration
			challenge_period
			input_contract_address
			contract_creation_timestamp
			output_contract_address
			validator_contract_address
			dispute_contract_address
			descartesv2_contract_address
		}
	}
`;

const finalizedEpochsQuery = `
	query finalized_epochs {
		finalized_epochs {
			id
			finalized_epochs {
				id
				epoch_number
				hash
				inputs {
					id
					epoch_number
					inputs
					input_contract_address
				}
				finalized_block_hash
				finalized_block_number
			}
			initial_epoch
			descartesv2_contract_address
			input_contract_address
		}
	}
`;

const currentEpochQuery = `
	query current_epoch {
		current_epoch {
			id
			epoch_number
			inputs {
				id
				epoch_number
				inputs
				input_contract_address
			}
			descartesv2_contract_address
			input_contract_address
		}
	}
`;

const outputStateMutaion = `
	query output_state {
		output_state {
			id
			output_address
			outputs {
				integer {
					integer {
						integer
					}
				}
			}
		}
	}
`;

const descartesQuery = `
	query descartes {
		DescartesState {
			block_hash
			constants {
				id
				input_duration
				challenge_period
				input_contract_address
				contract_creation_timestamp
				output_contract_address
				validator_contract_address
				dispute_contract_address
				descartesv2_contract_address
			}
			initial_epoch
			finalized_epochs {
				id
				finalized_epochs {
					id
					epoch_number
					hash
					inputs {
						id
						epoch_number
						inputs
						input_contract_address
					}
					finalized_block_hash
					finalized_block_number
				}
				initial_epoch
				descartesv2_contract_address
				input_contract_address
			}
			current_epoch {
				id
				epoch_number
				inputs {
					id
					epoch_number
					inputs
					input_contract_address
				}
				descartesv2_contract_address
				input_contract_address
			}
			output_state {
				id
				output_address
				outputs {
					integer {
						integer {
							integer
						}
					}
				}
			}
		}
	}
`;

beforeEach(async () => {
	await db.sequelize.sync();
});

afterEach(async () => {
	await db.sequelize.drop();
	await db.sequelize.close();
});

describe("resolvers", () => {
	it("constants mutation and query", async () => {
		const constantsMutationRepsonse = await graphqlTestCall(constantsMutation);
		expect(constantsMutationRepsonse?.data?.constants.length).to.equal(2);

		const constantsQueryResponse = await graphqlTestCall(constantsQuery);
		expect(constantsQueryResponse?.data?.constants).to.equal(
			constantsMutationRepsonse?.data?.constants
		);
	});
});
