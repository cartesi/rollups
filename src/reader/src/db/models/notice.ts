"use strict";
import { Model, UUID } from "sequelize";

interface NoticeAttributes {
	id: string;
	session_id: string;
	epoch_index: string;
	input_index: string;
	notice_index: string;
	keccak: string;
	payload: string;
	keccak_in_notice_hashes: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class Notice extends Model<NoticeAttributes> implements NoticeAttributes {
		id!: string;
		session_id!: string;
		epoch_index!: string;
		input_index!: string;
		notice_index!: string;
		keccak!: string;
		payload!: string;
		keccak_in_notice_hashes!: string;
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			// define association here
		}
	}
	Notice.init(
		{
			id: {
				allowNull: false,
				primaryKey: true,
				defaultValue: UUID,
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
			notice_index: {
				type: DataTypes.STRING,
				allowNull: false,
				primaryKey: true
			},
			keccak: {
				type: DataTypes.STRING,
				allowNull: false
			},
			payload: {
				type: DataTypes.STRING,
				allowNull: false
			},
			keccak_in_notice_hashes: {
				type: DataTypes.UUID,
				allowNull: false
			},
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
			modelName: "Notice"
		}
	);
	return Notice;
};
