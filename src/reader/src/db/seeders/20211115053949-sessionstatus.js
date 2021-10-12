"use strict";
const { v4: uuidv4 } = require("uuid");

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.bulkInsert(
			"SessionStatuses",
			[
				{
					session_id: "6027956b-30c5-4e9c-a156-88cfa4da48e1",
					active_epoch_index: 400,
					epoch_index: [300, 400, 500],
					taint_status: `{
						"error_code": 200,
						"error_message": "No error occured"
					}`,
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					session_id: uuidv4(),
					active_epoch_index: 700,
					epoch_index: [600, 700, 800],
					taint_status: `{
						"error_code": 404,
						"error_message": "Not found"
					}`,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);
	},

	down: async (queryInterface, Sequelize) => {
		await queryInterface.bulkDelete("SessionStatuses", null, {});
	}
};
