"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("ProcessedInputs", {
			id: {
				type: Sequelize.UUID,
				allowNull: false,
				primaryKey: true
			},
			input_index: {
				type: Sequelize.INTEGER,
				allowNull: false
			},
			most_recent_machine_hash: {
				type: Sequelize.JSON,
				allowNull: false
			},
			voucher_hashes_in_epoch: {
				type: Sequelize.JSON,
				allowNull: false
			},
			notice_hashes_in_epoch: {
				type: Sequelize.JSON,
				allowNull: false
			},
			// reports: {
			// 	type: Sequelize.ARRAY(Sequelize.STRING),
			// 	allowNull: false
			// },
			// result: Sequelize.UUID,
			skip_reason: {
				type: Sequelize.STRING,
				allowNull: false
			},
			epoch_status_id: Sequelize.UUID,
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
