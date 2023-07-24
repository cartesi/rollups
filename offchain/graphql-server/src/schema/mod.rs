// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

mod resolvers;
mod scalar;

pub use resolvers::{Context, Query};
pub use scalar::RollupsGraphQLScalarValue;

pub type Schema = juniper::RootNode<
    'static,
    Query,
    juniper::EmptyMutation<Context>,
    juniper::EmptySubscription<Context>,
    RollupsGraphQLScalarValue,
>;
