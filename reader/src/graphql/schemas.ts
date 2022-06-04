import 'graphql-import-node'
import * as typeDefs from './typeDefs/typeDefs.graphql'
import { makeExecutableSchema } from '@graphql-tools/schema'
import resolvers from './resolversMap'
import { GraphQLSchema } from 'graphql'

const schema: GraphQLSchema = makeExecutableSchema({
  typeDefs: [typeDefs],
  resolvers
})
export default schema