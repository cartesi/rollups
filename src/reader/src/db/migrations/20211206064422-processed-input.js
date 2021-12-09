"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("ProcessedInputs", {
			input_index: {
				type: Sequelize.STRING,
				allowNull: false
			},
			most_recent_machine_hash: {
				type: Sequelize.JSON,
			},
			voucher_hashes_in_epoch: {
				type: Sequelize.JSON,
			},
			notice_hashes_in_epoch: {
				type: Sequelize.JSON,
			},
			// reports: {
			// 	type: Sequelize.ARRAY(Sequelize.STRING),
			// 	allowNull: false
			// },
			result: { 
				type: Sequelize.UUID,
			},
			skip_reason: {
				type: Sequelize.STRING,
			},
			//epoch_status_id: Sequelize.UUID,
			session_id: {
				type: Sequelize.STRING,
				allowNull: false,
			},
			epoch_index: {
				type: Sequelize.STRING,
				allowNull: false
			},
			createdAt: {
				type: Sequelize.DATE
			},
			updatedAt: {
				type: Sequelize.DATE
			}
		});
	},
	down: async (queryInterface, Sequelize) => {
		await queryInterface.dropTable("ProcessedInputs");
	}
};
