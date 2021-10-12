"use strict";
import { Model, UUIDV4 } from "sequelize";

export interface FinalizedEpochAttributes {
	id: string;
	epoch_number: string;
	hash: string;
	finalized_block_hash: string;
	finalized_block_number: number;
	epochInputStateId: string;
	FinalizedEpochId: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class FinalizedEpoch extends Model<FinalizedEpochAttributes>
		implements FinalizedEpochAttributes {
		id!: string;
		epoch_number!: string;
		hash!: string;
		finalized_block_hash!: string;
		finalized_block_number!: number;
		epochInputStateId!: string;
		FinalizedEpochId!: string;
		createdAt!: string;
		updatedAt!: string;
	}
	FinalizedEpoch.init(
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: UUIDV4,
				allowNull: false,
				primaryKey: true
			},
			epoch_number: {
				type: DataTypes.STRING,
				allowNull: false
			},
			hash: {
				type: DataTypes.STRING,
				allowNull: false
			},
			finalized_block_hash: {
				type: DataTypes.STRING,
				allowNull: false
			},
			finalized_block_number: {
				type: DataTypes.STRING,
				allowNull: false
			},
			epochInputStateId: {
				type: DataTypes.UUID,
				allowNull: false
			},
			FinalizedEpochId: {
				type: DataTypes.UUID,
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
			modelName: "FinalizedEpoch"
		}
	);
	return FinalizedEpoch;
};
