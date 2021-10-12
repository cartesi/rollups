"use strict";
import { Model, UUIDV4 } from "sequelize";

interface VoucherStateAttributes {
	id: string;
	voucher_address: string;
	vouchers: {
		integer: {
			integer: {
				integer: boolean;
			};
		};
	};
	descartes_hash: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class VoucherState extends Model<VoucherStateAttributes>
		implements VoucherStateAttributes {
		id!: string;
		voucher_address!: string;
		vouchers!: {
			integer: {
				integer: {
					integer: boolean;
				};
			};
		};
		descartes_hash!: string;
		createdAt!: string;
		updatedAt!: string;
	}
	VoucherState.init(
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: UUIDV4,
				allowNull: false,
				primaryKey: true
			},
			voucher_address: {
				type: DataTypes.STRING,
				allowNull: false
			},
			vouchers: {
				type: DataTypes.JSON,
				allowNull: false
			},
			descartes_hash: DataTypes.UUID,
			createdAt: {
				type: DataTypes.DATE
			},
			updatedAt: {
				type: DataTypes.DATE
			}
		},
		{
			sequelize,
			modelName: "VoucherState"
		}
	);
	return VoucherState;
};
