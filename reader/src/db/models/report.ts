"use strict";
import { Model, UUIDV4 } from "sequelize";

interface ReportAttribute {
	id: string;
	payload: string;
	processed_input_id: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class Report extends Model<ReportAttribute> implements ReportAttribute {
		id!: string;
		payload!: string;
		processed_input_id!: string;
		createdAt!: string;
		updatedAt!: string;
	}
	Report.init(
		{
			id: {
				allowNull: false,
				primaryKey: true,
				defaultValue: UUIDV4,
				type: DataTypes.UUID
			},
			payload: {
				type: DataTypes.STRING
			},
			processed_input_id: DataTypes.UUID,
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
			modelName: "Report"
		}
	);
	return Report;
};
