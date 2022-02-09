"use strict";
import { Model, UUID } from "sequelize";

interface RollupsStateAttributes {
	id: string;
	block_hash: string;
	constants: string[];
	initial_epoch: string;
	current_epoch: string;
	current_phase: string;
	voucher_state: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class RollupsState extends Model<RollupsStateAttributes>
		implements RollupsStateAttributes {
		id!: string;
		block_hash!: string;
		constants!: string[];
		initial_epoch!: string;
		current_epoch!: string;
		current_phase!: string;
		voucher_state!: string;
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			RollupsState.hasMany(models.ImmutableState, {
				foreignKey: "rollups_hash"
			});
			RollupsState.hasOne(models.AccumulatingEpoch, {
				foreignKey: "rollups_hash"
			});
			RollupsState.hasOne(models.VoucherState, {
				foreignKey: "rollups_hash"
			});
		}
	}
	RollupsState.init(
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: UUID,
				allowNull: false,
				primaryKey: true
			},
			block_hash: {
				type: DataTypes.STRING,
				allowNull: false
			},
			constants: DataTypes.UUID,
			initial_epoch: DataTypes.STRING,
			current_epoch: DataTypes.UUID,
			current_phase: DataTypes.STRING,
			voucher_state: DataTypes.UUID,
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
			modelName: "RollupsState"
		}
	);
	return RollupsState;
};
