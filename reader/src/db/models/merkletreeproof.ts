"use strict";
import { Model, UUID } from "sequelize";

interface Hash {
	data: string;
}

interface MerkleTreeProofAttributes {
	id: string;
	target_address: string;
	log2_target_size: string;
	target_hash: string;
	log2_root_size: string;
	root_hash: string;
	sibling_hashes: [Hash];
	createdAt: string;
	updatedAt: string;
}

module.exports = (sequelize: any, DataTypes: any) => {
	class MerkleTreeProof extends Model<MerkleTreeProofAttributes>
		implements MerkleTreeProofAttributes {
		id!: string;
		target_address!: string;
		log2_target_size!: string;
		target_hash!: string;
		log2_root_size!: string;
		root_hash!: string;
		sibling_hashes!: [Hash];
		createdAt!: string;
		updatedAt!: string;

		static associate(models: any) {
			// define association here
		}
	}
	MerkleTreeProof.init(
		{
			id: {
				allowNull: false,
				primaryKey: true,
				defaultValue: UUID,
				type: DataTypes.UUID
			},
			target_address: {
				type: DataTypes.STRING,
				allowNull: false
			},
			log2_target_size: {
				type: DataTypes.STRING,
				allowNull: false
			},
			target_hash: {
				type: DataTypes.STRING,
				allowNull: false
			},
			log2_root_size: {
				type: DataTypes.STRING,
				allowNull: false
			},
			root_hash: {
				type: DataTypes.STRING,
				allowNull: false
			},
			sibling_hashes: {
				type: DataTypes.JSON,
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
			modelName: "MerkleTreeProof"
		}
	);
	return MerkleTreeProof;
};
