"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("Notices", {
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
			notice_index: {
				type: Sequelize.STRING,
				allowNull: false,
				primaryKey: true
			},
			keccak: {
				type: Sequelize.STRING,
			},
			payload: {
				type: Sequelize.STRING,
				allowNull: false
			},
			keccak_in_notice_hashes: {
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
		await queryInterface.dropTable("Notices");
	}
};
