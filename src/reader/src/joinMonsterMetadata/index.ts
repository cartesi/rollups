export default {
	Query: {
		fields: {
			GetSessionStatus: {
				extensions: {
					joinMonster: {
						where: (table: any, args: any) =>
							`${table}."session_id" = '${args.query.session_id}'`
					}
				}
			},
			GetEpochStatus: {
				extensions: {
					joinMonster: {
						where: (table: any, args: any) =>
							`${table}.session_id = '${args.query.session_id}'`
					}
				}
			}
		}
	},
	ImmutableState: {
		extensions: {
			joinMonster: {
				sqlTable: '"ImmutableStates"',
				sqlPaginate: true,
				orderBy: '"createdAt"',
				uniqueKey: "id"
			}
		}
	},
	Input: {
		extensions: {
			joinMonster: {
				sqlTable: '"Inputs"',
				sqlPaginate: true,
				orderBy: '"createdAt"',
				uniqueKey: "id"
			}
		}
	},
	EpochInputState: {
		extensions: {
			joinMonster: {
				sqlTable: '"EpochInputStates"',
				uniqueKey: "id",
				sqlPaginate: true,
				orderBy: '"createdAt"'
			}
		},
		fields: {
			inputs: {
				extensions: {
					joinMonster: {
						sqlTable: '"Inputs"',
						uniqueKey: "id",
						sqlJoin: (epochInputStateTable: any, inputTable: any) =>
							`${epochInputStateTable}.id = ${inputTable}."epoch_input_state_id"`
					}
				}
			}
		}
	},
	FinalizedEpochs: {
		extensions: {
			joinMonster: {
				sqlTable: '"FinalizedEpochs"',
				sqlPaginate: true,
				orderBy: '"createdAt"',
				uniqueKey: "id"
			}
		},
		fields: {
			finalized_epochs: {
				extensions: {
					joinMonster: {
						sqlTable: '"FinalizedEpoches"',
						uniqueKey: "id",
						sqlJoin: (finalizedEpochsTable: any, finalizedEpochTable: any) =>
							`${finalizedEpochsTable}.id = ${finalizedEpochTable}."FinalizedEpochId"`
					}
				}
			}
		}
	},
	FinalizedEpoch: {
		extensions: {
			joinMonster: {
				sqlTable: '"FinalizedEpoches"',
				uniqueKey: "id"
			}
		},
		fields: {
			inputs: {
				extensions: {
					joinMonster: {
						sqlTable: '"EpochInputStates"',
						uniqueKey: "id",
						sqlJoin: (finalizedEpochTable: any, epochInputStateTable: any) =>
							`${finalizedEpochTable}."epochInputStateId" = ${epochInputStateTable}.id`
					}
				}
			}
		}
	},
	AccumulatingEpoch: {
		extensions: {
			joinMonster: {
				sqlTable: '"AccumulatingEpoches"',
				uniqueKey: "id",
				sqlPaginate: true,
				orderBy: '"createdAt"'
			}
		},
		fields: {
			inputs: {
				extensions: {
					joinMonster: {
						sqlTable: '"EpochInputStates"',
						uniqueKey: "id",
						sqlJoin: (accumulatingEpochTable: any, epochInputStateTable: any) =>
							`${accumulatingEpochTable}."epochInputStateId" = ${epochInputStateTable}."id"`
					}
				}
			}
		}
	},
	VoucherState: {
		extensions: {
			joinMonster: {
				sqlTable: '"VoucherStates"',
				uniqueKey: "id",
				sqlPaginate: true,
				orderBy: '"createdAt"'
			}
		}
	},
	DescartesV2State: {
		extensions: {
			joinMonster: {
				sqlTable: '"DescartesV2States"',
				uniqueKey: "block_hash",
				orderBy: '"createdAt"'
			}
		},
		fields: {
			constants: {
				extensions: {
					joinMonster: {
						sqlTable: '"ImmutableStates"',
						uniqueKey: "id",
						sqlJoin: (descartesV2StateTable: any, immutableStateTable: any) =>
							`${descartesV2StateTable}.block_hash = ${immutableStateTable}.descartes_hash`
					}
				}
			},
			current_epoch: {
				extensions: {
					joinMonster: {
						sqlTable: '"AccumulatingEpoches"',
						uniqueKey: "id",
						sqlJoin: (
							descartesV2StateTable: any,
							accumulatingEpochTable: any
						) =>
							`${descartesV2StateTable}.block_hash = ${accumulatingEpochTable}.descartes_hash`
					}
				}
			},
			voucher_state: {
				extensions: {
					joinMonster: {
						sqlTable: '"VoucherStates"',
						uniqueKey: "id",
						sqlJoin: (descartesV2StateTable: any, voucherStateTable: any) =>
							`${descartesV2StateTable}.block_hash = ${voucherStateTable}.descartes_hash`
					}
				}
			}
		}
	},
	GetSessionStatusResponse: {
		extensions: {
			joinMonster: {
				sqlTable: '"SessionStatuses"',
				sqlPaginate: true,
				orderBy: '"createdAt"',
				uniqueKey: "session_id"
			}
		}
	},
	GetEpochStatusResponse: {
		extensions: {
			joinMonster: {
				sqlTable: '"EpochStatuses"',
				sqlPaginate: true,
				orderBy: '"createdAt"',
				uniqueKey: "session_id"
			}
		}
	}
};
