"use strict";

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("FinalizedEpoches", {
			id: {
				type: Sequelize.UUID,
				allowNull: false,
				primaryKey: true
			},
			epoch_number: {
				type: Sequelize.STRING,
				allowNull: false
			},
			hash: {
				type: Sequelize.STRING,
				allowNull: false
			},
			epochInputStateId: {
				type: Sequelize.UUID,
				allowNull: false
			},
			finalized_block_hash: {
				type: Sequelize.STRING,
				allowNull: false
			},
			finalized_block_number: {
				type: Sequelize.STRING,
				allowNull: false
			},
			FinalizedEpochId: {
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
		await queryInterface.dropTable("FinalizedEpoches");
	}
};
