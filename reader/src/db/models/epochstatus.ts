"use strict";
import { Model } from "sequelize";

interface ProcessedInput {
	// I don't think this is needed
}
interface CartesiMachineHash {
	data: string;
}

interface EpochStatusAttributes {
	session_id: string;
	epoch_index: string;
	state: string;
	most_recent_machine_hash: string;
	most_recent_vouchers_epoch_root_hash: string;
	most_recent_notices_epoch_root_hash: string;
	pending_input_count: string;
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
		epoch_index!: string;
		state!: string;
		most_recent_machine_hash!: string;
		most_recent_vouchers_epoch_root_hash!: string;
		most_recent_notices_epoch_root_hash!: string;
		pending_input_count!: string;
		taint_status!: {
			error_code: number;
			error_message: string;
		};
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			EpochStatus.hasMany(models.ProcessedInput, {
				foreignKey: "epoch_status_id"
			});
		}
	}
	EpochStatus.init(
		{
			session_id: {
				type: DataTypes.STRING,
				allowNull: false,
				primaryKey: true
			},
			epoch_index: {
				type: DataTypes.STRING,
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
			pending_input_count: {
				type: DataTypes.STRING,
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
