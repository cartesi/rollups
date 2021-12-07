"use strict";
import { Model, UUIDV4 } from "sequelize";

interface CartesiMachineHash {
	data: string;
}

interface ProcessedInputAttributes {
	id: string;
	input_index: number;
	most_recent_machine_hash: CartesiMachineHash;
	voucher_hashes_in_epoch: CartesiMachineHash;
	notice_hashes_in_epoch: CartesiMachineHash;
	skip_reason: string;
	epoch_status_id: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class ProcessedInput extends Model<ProcessedInputAttributes>
		implements ProcessedInputAttributes {
		id!: string;
		input_index!: number;
		most_recent_machine_hash!: CartesiMachineHash;
		voucher_hashes_in_epoch!: CartesiMachineHash;
		notice_hashes_in_epoch!: CartesiMachineHash;
		most_recent_notices_epoch_root_hash!: CartesiMachineHash;
		skip_reason!: string;
		epoch_status_id!: string;
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			ProcessedInput.hasMany(models.Report, {
				foreignKey: "processed_input_id"
			});
			ProcessedInput.hasOne(models.InputResult, {
				foreignKey: "processed_input_id"
			});
		}
	}
	ProcessedInput.init(
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: UUIDV4,
				allowNull: false,
				primaryKey: true
			},
			input_index: {
				type: DataTypes.INTEGER,
				allowNull: false
			},
			most_recent_machine_hash: {
				type: DataTypes.JSON,
				allowNull: false
			},
			voucher_hashes_in_epoch: {
				type: DataTypes.JSON,
				allowNull: false
			},
			notice_hashes_in_epoch: {
				type: DataTypes.JSON,
				allowNull: false
			},
			skip_reason: {
				type: DataTypes.STRING,
				allowNull: false
			},
			epoch_status_id: DataTypes.UUID,
			createdAt: {
				type: DataTypes.DATE
			},
			updatedAt: {
				type: DataTypes.DATE
			}
		},
		{
			sequelize,
			modelName: "ProcessedInput"
		}
	);
	return ProcessedInput;
};
