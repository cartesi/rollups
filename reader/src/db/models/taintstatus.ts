"use strict";
import { Model, UUID } from "sequelize";

interface TaintStatusAttributes {
	id: string;
	error_code: number;
	error_message: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class TaintStatus extends Model<TaintStatusAttributes>
		implements TaintStatusAttributes {
		id!: string;
		error_code!: number;
		error_message!: string;
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			// define association here
		}
	}
	TaintStatus.init(
		{
			id: {
				allowNull: false,
				primaryKey: true,
				defaultValue: UUID,
				type: DataTypes.UUID
			},
			error_code: {
				type: DataTypes.INTEGER,
				allowNull: false
			},
			error_message: {
				type: DataTypes.STRING,
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
			modelName: "TaintStatus"
		}
	);
	return TaintStatus;
};
