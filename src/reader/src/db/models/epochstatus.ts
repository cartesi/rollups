"use strict";
import { Model, UUIDV4 } from "sequelize";

interface ProcessedInput {
	// I don't think this is needed
}

interface EpochStatusAttributes {
	session_id: string;
	epoch_index: number;
	state: string;
	most_recent_machine_hash: string;
	most_recent_vouchers_epoch_root_hash: string;
	most_recent_notices_epoch_root_hash: string;
	processed_inputs: ProcessedInput[];
	pending_input_count: number;
	taint_status: {
		error_code: number;
		error_message: string;
	};
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class EpochStatus extends Model<EpochStatusAttributes>
		implements EpochStatusAttributes {
		session_id!: string;
		epoch_index!: number;
		state!: string;
		most_recent_machine_hash!: string;
		most_recent_vouchers_epoch_root_hash!: string;
		most_recent_notices_epoch_root_hash!: string;
		processed_inputs!: ProcessedInput[];
		pending_input_count!: number;
		taint_status!: {
			error_code: number;
			error_message: string;
		};
		createdAt!: string;
		updatedAt!: string;
	}
	EpochStatus.init(
		{
			session_id: {
				type: DataTypes.UUID,
				defaultValue: UUIDV4,
				allowNull: false,
				primaryKey: true
			},
			epoch_index: {
				type: DataTypes.INTEGER,
				allowNull: false
			},
			state: {
				type: DataTypes.STRING,
				allowNull: false
			},
			most_recent_machine_hash: {
				type: DataTypes.STRING,
				allowNull: false
			},
			most_recent_vouchers_epoch_root_hash: {
				type: DataTypes.STRING,
				allowNull: false
			},
			most_recent_notices_epoch_root_hash: {
				type: DataTypes.STRING,
				allowNull: false
			},
			processed_inputs: {
				type: DataTypes.ARRAY(DataTypes.JSON),
				allowNull: false
			},
			pending_input_count: {
				type: DataTypes.INTEGER,
				allowNull: false
			},
			taint_status: {
				type: DataTypes.JSON,
				allowNull: false
			},
			createdAt: {
				type: DataTypes.DATE
			},
			updatedAt: {
				type: DataTypes.DATE
			}
		},
		{
			sequelize,
			modelName: "EpochStatus"
		}
	);
	return EpochStatus;
};
