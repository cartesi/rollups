"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("SessionStatuses", {
			session_id: {
				type: Sequelize.UUID,
				allowNull: false,
				primaryKey: true
			},
			active_epoch_index: {
				type: Sequelize.INTEGER,
				allowNull: false
			},
			epoch_index: {
				type: Sequelize.ARRAY(Sequelize.INTEGER),
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
		await queryInterface.dropTable("SessionStatuses");
	}
};
