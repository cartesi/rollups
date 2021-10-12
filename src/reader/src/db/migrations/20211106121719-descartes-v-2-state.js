"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("DescartesV2States", {
			id: {
				allowNull: false,
				primaryKey: true,
				type: Sequelize.UUID
			},
			block_hash: {
				type: Sequelize.STRING,
				allowNull: false
			},
			constants: {
				type: Sequelize.ARRAY(Sequelize.STRING),
				allowNull: false
			},
			initial_epoch: {
				type: Sequelize.STRING,
				allowNull: false
			},
			current_epoch: {
				type: Sequelize.UUID,
				allowNull: false
			},
			current_phase: {
				type: Sequelize.STRING,
				allowNull: false
			},
			voucher_state: {
				type: Sequelize.UUID,
				allowNull: false
			},
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
		await queryInterface.dropTable("DescartesV2States");
	}
};
