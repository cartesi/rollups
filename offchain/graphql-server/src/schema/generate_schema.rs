// Copyright 2023 Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

// Note: it was impractical to generate schema with build.rs,
// because it is executed before crate is built, and many structures/entities
// from graphql module must be used to generate schema

use juniper::{EmptyMutation, EmptySubscription};
use std::fs::File;
use std::io::Write;

use graphql_server::schema::{Query, Schema};

const GRAPHQL_SCHEMA_FILE: &str = "schema.graphql";

fn main() {
    let schema = Schema::new_with_scalar_value(
        Query {},
        EmptyMutation::new(),
        EmptySubscription::new(),
    );
    let graphql_schema = schema.as_schema_language();
    let mut graphql_schema_file = File::create(GRAPHQL_SCHEMA_FILE).unwrap();
    match write!(graphql_schema_file, "{}", graphql_schema) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error writing schema to file {}", e.to_string());
        }
    }
}
