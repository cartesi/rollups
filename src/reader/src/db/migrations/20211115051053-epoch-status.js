"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("EpochStatuses", {
			session_id: {
				type: Sequelize.UUID,
				allowNull: false,
				primaryKey: true
			},
			epoch_index: {
				type: Sequelize.STRING,
				allowNull: false,
				primaryKey: true
			},
			state: {
				type: Sequelize.STRING,
				allowNull: false
			},
			most_recent_machine_hash: {
				type: Sequelize.STRING,
				allowNull: false
			},
			most_recent_vouchers_epoch_root_hash: {
				type: Sequelize.STRING,
				allowNull: false
			},
			most_recent_notices_epoch_root_hash: {
				type: Sequelize.STRING,
				allowNull: false
			},
			pending_input_count: {
				type: Sequelize.STRING,
				allowNull: false
			},
			taint_status: {
				type: Sequelize.JSON,
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
		await queryInterface.dropTable("EpochStatuses");
	}
};
