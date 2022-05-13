// Note: it was impractical to generate schema with build.rs,
// because it is executed before crate is built, and many structures/entities
// from graphql module must be used to generate schema

use std::io::Write;

extern crate rollups_data;
use rollups_data::graphql;
const GRAPHQL_SCHEMA_FILE: &str = "graphql/typeDefs.graphql";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create graphql schema object from code definition
    let schema = graphql::types::Schema::new_with_scalar_value(
        graphql::resolvers::Query {},
        juniper::EmptyMutation::<graphql::resolvers::Context>::new(),
        juniper::EmptySubscription::<graphql::resolvers::Context>::new(),
    );

    // Convert the Rust schema into the GraphQL Schema Language.
    let graphql_schema = schema.as_schema_language();
    let mut graphql_schema_file =
        std::fs::File::create(GRAPHQL_SCHEMA_FILE).unwrap();
    match write!(graphql_schema_file, "{}", graphql_schema) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error writing schema to file {}", e.to_string());
        }
    }

    Ok(())
}
