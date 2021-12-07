"use strict";
import { Model, UUID } from "sequelize";

interface NoticeAttributes {
	id: string;
	keccak: string;
	payload: string;
	keccak_in_notice_hashes: string;
	input_result_id: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
  class Notice extends Model<NoticeAttributes> implements NoticeAttributes {
		id!: string;
		keccak!: string;
		payload!: string;
		keccak_in_notice_hashes!: string;
		input_result_id!: string;
		createdAt!: string;
    updatedAt!: string;
    
    static associate(models: any) {
      // define association here
    }
  };
  Notice.init(
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
			payload: {
				type: DataTypes.STRING,
				allowNull: false
			},
			keccak_in_notice_hashes: {
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
			modelName: "Notice"
		}
	);
  return Notice;
};