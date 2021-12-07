"use strict";
const { v4: uuidv4 } = require("uuid");

const epoch_status_id = uuidv4();
const processed_input_id = uuidv4();
const input_result_id = uuidv4();

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.bulkInsert(
			"EpochStatuses",
			[
				{
					session_id: epoch_status_id,
					epoch_index: 400,
					state: "FINISHED",
					most_recent_machine_hash: `{"data": "Most recent machine hash"}`,
					most_recent_vouchers_epoch_root_hash: `{"data": "Most recent machine hash"}`,
					most_recent_notices_epoch_root_hash: `{"data": "Most recent machine hash"}`,
					pending_input_count: 2,
					taint_status: `{
						"error_code": 200,
						"error_message": "No error occured"
					}`,
					createdAt: new Date(),
					updatedAt: new Date()
				},
				{
					session_id: "42b22569-118b-4232-8cad-8957baecf507",
					epoch_index: 400,
					state: "FINISHED",
					most_recent_machine_hash: `{"data": "Most recent machine hash"}`,
					most_recent_vouchers_epoch_root_hash: `{"data": "Most recent machine hash"}`,
					most_recent_notices_epoch_root_hash: `{"data": "Most recent machine hash"}`,
					pending_input_count: 2,
					taint_status: `{
						"error_code": 200,
						"error_message": "No error occured"
					}`,
					createdAt: new Date(),
					updatedAt: new Date()
				}
			],
			{}
		);

		await queryInterface.bulkInsert("ProcessedInputs", [
			{
				id: processed_input_id,
				input_index: 23,
				most_recent_machine_hash: `{"data": "Most recent machine hash"}`,
				voucher_hashes_in_epoch: `{
					"target_address": 21,
					"log2_target_size": 22,
					"target_hash": {
						"data": "Target hash"
					},
					"log2_root_size": 23,
					"root_hash": {
						"data": "Target hash"
					},
					"sibling_hashes": [
						{
							"data": "Sibling hash 1"
						},
						{
							"data": "Sibling hash 2"
						}
					]
				}`,
				notice_hashes_in_epoch: `{
					"target_address": 21,
					"log2_target_size": 22,
					"target_hash": {
						"data": "Target hash"
					},
					"log2_root_size": 23,
					"root_hash": {
						"data": "Target hash"
					},
					"sibling_hashes": [
						{
							"data": "Sibling hash 1"
						},
						{
							"data": "Sibling hash 2"
						}
					]
				}`,
				skip_reason: "ACCEPTED",
				epoch_status_id,
				createdAt: new Date(),
				updatedAt: new Date()
			}
		]);

		await queryInterface.bulkInsert("InputResults", [
			{
				id: input_result_id,
				voucher_hashes_in_machine: `{
					"target_address": 21,
					"log2_target_size": 22,
					"target_hash": {
						"data": "Target hash"
					},
					"log2_root_size": 23,
					"root_hash": {
						"data": "Target hash"
					},
					"sibling_hashes": [
						{
							"data": "Sibling hash 1"
						},
						{
							"data": "Sibling hash 2"
						}
					]
				}`,
				notice_hashes_in_machine: `{
					"target_address": 21,
					"log2_target_size": 22,
					"target_hash": {
						"data": "Target hash"
					},
					"log2_root_size": 23,
					"root_hash": {
						"data": "Target hash"
					},
					"sibling_hashes": [
						{
							"data": "Sibling hash 1"
						},
						{
							"data": "Sibling hash 2"
						}
					]
				}`,
				processed_input_id,
				createdAt: new Date(),
				updatedAt: new Date()
			}
		]);

		await queryInterface.bulkInsert("Vouchers", [
			{
				id: uuidv4(),
				keccak: "A keccak",
				address: "An Address",
				payload: "A payload",
				keccak_in_voucher_hashes: "A keccak in voucher hash",
				input_result_id,
				createdAt: new Date(),
				updatedAt: new Date()
			}
		]);

		await queryInterface.bulkInsert("Notices", [
			{
				id: uuidv4(),
				keccak: "A keccak",
				payload: "A payload",
				keccak_in_notice_hashes: "A keccak in notice hash",
				input_result_id,
				createdAt: new Date(),
				updatedAt: new Date()
			}
		]);

		await queryInterface.bulkInsert("Reports", [
			{
				id: uuidv4(),
				payload: "A payload",
				processed_input_id,
				createdAt: new Date(),
				updatedAt: new Date()
			}
		]);
	},

	down: async (queryInterface, Sequelize) => {
		await queryInterface.bulkDelete("EpochStatuses", null, {});
	}
};
