"use strict";
const { v4: uuidv4 } = require("uuid");

const rollups_hash = "a particular hash";
const parentId1 = uuidv4();
const parentId2 = uuidv4();

const epochInputId1 = uuidv4();
const epochInputId2 = uuidv4();

const inputId1 = uuidv4();
const inputId2 = uuidv4();

const immutableStateId1 = uuidv4();
const immutableStateId2 = uuidv4();

const accumulatingEpochId = uuidv4();

const voucherStateId = uuidv4();

const inputs = [inputId1, inputId2];

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.bulkInsert(
			"ImmutableStates",
			[
				{
					id: immutableStateId1,
					input_duration: 123,
					challenge_period: 1234567890,
					contract_creation_timestamp: new Date(),
					dapp_contract_address: "Address 1",
					rollups_hash,
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: immutableStateId2,
					input_duration: 123,
					challenge_period: 1234567890,
					contract_creation_timestamp: new Date(),
					dapp_contract_address: "Address 2",
					rollups_hash,
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
					epoch_input_state_id: epochInputId1,
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: inputId2,
					sender: "Sender 2",
					timestamp: "Timestamp 2",
					payload: ["Payload 2"],
					epoch_input_state_id: epochInputId2,
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
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: epochInputId2,
					epoch_number: "1",
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
					dapp_contract_address: "Address 1",
					epochInputStateId: epochInputId1,
					rollups_hash: rollups_hash,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert(
			"VoucherStates",
			[
				{
					id: voucherStateId,
					voucher_address: "voucher address 1",
					vouchers: `{ "intger": { "integer": { "integer": false } } }`,
					rollups_hash,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert(
			"RollupsStates",
			[
				{
					id: uuidv4(),
					block_hash: rollups_hash,
					constants: immutableStateId1,
					initial_epoch: "1234567890",
					current_epoch: accumulatingEpochId,
					current_phase: "InputAccumulation",
					voucher_state: voucherStateId,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);
	},

	down: async (queryInterface, Sequelize) => {
		await queryInterface.bulkDelete("RollupsStates", null, {});
	}
};
