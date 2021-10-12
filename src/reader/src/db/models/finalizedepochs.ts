"use strict";
import { Model, UUIDV4 } from "sequelize";

interface FinalizedEpochsAttributes {
	id: string;
	initial_epoch: string;
	descartesv2_contract_address: string;
	input_contract_address: string;
	descartes_hash: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class FinalizedEpochs extends Model<FinalizedEpochsAttributes>
		implements FinalizedEpochsAttributes {
		id!: string;
		initial_epoch!: string;
		descartesv2_contract_address!: string;
		input_contract_address!: string;
		descartes_hash!: string;
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			FinalizedEpochs.hasMany(models.FinalizedEpoch, {
				as: "finalized_epochs"
			});
		}
	}
	FinalizedEpochs.init(
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: UUIDV4,
				allowNull: false,
				primaryKey: true
			},
			initial_epoch: {
				type: DataTypes.STRING,
				allowNull: false
			},
			descartesv2_contract_address: {
				type: DataTypes.STRING,
				allowNull: false
			},
			input_contract_address: {
				type: DataTypes.STRING,
				allowNull: false
			},
			descartes_hash: DataTypes.UUID,
			createdAt: {
				type: DataTypes.DATE,
				allowNull: false
			},
			updatedAt: {
				type: DataTypes.DATE,
				allowNull: false
			}
		},
		{
			sequelize,
			modelName: "FinalizedEpochs"
		}
	);
	return FinalizedEpochs;
};
