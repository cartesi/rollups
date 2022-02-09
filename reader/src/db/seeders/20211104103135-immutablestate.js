"use strict";
const { v4: uuidv4 } = require("uuid");

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.bulkInsert(
			"ImmutableStates",
			[
				{
					id: uuidv4(),
					input_duration: "An Input duration",
					challenge_period: "A challenge period",
					contract_creation_timestamp: new Date(),
					dapp_contract_address: "Address 1",
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: uuidv4(),
					input_duration: "An Input duration",
					challenge_period: "A challenge period",
					contract_creation_timestamp: new Date(),
					dapp_contract_address: "Address 2",
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);
	},

	down: async (queryInterface, Sequelize) => {
		await queryInterface.bulkDelete("ImmutableStates", null, {});
	}
};
