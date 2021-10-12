import express from "express";
import { ApolloServer } from "apollo-server-express";
import schema from "./graphql/schemas";
import db from "./db/models";

const joinMonsterAdapt = require("join-monster-graphql-tools-adapter");

import joinMonsterMetadata from "./joinMonsterMetadata";

joinMonsterAdapt(schema, joinMonsterMetadata);

const PORT = 4000;

const app = express();

const server = new ApolloServer({
	schema
});

db.sequelize
	.authenticate()
	.then(() => {
		console.log("Connected to database successfully");
	})
	.catch(() => {
		console.error("Error connecting to database");
	});

server.applyMiddleware({ app, path: "/graphql" });
app.listen(PORT, () => {
	console.log(
		`\n🚀      GraphQL is now running on http://localhost:${PORT}/graphql`
	);
});
