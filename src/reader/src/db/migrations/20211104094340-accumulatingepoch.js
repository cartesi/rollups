"use strict";

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("AccumulatingEpoches", {
			id: {
				type: Sequelize.UUID,
				allowNull: false,
				primaryKey: true
			},
			epoch_number: {
				type: Sequelize.STRING,
				allowNull: false
			},
			descartesv2_contract_address: {
				type: Sequelize.STRING,
				allowNull: false
			},
			input_contract_address: {
				type: Sequelize.STRING,
				allowNull: false
			},
			epochInputStateId: {
				type: Sequelize.UUID,
				allowNull: false
			},
			descartes_hash: Sequelize.STRING,
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
		await queryInterface.dropTable("AccumulatingEpoches");
	}
};
