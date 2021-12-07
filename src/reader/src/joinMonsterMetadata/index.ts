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
	RollupsState: {
		extensions: {
			joinMonster: {
				sqlTable: '"RollupsStates"',
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
						sqlJoin: (RollupsStateTable: any, immutableStateTable: any) =>
							`${RollupsStateTable}.block_hash = ${immutableStateTable}.rollups_hash`
					}
				}
			},
			current_epoch: {
				extensions: {
					joinMonster: {
						sqlTable: '"AccumulatingEpoches"',
						uniqueKey: "id",
						sqlJoin: (RollupsStateTable: any, accumulatingEpochTable: any) =>
							`${RollupsStateTable}.block_hash = ${accumulatingEpochTable}.rollups_hash`
					}
				}
			},
			voucher_state: {
				extensions: {
					joinMonster: {
						sqlTable: '"VoucherStates"',
						uniqueKey: "id",
						sqlJoin: (RollupsStateTable: any, voucherStateTable: any) =>
							`${RollupsStateTable}.block_hash = ${voucherStateTable}.rollups_hash`
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
	Report: {
		extensions: {
			joinMonster: {
				sqlTable: '"Reports"',
				uniqueKey: "id"
			}
		}
	},
	Notice: {
		extensions: {
			joinMonster: {
				sqlTable: '"Notices"',
				sqlPaginate: true,
				orderBy: '"createdAt"',
				uniqueKey: "id"
			}
		}
	},
	Voucher: {
		extensions: {
			joinMonster: {
				sqlTable: '"Vouchers"',
				sqlPaginate: true,
				orderBy: '"createdAt"',
				uniqueKey: "id"
			}
		}
	},
	InputResult: {
		extensions: {
			joinMonster: {
				sqlTable: '"InputResults"',
				sqlPaginate: true,
				orderBy: '"createdAt"',
				uniqueKey: "id"
			}
		},
		fields: {
			vouchers: {
				extensions: {
					joinMonster: {
						sqlTable: '"Vouchers"',
						uniqueKey: "id",
						sqlJoin: (inputResultsTable: any, voucherTable: any) =>
							`${inputResultsTable}."id" = ${voucherTable}."input_result_id"`
					}
				}
			},
			notices: {
				extensions: {
					joinMonster: {
						sqlTable: '"Notices"',
						uniqueKey: "id",
						sqlJoin: (inputResultsTable: any, noticesTable: any) =>
							`${inputResultsTable}."id" = ${noticesTable}."input_result_id"`
					}
				}
			}
		}
	},
	ProcessedInput: {
		extensions: {
			joinMonster: {
				sqlTable: '"ProcessedInputs"',
				sqlPaginate: true,
				orderBy: '"createdAt"',
				uniqueKey: "id"
			}
		},
		fields: {
			reports: {
				extensions: {
					joinMonster: {
						sqlTable: '"Reports"',
						uniqueKey: "id",
						sqlJoin: (processInputsTable: any, reportsTable: any) =>
							`${processInputsTable}."id" = ${reportsTable}."processed_input_id"`
					}
				}
			},
			result: {
				extensions: {
					joinMonster: {
						sqlTable: '"InputResults"',
						uniqueKey: "id",
						sqlJoin: (processInputsTable: any, inputResultsTable: any) =>
							`${processInputsTable}."id" = ${inputResultsTable}."processed_input_id"`
					}
				}
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
		},
		fields: {
			processed_inputs: {
				extensions: {
					joinMonster: {
						sqlTable: '"ProcessedInputs"',
						uniqueKey: "id",
						sqlJoin: (epochStatusTable: any, processedInputTable: any) =>
							`${epochStatusTable}."session_id" = ${processedInputTable}."epoch_status_id"`
					}
				}
			}
		}
	},
	Version: {
		extensions: {
			joinMonster: {
				sqlTable: '"Versions"',
				sqlPaginate: true,
				orderBy: '"createdAt"',
				uniqueKey: "id"
			}
		}
	}
};
