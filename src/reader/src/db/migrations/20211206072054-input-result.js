"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("InputResults", {
			/*
			id: {
				allowNull: false,
				primaryKey: true,
				type: Sequelize.UUID
			},
			*/
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
			voucher_hashes_in_machine: {
				type: Sequelize.UUID,
				allowNull: false
			},
			notice_hashes_in_machine: {
				type: Sequelize.UUID,
				allowNull: false
			},
			processed_input_id: Sequelize.UUID,
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
		await queryInterface.dropTable("InputResults");
	}
};
