import { graphql } from "graphql";
import { makeExecutableSchema } from "graphql-tools";
import { GraphQLSchema } from "graphql";

import typeDefs from "./typeDefs";
import resolvers from "../graphql/resolversMap";

const schema: GraphQLSchema = makeExecutableSchema({
	typeDefs,
	resolvers
});

export const graphqlTestCall = async (query: any, variables?: any) => {
	return graphql(schema, query, undefined, undefined, variables);
};
