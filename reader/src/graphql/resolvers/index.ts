require("dotenv").config();
import { IResolvers } from "graphql-tools";
import { UserInputError } from "apollo-server-express";
import { v4 as uuidv4 } from "uuid";
import {
	Version,
	FinalizedEpoch,
	EpochInputState,
	FinalizedEpochs,
	ImmutableState,
	AccumulatingEpoch,
	VoucherState,
	PhaseState,
	ImmutableStateInput,
	FinalizedEpochsInput,
	AccumulatingEpochInput,
	VoucherStateInput,
	RollupsInput,
	RollupsState,
	ProcessedInput,
	Voucher,
	Notice,
	GetStatusResponse,
	GetSessionStatusResponse,
	GetEpochStatusResponse,
	Metrics
} from "../generated-typeDefs";
import joinMonster from "join-monster";
import db from "../../db/models";

// Metrics
import { answeredQueryCounter, PromClient } from "../../utils/metrics";

const getFinalizedEpochsDetails = async () => {
	try {
		const latestFinalizedEpochs = await db.FinalizedEpochs.findOne({
			order: [["createdAt", "DESC"]]
		});

		if (latestFinalizedEpochs) {
			const dapp_contract_address =
				latestFinalizedEpochs?.dapp_contract_address;
			let block_number: string | null = null;
			let block_hash: string | null = null;
			let number_of_processed_inputs: number | null = null;

			const latestFinalizedEpoch = await db.FinalizedEpoch.findOne({
				where: { FinalizedEpochId: latestFinalizedEpochs?.id },
				order: [["createdAt", "DESC"]]
			});

			if (latestFinalizedEpoch) {
				block_number = latestFinalizedEpoch?.finalized_block_number;
				block_hash = latestFinalizedEpoch?.finalized_block_hash;

				const epochInputStates = await db.EpochInputState.findAll({
					where: { id: latestFinalizedEpoch?.epochInputStateId },
					order: [["createdAt", "DESC"]]
				});

				number_of_processed_inputs = epochInputStates.length;
			}

			return {
				block_number,
				block_hash,
				number_of_processed_inputs,
				dapp_contract_address
			};
		} else {
			return {
				block_number: null,
				block_hash: null,
				number_of_processed_inputs: null,
				dapp_contract_address: null
			};
		}
	} catch (error: any) {
		throw new Error(
			error || "There was an error while getting Finalized Epoch Details"
		);
	}
};

export const UserResolvers: IResolvers = {
	Query: {
		async constants(_: void, args, {}, info): Promise<ImmutableState> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();

					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw Error(error);
			}
		},

		initial_epoch(): string {
			answeredQueryCounter.inc();
			return "234567890";
		},

		async finalized_epochs(_: void, args, {}, info): Promise<FinalizedEpoch> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();
					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw Error(error);
			}
		},

		current_phase(): PhaseState {
			answeredQueryCounter.inc();
			return PhaseState.AwaitingConsensusAfterConflict;
		},

		async voucher_state(_: void, args, {}, info): Promise<VoucherState> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();

					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw Error(error);
			}
		},

		async current_epoch(_: void, args, {}, info): Promise<AccumulatingEpoch> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();

					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw Error(error);
			}
		},

		async RollupsState(_: void, args, {}, info): Promise<ImmutableState> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();

					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw Error(error);
			}
		},

		async GetVersion(_: void, args, {}, info): Promise<Version> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();

					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw new Error(error);
			}
		},

		GetStatus(): GetStatusResponse {
			answeredQueryCounter.inc();
			return {
				session_id: [uuidv4(), uuidv4(), uuidv4()]
			};
		},

		async GetSessionStatus(
			_: void,
			args,
			{},
			info
		): Promise<GetSessionStatusResponse> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();

					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw Error(error);
			}
		},

		async GetEpochStatus(
			_: void,
			args,
			{},
			info
		): Promise<GetEpochStatusResponse> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();
					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw Error(error);
			}
		},

		async GetProcessedInput(
			_: void,
			args,
			{},
			info
		): Promise<ProcessedInput[]> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();

					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw Error(error);
			}
		},

		async GetVoucher(_: void, args, {}, info): Promise<Voucher[]> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();

					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw Error(error);
			}
		},

		async GetNotice(_: void, args, {}, info): Promise<Notice[]> {
			try {
				return joinMonster(info, args, (sql: any) => {
					console.log(sql);

					answeredQueryCounter.inc();

					return db.sequelize.query(sql, {
						type: db.sequelize.QueryTypes.SELECT
					});
				});
			} catch (error: any) {
				throw Error(error);
			}
		},

		async getMetrics(): Promise<Metrics> {
			try {
				const collectDefaultMetrics = PromClient.collectDefaultMetrics;
				collectDefaultMetrics();
				const metrics = await PromClient.register.metrics();

				const {
					block_number,
					block_hash,
					number_of_processed_inputs,
					dapp_contract_address
				} = await getFinalizedEpochsDetails();

				return {
					block_hash,
					block_number,
					number_of_processed_inputs,
					dapp_contract_address,
					prometheus_metrics: metrics
				}
			} catch (error: any) {
				throw Error(error);
			}
		}
	},

	Mutation: {
		async constants(
			_: void,
			{ input }: { input: ImmutableStateInput[] },
			{}
		): Promise<ImmutableState[]> {
			try {
				const data = [];

				for (const item of input) {
					const newData = await db.ImmutableState.create({
						id: uuidv4(),
						challenge_period: item?.challenge_period,
						contract_creation_timestamp: new Date(),
						dapp_contract_address: item?.dapp_contract_address,
						input_duration: item?.input_duration,
						createdAt: new Date(),
						updatedAt: new Date()
					});

					data.push(newData);
				}

				answeredQueryCounter.inc();

				return data;
			} catch (error: any) {
				throw Error(error);
			}
		},

		initial_epoch(_: void, { input }: { input: string }): string {
			answeredQueryCounter.inc();
			return input;
		},

		async finalized_epochs(
			_void,
			{ input }: { input: FinalizedEpochsInput[] }
		): Promise<FinalizedEpochs[]> {
			const data = [];

			try {
				for (const item of input) {
					const parentId = uuidv4();
					const finalized_epochs = [];

					const finalizedEpochs = await db.FinalizedEpochs.create({
						id: parentId,
						initial_epoch: item?.initial_epoch,
						dapp_contract_address: item?.dapp_contract_address,
						createdAt: new Date(),
						updatedAt: new Date()
					});

					for (const finalizedEpoch of item?.finalized_epochs) {
						const epochInputStateId = uuidv4();
						let epochInputState: EpochInputState;
						const inputs = [];

						if (finalizedEpoch?.inputs?.inputs) {
							for (const input of finalizedEpoch?.inputs?.inputs) {
								const newInput = await db.Input.create({
									id: uuidv4(),
									sender: input?.sender,
									timestamp: input?.timestamp,
									payload: input?.payload,
									epoch_input_state_id: epochInputStateId
								});

								inputs.push(newInput);
							}
						}

						epochInputState = await db.EpochInputState.create({
							id: epochInputStateId,
							epoch_number: finalizedEpoch?.inputs?.epoch_number,
							createdAt: new Date(),
							updatedAt: new Date()
						});

						const newFinalizedEpoch = await db.FinalizedEpoch.create({
							id: uuidv4(),
							epoch_number: finalizedEpoch?.epoch_number,
							hash: finalizedEpoch?.hash,
							inputs: finalizedEpoch?.inputs,
							finalized_block_hash: finalizedEpoch?.finalized_block_hash,
							finalized_block_number: finalizedEpoch?.finalized_block_number,
							FinalizedEpochId: parentId,
							epochInputStateId,
							createdAt: new Date(),
							updatedAt: new Date()
						});

						newFinalizedEpoch.inputs = {
							id: epochInputState?.id,
							epoch_number: epochInputState?.epoch_number,
							inputs
						};
						finalized_epochs.push(newFinalizedEpoch);
					}

					finalizedEpochs.finalized_epochs = finalized_epochs;
					data.push(finalizedEpochs);
				}

				answeredQueryCounter.inc();
				return data;
			} catch (error: any) {
				throw new Error(error);
			}
		},

		async current_epoch(
			_: void,
			{
				input: { dapp_contract_address, epoch_number, inputs }
			}: { input: AccumulatingEpochInput }
		): Promise<AccumulatingEpoch> {
			try {
				const epochInputStateId = uuidv4();
				const inputsArray = [];

				if (inputs?.inputs) {
					for (const input of inputs?.inputs) {
						const newInput = await db.Input.create({
							id: uuidv4(),
							sender: input?.sender,
							timestamp: input?.timestamp,
							payload: input?.payload,
							epoch_input_state_id: epochInputStateId
						});

						inputsArray.push(newInput);
					}
				}

				const epochInputState = await db.EpochInputState.create({
					id: epochInputStateId,
					epoch_number: inputs?.epoch_number,
					createdAt: new Date(),
					updatedAt: new Date()
				});

				const accumulatingEpoch = await db.AccumulatingEpoch.create({
					id: uuidv4(),
					dapp_contract_address,
					epoch_number,
					epochInputStateId,
					createdAt: new Date(),
					updatedAt: new Date()
				});

				accumulatingEpoch.inputs = {
					id: epochInputState?.id,
					epoch_number: epochInputState?.epoch_number,
					inputs: inputsArray
				};

				answeredQueryCounter.inc();

				return accumulatingEpoch;
			} catch (error: any) {
				throw new Error(error);
			}
		},

		current_phase(_: void, { input }: { input: PhaseState }): PhaseState {
			answeredQueryCounter.inc();
			return input;
		},

		async voucher_state(
			_: void,
			{ input: { voucher_address, vouchers } }: { input: VoucherStateInput }
		): Promise<VoucherState> {
			try {
				const VoucherState = await db.VoucherState.create({
					id: uuidv4(),
					voucher_address,
					vouchers,
					createdAt: new Date(),
					updatedAt: new Date()
				});

				answeredQueryCounter.inc();
				return VoucherState;
			} catch (error: any) {
				throw new Error(error);
			}
		},

		async RollupsState(
			_: void,
			{ input }: { input: RollupsInput }
		): Promise<RollupsState> {
			try {
				const existingRollups = await db.RollupsState.findOne({
					where: { block_hash: input.block_hash }
				});

				if (existingRollups) {
					throw new UserInputError(
						"A RollupsState with that block_hash already exists",
						{
							message: "A RollupsState with that block_hash already exists"
						}
					);
				}

				const rollups_hash = input.block_hash;
				const rollups_id = uuidv4();
				const immutableStateIds: string[] = [];
				const finalizedEpochsIds: string[] = [];
				const accumulatingEpochId = uuidv4();
				const VoucherStateId = uuidv4();

				let constants: ImmutableState;
				let current_epoch: AccumulatingEpoch;
				let voucher_state: VoucherState;

				// Create Immutable States
				try {
					constants = await db.ImmutableState.create({
						id: uuidv4(),
						challenge_period: input?.constants?.challenge_period,
						contract_creation_timestamp: new Date(),
						dapp_contract_address: input?.constants?.dapp_contract_address,
						input_duration: input?.constants?.input_duration,
						rollups_hash,
						createdAt: new Date(),
						updatedAt: new Date()
					});
				} catch (error: any) {
					throw Error(error);
				}

				// Create Accumulated Epoch
				try {
					const epochInputStateId = uuidv4();
					const inputsArray = [];

					if (input?.current_epoch?.inputs?.inputs) {
						for (const item of input?.current_epoch?.inputs?.inputs) {
							const newInput = await db.Input.create({
								id: uuidv4(),
								sender: item?.sender,
								timestamp: item?.timestamp,
								payload: item?.payload,
								epoch_input_state_id: epochInputStateId
							});

							inputsArray.push(newInput);
						}
					}

					const epochInputState = await db.EpochInputState.create({
						id: epochInputStateId,
						epoch_number: input?.current_epoch?.inputs?.epoch_number,
						createdAt: new Date(),
						updatedAt: new Date()
					});

					const accumulatingEpoch = await db.AccumulatingEpoch.create({
						id: accumulatingEpochId,
						dapp_contract_address: input?.current_epoch?.dapp_contract_address,
						epoch_number: input?.current_epoch?.epoch_number,
						epochInputStateId,
						rollups_hash: rollups_hash,
						createdAt: new Date(),
						updatedAt: new Date()
					});

					accumulatingEpoch.inputs = {
						id: epochInputState?.id,
						epoch_number: epochInputState?.epoch_number,
						inputs: inputsArray
					};
					current_epoch = accumulatingEpoch;
				} catch (error: any) {
					throw new Error(error);
				}

				// Create Voucher State
				try {
					const VoucherState = await db.VoucherState.create({
						id: VoucherStateId,
						voucher_address: input?.voucher_state?.voucher_address,
						vouchers: input?.voucher_state?.vouchers,
						rollups_hash,
						createdAt: new Date(),
						updatedAt: new Date()
					});

					voucher_state = VoucherState;
				} catch (error: any) {
					throw new Error(error);
				}

				await db.RollupsState.create({
					id: rollups_id,
					block_hash: rollups_hash,
					constants: immutableStateIds,
					initial_epoch: input?.initial_epoch,
					finalized_epochs: finalizedEpochsIds,
					current_epoch: accumulatingEpochId,
					current_phase: input?.current_phase,
					voucher_state: VoucherStateId,
					createdAt: new Date(),
					updatedAt: new Date()
				});

				answeredQueryCounter.inc();

				return {
					id: rollups_id,
					block_hash: rollups_hash,
					constants,
					initial_epoch: input?.initial_epoch,
					current_epoch,
					current_phase: input?.current_phase,
					voucher_state
				};
			} catch (error: any) {
				throw new Error(error);
			}
		}
	}
};
