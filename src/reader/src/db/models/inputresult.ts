"use strict";
import { Model, UUIDV4 } from "sequelize";

interface Hash {
	data: string;
}

interface CartesiMachineMerkleTreeProof {
	target_address: number;
	log2_target_size: number;
	target_hash: Hash;
	log2_root_size: number;
	root_hash: Hash;
	sibling_hashes: [Hash];
}

interface InputResultAttribute {
	id: string;
	session_id: string;
	epoch_index: string;
	input_index: string;
	voucher_hashes_in_machine: string;
	notice_hashes_in_machine: string;
	processed_input_id: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class InputResult extends Model<InputResultAttribute>
		implements InputResultAttribute {
		id!: string;
		session_id!: string;
		epoch_index!: string;
		input_index!: string;
		voucher_hashes_in_machine!: string;
		notice_hashes_in_machine!: string;
		processed_input_id!: string;
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			InputResult.hasMany(models.Voucher, {
				foreignKey: "input_result_id"
			});
			InputResult.hasMany(models.Notice, {
				foreignKey: "input_result_id"
			});
		}
	}
	InputResult.init(
		{
			id: {
				allowNull: false,
				primaryKey: true,
				defaultValue: UUIDV4,
				type: DataTypes.UUID
			},
			session_id: {
				type: DataTypes.STRING,
				allowNull: false,
				primaryKey: true
			},
			epoch_index: {
				type: DataTypes.STRING,
				allowNull: false,
				primaryKey: true
			},
			input_index: {
				type: DataTypes.STRING,
				allowNull: false,
				primaryKey: true
			},
			voucher_hashes_in_machine: {
				type: DataTypes.UUID,
				allowNull: false
			},
			notice_hashes_in_machine: {
				type: DataTypes.UUID,
				allowNull: false
			},
			processed_input_id: DataTypes.UUID,
			createdAt: {
				allowNull: false,
				type: DataTypes.DATE
			},
			updatedAt: {
				allowNull: false,
				type: DataTypes.DATE
			}
		},
		{
			sequelize,
			modelName: "InputResult"
		}
	);
	return InputResult;
};
