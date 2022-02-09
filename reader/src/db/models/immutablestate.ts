"use strict";
import { Model, UUIDV4 } from "sequelize";

interface ImmutableStateAttributes {
	id: string;
	input_duration: number;
	challenge_period: number;
	contract_creation_timestamp: string;
	dapp_contract_address: string;
	rollups_hash: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class ImmutableState extends Model<ImmutableStateAttributes>
		implements ImmutableStateAttributes {
		id!: string;
		input_duration!: number;
		challenge_period!: number;
		contract_creation_timestamp!: string;
		dapp_contract_address!: string;
		rollups_hash!: string;
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			// define association here
		}
	}
	ImmutableState.init(
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: UUIDV4,
				allowNull: false,
				primaryKey: true
			},
			input_duration: {
				type: DataTypes.STRING,
				allowNull: false
			},
			challenge_period: {
				type: DataTypes.STRING,
				allowNull: false
			},
			contract_creation_timestamp: {
				type: DataTypes.DATE,
				allowNull: false
			},
			dapp_contract_address: {
				type: DataTypes.STRING,
				allowNull: false
			},
			rollups_hash: DataTypes.STRING,
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
			modelName: "ImmutableState"
		}
	);
	return ImmutableState;
};
