"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("Inputs", {
			id: {
				allowNull: false,
				primaryKey: true,
				type: Sequelize.UUID
			},
			sender: {
				type: Sequelize.STRING,
				allowNull: false
			},
			payload: {
				type: Sequelize.ARRAY(Sequelize.STRING),
				allowNull: false
			},
			timestamp: {
				type: Sequelize.STRING,
				allowNull: false
			},
			epoch_input_state_id: {
				type: Sequelize.UUID,
				allowNull: false
			},
			createdAt: {
				type: Sequelize.DATE,
				allowNull: false
			},
			updatedAt: {
				type: Sequelize.DATE,
				allowNull: false
			}
		});
	},
	down: async (queryInterface, Sequelize) => {
		await queryInterface.dropTable("Inputs");
	}
};
