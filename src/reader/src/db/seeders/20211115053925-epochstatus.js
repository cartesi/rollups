"use strict";
const { v4: uuidv4 } = require("uuid");

module.exports = {
	up: async (queryInterface, Sequelize) => {
		await queryInterface.bulkInsert(
			"EpochStatuses",
			[
				{
					session_id: uuidv4(),
					epoch_index: 400,
					state: "FINISHED",
					most_recent_machine_hash: "A recent machine hash",
					most_recent_vouchers_epoch_root_hash: "A recent voucher hash",
					most_recent_notices_epoch_root_hash: "A recent notice hash",
					processed_inputs: `[
						{
							"input_index": 1,
							"most_recent_machine_hash": "Another recent machine hash",
							"voucher_hashes_in_epoch": "Vouvher hashes",
							"notice_hashes_in_epoch": "Notices has",
							"reports": [
								{
									"payload": "A report payload"
								},
								{
									"payload": "Another report payload"
								}
							],
							"result": {
								"voucher_hashes_in_machine": "A vouvher machine hash",
								"vouchers": [
									{
										"keccak": "A Keccak",
										"address": "A voucher address",
										"payload": "A voucher payload",
										"keccak_in_voucher_hashes": "Voucher Keccak hashes"
									},
									{
										"keccak": "Another Keccak",
										"address": "Another voucher address",
										"payload": "Another voucher payload",
										"keccak_in_voucher_hashes": "Voucher Keccak hashes"
									}
								],
								"notice_hashes_in_machine": "A notice hash in machine",
								"notices": [
									{
										"keccak": "A notice Keccak",
										"payload": "A notice payload",
										"keccak_in_notice_hashes": "A notice hash"
									},
									{
										"keccak": "Another notice Keccak",
										"payload": "Another notice payload",
										"keccak_in_notice_hashes": "Another notice hash"
									}
								]
							},
							"skip_reason": "ACCEPTED"
						}
					]`,
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
					most_recent_machine_hash: "A recent machine hash",
					most_recent_vouchers_epoch_root_hash: "A recent voucher hash",
					most_recent_notices_epoch_root_hash: "A recent notice hash",
					processed_inputs: `[
						{
							"input_index": 1,
							"most_recent_machine_hash": "Another recent machine hash",
							"voucher_hashes_in_epoch": "Vouvher hashes",
							"notice_hashes_in_epoch": "Notices has",
							"reports": [
								{
									"payload": "A report payload"
								},
								{
									"payload": "Another report payload"
								}
							],
							"result": {
								"voucher_hashes_in_machine": "A vouvher machine hash",
								"vouchers": [
									{
										"keccak": "A Keccak",
										"address": "A voucher address",
										"payload": "A voucher payload",
										"keccak_in_voucher_hashes": "Voucher Keccak hashes"
									},
									{
										"keccak": "Another Keccak",
										"address": "Another voucher address",
										"payload": "Another voucher payload",
										"keccak_in_voucher_hashes": "Voucher Keccak hashes"
									}
								],
								"notice_hashes_in_machine": "A notice hash in machine",
								"notices": [
									{
										"keccak": "A notice Keccak",
										"payload": "A notice payload",
										"keccak_in_notice_hashes": "A notice hash"
									},
									{
										"keccak": "Another notice Keccak",
										"payload": "Another notice payload",
										"keccak_in_notice_hashes": "Another notice hash"
									}
								]
							},
							"skip_reason": "ACCEPTED"
						}
					]`,
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
	},

	down: async (queryInterface, Sequelize) => {
		await queryInterface.bulkDelete("EpochStatuses", null, {});
	}
};
