"use strict";
const { v4: uuidv4 } = require("uuid");

const epoch_status_id = "42b22569-118b-4232-8cad-8957baecf507";
const processed_input_id = uuidv4();
const input_result_id = uuidv4();

const MerkleTreeProofId1 = uuidv4();
const MerkleTreeProofId2 = uuidv4();
const MerkleTreeProofId3 = uuidv4();
const MerkleTreeProofId4 = uuidv4();
const MerkleTreeProofId5 = uuidv4();
const MerkleTreeProofId6 = uuidv4();


const session_id = "e9f1061b-3319-4e0f-86ab-4c12177fa71b";
const epoch_index = "5f6278bf-9272-462e-b435-80b443a10c25";
const input_index = "acf3bd3d-1d71-4adb-8b4a-e895055de962";

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
				},
				{
					id: MerkleTreeProofId2,
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
				},
				{
					id: MerkleTreeProofId3,
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
				},
				{
					id: MerkleTreeProofId4,
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
				},
				{
					id: MerkleTreeProofId5,
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
				},
				{
					id: MerkleTreeProofId6,
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
			"EpochStatuses",
			[
				{
					session_id,
					epoch_index,
					state: "FINISHED",
					most_recent_machine_hash: "A machine hash",
					most_recent_vouchers_epoch_root_hash: "A recent voucher hash",
					most_recent_notices_epoch_root_hash: "A recent notice hash",
					pending_input_count: "2",
					taint_status: `{
						"error_code": 200,
						"error_message": "Successful"
					}`,
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					session_id: uuidv4(),
					epoch_index: 400,
					state: "FINISHED",
					most_recent_machine_hash: "A machine hash",
					most_recent_vouchers_epoch_root_hash: "A recent voucher hash",
					most_recent_notices_epoch_root_hash: "A recent notice hash",
					pending_input_count: "2",
					taint_status: `{
						"error_code": 200,
						"error_message": "Successful"
					}`,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert("ProcessedInputs", [
			{
				session_id,
				epoch_index,
				input_index,
				most_recent_machine_hash: "A recent Hash",
				voucher_hashes_in_epoch: MerkleTreeProofId1,
				notice_hashes_in_epoch: MerkleTreeProofId2,
				reports: `[
					{
						"payload": "A payload"
					},
					{
						"payload": "A payload"
					}
				]`,
				skip_reason: "ACCEPTED",
				epoch_status_id,
				createdAt: new Date(),
				updatedAt: new Date()
			}
		]);

		await queryInterface.bulkInsert("InputResults", [
			{
				session_id,
				epoch_index,
				input_index,
				voucher_hashes_in_machine: MerkleTreeProofId3,
				notice_hashes_in_machine: MerkleTreeProofId4,
				createdAt: new Date(),
				updatedAt: new Date()
			}
		]);

		await queryInterface.bulkInsert("Vouchers", [
			{
				session_id,
				epoch_index,
				input_index,
				voucher_index: "A voucher index",
				keccak: "A keccak",
				Address: "An address",
				payload: "A payload",
				keccak_in_voucher_hashes: MerkleTreeProofId5,
				createdAt: new Date(),
				updatedAt: new Date()
			}
		]);

		await queryInterface.bulkInsert("Notices", [
			{
				session_id,
				epoch_index,
				input_index,
				notice_index: "A voucher index",
				keccak: "A keccak",
				payload: "A payload",
				keccak_in_notice_hashes: MerkleTreeProofId5,
				createdAt: new Date(),
				updatedAt: new Date()
			}
		]);
	},

	down: async (queryInterface, Sequelize) => {
		await queryInterface.bulkDelete("EpochStatuses", null, {});
	}
};
