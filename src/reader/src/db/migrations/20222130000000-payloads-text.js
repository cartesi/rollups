"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.changeColumn('Inputs', 'payload', {
			type: Sequelize.ARRAY(Sequelize.TEXT)
		});
		await queryInterface.changeColumn('Reports', 'payload', {
			type: Sequelize.TEXT
		});
		await queryInterface.changeColumn('Vouchers', 'payload', {
			type: Sequelize.TEXT
		});
		await queryInterface.changeColumn('Notices', 'payload', {
			type: Sequelize.TEXT
		});
	},
	down: async (queryInterface, Sequelize) => {
		await queryInterface.changeColumn('Inputs', 'payload', {
			type: Sequelize.ARRAY(Sequelize.STRING)
		});
		await queryInterface.changeColumn('Reports', 'payload', {
			type: Sequelize.STRING,
		});
		await queryInterface.changeColumn('Vouchers', 'payload', {
			type: Sequelize.STRING,
		});
		await queryInterface.changeColumn('Notices', 'payload', {
			type: Sequelize.STRING,
		});
	}
};
