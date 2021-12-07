"use strict";
import { Model, UUID } from "sequelize";

interface VoucherAttributes {
	id: string;
	keccak: string;
	address: string;
	payload: string;
	keccak_in_voucher_hashes: string;
	input_result_id: string;
	createdAt: string;
	updatedAt: string;
}

export default (sequelize: any, DataTypes: any) => {
	class Voucher extends Model<VoucherAttributes> implements VoucherAttributes {
		id!: string;
		keccak!: string;
		address!: string;
		payload!: string;
		keccak_in_voucher_hashes!: string;
		input_result_id!: string;
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			// define association here
		}
	}
	Voucher.init(
		{
			id: {
				allowNull: false,
				primaryKey: true,
				defaultValue: UUID,
				type: DataTypes.UUID
			},
			keccak: {
				type: DataTypes.STRING,
				allowNull: false
			},
			address: {
				type: DataTypes.STRING,
				allowNull: false
			},
			payload: {
				type: DataTypes.STRING,
				allowNull: false
			},
			keccak_in_voucher_hashes: {
				type: DataTypes.STRING,
				allowNull: false
			},
			input_result_id: DataTypes.UUID,
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
			modelName: "Voucher"
		}
	);
	return Voucher;
};
