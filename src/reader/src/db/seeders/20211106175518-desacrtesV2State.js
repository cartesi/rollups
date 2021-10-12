"use strict";
const { v4: uuidv4 } = require("uuid");

const descartes_hash = uuidv4();
const parentId1 = uuidv4();
const parentId2 = uuidv4();

const epochInputId1 = uuidv4();
const epochInputId2 = uuidv4();

const inputId1 = uuidv4();
const inputId2 = uuidv4();

const immutableStateId1 = uuidv4();
const immutableStateId2 = uuidv4();

const accumulatingEpochId = uuidv4();

const outputStateId = uuidv4();

const inputs = [inputId1, inputId2];

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.bulkInsert(
			"ImmutableStates",
			[
				{
					id: immutableStateId1,
					input_duration: 1000,
					challenge_period: 2000,
					contract_creation_timestamp: 3000,
					input_contract_address: "Address 1",
					output_contract_address: "Address 1",
					validator_contract_address: "Address 1",
					dispute_contract_address: "Address 1",
					descartesv2_contract_address: "Address 1",
					descartes_hash,
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: immutableStateId2,
					input_duration: 4000,
					challenge_period: 5000,
					contract_creation_timestamp: 6000,
					input_contract_address: "Address 2",
					output_contract_address: "Address 2",
					validator_contract_address: "Address 2",
					dispute_contract_address: "Address 2",
					descartesv2_contract_address: "Address 2",
					descartes_hash,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert(
			"Inputs",
			[
				{
					id: inputId1,
					sender: "Sender 1",
					timestamp: "Timestamp 1",
					payload: ["Payload 1"],
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: inputId2,
					sender: "Sender 2",
					timestamp: "Timestamp 2",
					payload: ["Payload 2"],
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert(
			"EpochInputStates",
			[
				{
					id: epochInputId1,
					epoch_number: "1",
					inputs,
					input_contract_address: "Address 1",
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: epochInputId2,
					epoch_number: "1",
					inputs,
					input_contract_address: "Address 2",
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert(
			"AccumulatingEpoches",
			[
				{
					id: accumulatingEpochId,
					epoch_number: 500,
					descartesv2_contract_address: "Address 1",
					input_contract_address: "Address 1",
					epochInputStateId: epochInputId1,
					descartes_hash,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert(
			"FinalizedEpochs",
			[
				{
					id: parentId1,
					initial_epoch: "40",
					descartesv2_contract_address: "Address 1",
					input_contract_address: "Address 1",
					descartes_hash,
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: parentId2,
					initial_epoch: "80",
					descartesv2_contract_address: "Address 2",
					input_contract_address: "Address 2",
					descartes_hash,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert(
			"FinalizedEpoches",
			[
				{
					id: uuidv4(),
					epoch_number: "10",
					hash: 20,
					finalized_block_hash: "Hash",
					finalized_block_number: 30,
					epochInputStateId: epochInputId1,
					FinalizedEpochId: parentId1,
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: uuidv4(),
					epoch_number: "50",
					hash: 60,
					finalized_block_hash: "Hash",
					finalized_block_number: 70,
					FinalizedEpochId: parentId2,
					epochInputStateId: epochInputId2,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert(
			"OutputStates",
			[
				{
					id: outputStateId,
					output_address: "Output address 1",
					outputs: `{ "intger": { "integer": { "integer": false } } }`,
					descartes_hash,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert(
			"DescartesV2States",
			[
				{
					block_hash: descartes_hash,
					constants: [immutableStateId1, immutableStateId2],
					initial_epoch: "1234567890",
					finalized_epochs: [parentId1, parentId2],
					current_epoch: accumulatingEpochId,
					current_phase: "InputAccumulation",
					output_state: outputStateId,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);
	},

	down: async (queryInterface, Sequelize) => {
		await queryInterface.bulkDelete("OutputStates", null, {});
	}
};
