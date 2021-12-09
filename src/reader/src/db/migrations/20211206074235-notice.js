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
			},
			payload: {
				type: Sequelize.STRING,
				allowNull: false
			},
			keccak_in_notice_hashes: {
				type: Sequelize.JSON,
			},
			session_id: {
				type: Sequelize.STRING,
				allowNull: false,
			},
			epoch_index: {
				type: Sequelize.STRING,
				allowNull: false
			},
			input_index: {
				type: Sequelize.STRING,
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
		await queryInterface.dropTable("Notices");
	}
};
