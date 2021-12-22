"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("Vouchers", {
			session_id: {
				type: Sequelize.STRING,
				allowNull: false,
				primaryKey: true
			},
			epoch_index: {
				type: Sequelize.STRING,
				allowNull: false,
				primaryKey: true
			},
			input_index: {
				type: Sequelize.STRING,
				allowNull: false,
				primaryKey: true
			},
			voucher_index: {
				type: Sequelize.STRING,
				allowNull: false,
				primaryKey: true
			},
			keccak: {
				type: Sequelize.STRING,
			},
			Address: {
				type: Sequelize.STRING,
				allowNull: false
			},
			payload: {
				type: Sequelize.STRING,
				allowNull: false
			},
			keccak_in_voucher_hashes: {
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
		await queryInterface.dropTable("Vouchers");
	}
};
