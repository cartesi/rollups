"use strict";
module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.createTable("MerkleTreeProofs", {
			id: {
				allowNull: false,
				primaryKey: true,
				type: Sequelize.UUID
			},
			target_address: {
				type: Sequelize.STRING,
				allowNull: false
			},
			log2_target_size: {
				type: Sequelize.STRING,
				allowNull: false
			},
			target_hash: {
				type: Sequelize.STRING,
				allowNull: false
			},
			log2_root_size: {
				type: Sequelize.STRING,
				allowNull: false
			},
			root_hash: {
				type: Sequelize.STRING,
				allowNull: false
			},
			sibling_hashes: {
				type: Sequelize.JSON,
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
		await queryInterface.dropTable("MerkleTreeProofs");
	}
};
