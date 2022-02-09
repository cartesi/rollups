"use strict";

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("FinalizedEpochs", {
			id: {
				type: Sequelize.UUID,
				allowNull: false,
				primaryKey: true
			},
			initial_epoch: {
				type: Sequelize.STRING,
				allowNull: false
			},
			dapp_contract_address: {
				type: Sequelize.STRING,
				allowNull: false
			},
			rollups_hash: Sequelize.STRING,
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
		await queryInterface.dropTable("FinalizedEpochs");
	}
};
