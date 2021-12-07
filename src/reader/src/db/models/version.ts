"use strict";
import { Model, UUIDV4 } from "sequelize";

interface VersionAttributes {
	id: string;
	version: string;
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class Version extends Model<VersionAttributes> implements VersionAttributes {
		id!: string;
		version!: string;
		createdAt!: string;
		updatedAt!: string;
	}
	Version.init(
		{
			id: {
				type: DataTypes.UUID,
				defaultValue: UUIDV4,
				allowNull: false,
				primaryKey: true
			},
			version: {
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
			modelName: "Version"
		}
	);
	return Version;
};
