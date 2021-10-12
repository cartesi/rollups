"use strict";

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("VoucherStates", {
			id: {
				type: Sequelize.UUID,
				allowNull: false,
				primaryKey: true
			},
			voucher_address: {
				type: Sequelize.STRING,
				allowNull: false
			},
			vouchers: {
				type: Sequelize.JSON,
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
		await queryInterface.dropTable("VoucherStates");
	}
};
