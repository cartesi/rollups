"use strict";
const { v4: uuidv4 } = require("uuid");

const epochInputId1 = uuidv4();
const epochInputId2 = uuidv4();

const inputId1 = uuidv4();
const inputId2 = uuidv4();

module.exports = {
	up: async (queryInterface, Sequelize) => {
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
					input_contract_address: "Address 1",
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: epochInputId2,
					epoch_number: "1",
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
					id: uuidv4(),
					epoch_number: 500,
					descartesv2_contract_address: "Address 1",
					input_contract_address: "Address 1",
					epochInputStateId: epochInputId1,
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: uuidv4(),
					epoch_number: 600,
					descartesv2_contract_address: "Address 2",
					input_contract_address: "Address 2",
					epochInputStateId: epochInputId2,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);
	},

	down: async (queryInterface, Sequelize) => {
		await queryInterface.bulkDelete("AccumulatingEpoches", null, {});
	}
};
