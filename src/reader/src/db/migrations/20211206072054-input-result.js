"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("InputResults", {
			id: {
				allowNull: false,
				primaryKey: true,
				type: Sequelize.UUID
			},
			voucher_hashes_in_machine: {
				type: Sequelize.JSON,
				allowNull: false
			},
			// vouchers: {
			// 	type: Sequelize.ARRAY(Sequelize.STRING),
			// 	allowNull: false
			// },
			notice_hashes_in_machine: {
				type: Sequelize.JSON,
				allowNull: false
			},
			// notices: {
			// 	type: Sequelize.ARRAY(Sequelize.STRING),
			// 	allowNull: false
			// },
			processed_input_id: {
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
		await queryInterface.dropTable("InputResults");
	}
};
