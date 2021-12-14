"use strict";
const { v4: uuidv4 } = require("uuid");

const voucherId = "33f88ffc-3818-4e3e-9af2-c7d8c790638f";


const session_id = "e9f1061b-3319-4e0f-86ab-4c12177fa71a";
const epoch_index = "5f6278bf-9272-462e-b435-80b443a10c24";
const input_index = "acf3bd3d-1d71-4adb-8b4a-e895055de961";

const MerkleTreeProofId1 = uuidv4();

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.bulkInsert(
			"MerkleTreeProofs",
			[
				{
					id: MerkleTreeProofId1,
					target_address: "An address",
					log2_target_size: "A size",
					target_hash: "A target hash",
					log2_root_size: "Another size",
					root_hash: "A root hash",
					sibling_hashes: `[
						{
							"data": "Some data"
						}
					]`,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert(
			"Vouchers",
			[
				{
					id: uuidv4(),
					session_id,
					epoch_index,
					input_index,
					voucher_index: "A voucher index",
					keccak: "A keccak",
					Address: "An address",
					payload: "A payload",
					keccak_in_voucher_hashes: MerkleTreeProofId1,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);
	},

	down: async (queryInterface, Sequelize) => {
		/**
		 * Add commands to revert seed here.
		 *
		 * Example:
		 * await queryInterface.bulkDelete('People', null, {});
		 */
	}
};
