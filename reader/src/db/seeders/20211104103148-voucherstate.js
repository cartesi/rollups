"use strict";
const { v4: uuidv4 } = require("uuid");

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.bulkInsert(
			"VoucherStates",
			[
				{
					id: uuidv4(),
					voucher_address: "voucher address 1",
					vouchers: `{ "intger": { "integer": { "integer": false } } }`,
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					id: uuidv4(),
					voucher_address: "voucher address 2",
					vouchers: `{ "intger": { "integer": { "integer": false } } }`,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);
	},

	down: async (queryInterface, Sequelize) => {
		await queryInterface.bulkDelete("VoucherStates", null, {});
	}
};
