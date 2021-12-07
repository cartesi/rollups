"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("Reports", {
			id: {
				allowNull: false,
				primaryKey: true,
				type: Sequelize.UUID
			},
			payload: {
				type: Sequelize.STRING
			},
			processed_input_id: Sequelize.UUID,
			createdAt: {
				allowNull: false,
				type: Sequelize.DATE
			},
			updatedAt: {
				allowNull: false,
				type: Sequelize.DATE
			}
		});
	},
	down: async (queryInterface, Sequelize) => {
		await queryInterface.dropTable("Reports");
	}
};
