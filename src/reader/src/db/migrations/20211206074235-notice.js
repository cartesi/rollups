"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("Notices", {
			id: {
				allowNull: false,
				primaryKey: true,
				type: Sequelize.UUID
			},
			keccak: {
				type: Sequelize.STRING,
				allowNull: false
			},
			payload: {
				type: Sequelize.STRING,
				allowNull: false
			},
			keccak_in_notice_hashes: {
				type: Sequelize.STRING,
				allowNull: false
			},
			input_result_id: {
				type: Sequelize.UUID
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
		await queryInterface.dropTable("Notices");
	}
};
