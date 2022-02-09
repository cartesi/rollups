import express, { Request, Response, NextFunction } from "express";
import { ApolloServer } from "apollo-server-express";
import schema from "./graphql/schemas";
import db from "./db/models";

// Metrics
import { responseTimesHistogram } from "./utils/metrics";

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

app.use("/graphql", (req: Request, res: Response, next: NextFunction) => {
	const startHrTime = process.hrtime();

	res.on("finish", async () => {
		if (
			req.body &&
			req.body?.operationName &&
			req.body?.operationName !== "IntrospectionQuery"
		) {
			const elapsedHrTime = process.hrtime(startHrTime);
			const elapsedTimeInMs = elapsedHrTime[0] * 1000 + elapsedHrTime[1] / 1e6;

			responseTimesHistogram.observe(elapsedTimeInMs);
		}
	});

	next();
});

server.applyMiddleware({ app, path: "/graphql" });
app.listen(PORT, () => {
	console.log(
		`\n🚀      GraphQL is now running on http://localhost:${PORT}/graphql`
	);
});
