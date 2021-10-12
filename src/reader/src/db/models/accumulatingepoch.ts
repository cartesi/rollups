"use strict";
import { Model, UUIDV4 } from "sequelize";

interface AccumulatingEpochAttributes {
	id: string;
	epoch_number: string;
	descartesv2_contract_address: string;
	input_contract_address: string;
	epochInputStateId: string;
	descartes_hash: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class AccumulatingEpoch extends Model<AccumulatingEpochAttributes>
		implements AccumulatingEpochAttributes {
		id!: string;
		epoch_number!: string;
		descartesv2_contract_address!: string;
		input_contract_address!: string;
		epochInputStateId!: string;
		descartes_hash!: string;
		createdAt!: string;
		updatedAt!: string;
	}
	AccumulatingEpoch.init(
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
			descartesv2_contract_address: {
				type: DataTypes.STRING,
				allowNull: false
			},
			input_contract_address: {
				type: DataTypes.STRING,
				allowNull: false
			},
			epochInputStateId: {
				type: DataTypes.UUID,
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
			modelName: "AccumulatingEpoch"
		}
	);
	return AccumulatingEpoch;
};
