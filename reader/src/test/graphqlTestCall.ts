import { graphql } from "graphql";
import { makeExecutableSchema } from "@graphql-tools/schema";
import { GraphQLSchema } from "graphql";

import typeDefs from "./typeDefs";
import resolvers from "../graphql/resolversMap";

const schema: GraphQLSchema = makeExecutableSchema({
	typeDefs,
	resolvers
});

export const graphqlTestCall = async (query: any, variables?: any) => {
	return graphql({
		schema,
		source: query,
		variableValues: variables,
	});
};
