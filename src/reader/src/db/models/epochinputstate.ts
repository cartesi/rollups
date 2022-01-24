"use strict";
import { Model, UUIDV4 } from "sequelize";

export interface EpochInputStateAttributes {
	id: string;
	epoch_number: number;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class EpochInputState extends Model<EpochInputStateAttributes>
		implements EpochInputStateAttributes {
		id!: string;
		epoch_number!: number;
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			EpochInputState.hasOne(models.FinalizedEpoch, {
				foreignKey: "epochInputStateId"
			});
			EpochInputState.hasOne(models.AccumulatingEpoch, {
				foreignKey: "epochInputStateId"
			});
			EpochInputState.hasMany(models.Input, {
				foreignKey: "epoch_input_state_id"
			});
		}
	}
	EpochInputState.init(
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: UUIDV4,
				allowNull: false,
				primaryKey: true
			},
			epoch_number: {
				type: DataTypes.INTEGER,
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
			modelName: "EpochInputState"
		}
	);
	return EpochInputState;
};
