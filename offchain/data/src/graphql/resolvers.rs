/* Copyright 2022 Cartesi Pte. Ltd.
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not
 * use this file except in compliance with the License. You may obtain a copy of
 * the License at http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
 * WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
 * License for the specific language governing permissions and limitations under
 * the License.
 * Parts of the code (BigInt scalar implementatation) is licenced
 * under BSD 2-Clause Copyright (c) 2016, Magnus Hallin
 */

use crate::database;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use juniper::{graphql_object, FieldResult};
use std::sync::Arc;

use super::resolvers_db::*;
pub use super::types::*;

/// Helper trait for Edge types
pub(crate) trait Cursor {
    fn cursor(&self) -> &String;
}

/// Helper macro to implement cursor trait on a struct
macro_rules! implement_cursor {
    ($cursor_type:ty) => {
        impl Cursor for $cursor_type {
            fn cursor(&self) -> &String {
                &self.cursor
            }
        }
    };
}

/// Context for graphql resolvers implementation
#[derive(Clone)]
pub struct Context {
    // Connection is not thread safe to share between threads, we use connection pool
    pub db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}
impl juniper::Context for Context {}

pub struct Pagination {
    pub first: Option<i32>,
    pub last: Option<i32>,
    pub after: Option<String>,
    pub before: Option<String>,
}

impl Pagination {
    pub fn new(
        first: Option<i32>,
        last: Option<i32>,
        after: Option<String>,
        before: Option<String>,
    ) -> Pagination {
        Pagination {
            first,
            last,
            after,
            before,
        }
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Represents the period of time for which a rollup claim is made, encapsulating a batch of inputs and its corresponding outputs"
)]
impl Epoch {
    #[graphql(description = "Epoch identifier")]
    fn id(&self) -> &juniper::ID {
        &self.id
    }

    #[graphql(description = "Epoch index")]
    fn index(&self) -> i32 {
        self.index
    }

    #[graphql(
        description = "Get input from this particular epoch given the input's index"
    )]
    fn input(
        &self,
        #[graphql(description = "Input index within the epoch")] index: i32,
    ) -> FieldResult<Input> {
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        get_input(&conn, None, Some((self.index, index)))
    }

    #[graphql(
        description = "Get inputs from this particular epoch with support for pagination (filtering not implemented at the moment)"
    )]
    fn inputs(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            InputFilter,
        >,
    ) -> FieldResult<InputConnection> {
        let conn = executor.context().db_pool.get()?;
        get_inputs(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            Some(self.index),
        )
    }

    #[graphql(
        description = "Get vouchers from this particular epoch with support for pagination (filtering not implemented at the moment)"
    )]
    fn vouchers(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            VoucherFilter,
        >,
    ) -> FieldResult<VoucherConnection> {
        let conn = executor.context().db_pool.get()?;
        get_vouchers(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            Some(self.index), //epoch index
            None,
        )
    }

    #[graphql(
        description = "Get notices from this particular input with support for pagination (filtering not implemented at the moment)"
    )]
    fn notices(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            NoticeFilter,
        >,
    ) -> FieldResult<NoticeConnection> {
        let conn = executor.context().db_pool.get()?;
        get_notices(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            Some(self.index), //epoch index
            None,
        )
    }

    #[graphql(
        description = "Get reports from this particular epoch with support for pagination (filtering not implemented at the moment)"
    )]
    fn reports(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            ReportFilter,
        >,
    ) -> FieldResult<ReportConnection> {
        let conn = executor.context().db_pool.get()?;
        get_reports(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            Some(self.index), //epoch index
            None,
        )
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Epoch pagination entry"
)]
impl EpochEdge {
    #[graphql(description = "Epoch instance")]
    fn node(&self) -> &Epoch {
        &self.node
    }

    #[graphql(description = "Pagination cursor")]
    fn cursor(&self) -> &String {
        &self.cursor
    }
}
implement_cursor!(EpochEdge);

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Epoch pagination result"
)]
impl EpochConnection {
    #[graphql(description = "Total number of entries that match the query")]
    fn total_count(&self) -> i32 {
        self.total_count
    }

    #[graphql(description = "Pagination entries returned for the current page")]
    fn edges(&self) -> &Vec<EpochEdge> {
        &self.edges
    }

    #[graphql(description = "Epoch instances returned for the current page")]
    fn nodes(&self) -> &Vec<Epoch> {
        &self.nodes
    }

    #[graphql(description = "Pagination metadata")]
    fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Request submitted to the application to advance its state"
)]
impl Input {
    #[graphql(description = "Input identifier")]
    fn id(&self) -> &juniper::ID {
        &self.id
    }

    #[graphql(description = "Input index within its containing epoch")]
    fn index(&self) -> i32 {
        self.index
    }

    #[graphql(description = "Epoch in which the input is contained")]
    fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    #[graphql(description = "Address responsible for submitting the input")]
    fn msg_sender(&self) -> &str {
        self.msg_sender.as_str()
    }

    #[graphql(
        description = "Timestamp associated with the input submission, as defined by the base layer's block in which it was recorded"
    )]
    fn timestamp(&self) -> &i64 {
        &self.timestamp
    }

    #[graphql(
        description = "Number of the base layer block in which the input was recorded"
    )]
    fn block_number(&self) -> &i64 {
        &self.block_number
    }

    #[graphql(
        description = "Get voucher from this particular input given the voucher's index"
    )]
    fn voucher(
        #[graphql(description = "Voucher index within the input")] index: i32,
    ) -> FieldResult<Voucher> {
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        get_voucher(
            &conn,
            None,
            Some((self.epoch.index, self.index)),
            Some(index),
        )
    }

    #[graphql(
        description = "Get notice from this particular input given the notice's index"
    )]
    fn notice(
        #[graphql(description = "Notice index within the input")] index: i32,
    ) -> FieldResult<Notice> {
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        get_notice(
            &conn,
            None,
            Some((self.epoch.index, self.index)),
            Some(index),
        )
    }

    #[graphql(
        description = "Get report from this particular input given report's index"
    )]
    fn report(
        #[graphql(description = "Notice index within the input")] index: i32,
    ) -> FieldResult<Report> {
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        get_report(
            &conn,
            None,
            Some((self.epoch.index, self.index)),
            Some(index),
        )
    }

    #[graphql(
        description = "Get vouchers from this particular input with support for pagination (filtering not implemented at the moment)"
    )]
    fn vouchers(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            VoucherFilter,
        >,
    ) -> FieldResult<VoucherConnection> {
        let conn = executor.context().db_pool.get()?;
        get_vouchers(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            Some(self.epoch.index),
            Some(self.index),
        )
    }

    #[graphql(
        description = "Get notices from this particular input with support for pagination (filtering not implemented at the moment)"
    )]
    fn notices(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            NoticeFilter,
        >,
    ) -> FieldResult<NoticeConnection> {
        let conn = executor.context().db_pool.get()?;
        get_notices(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            Some(self.epoch.index),
            Some(self.index),
        )
    }

    #[graphql(
        description = "Get reports from this particular input with support for pagination (filtering not implemented at the moment)"
    )]
    fn reports(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            ReportFilter,
        >,
    ) -> FieldResult<ReportConnection> {
        let conn = executor.context().db_pool.get()?;
        get_reports(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            Some(self.epoch.index),
            Some(self.index),
        )
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue
    description = "Input pagination entry"
)]
impl InputEdge {
    #[graphql(description = "Input instance")]
    fn node(&self) -> &Input {
        &self.node
    }

    #[graphql(description = "Pagination cursor")]
    fn cursor(&self) -> &String {
        &self.cursor
    }
}
implement_cursor!(InputEdge);

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue
    description = "Input pagination result"
)]
impl InputConnection {
    #[graphql(description = "Total number of entries that match the query")]
    fn total_count(&self) -> i32 {
        self.total_count
    }

    #[graphql(description = "Pagination entries returned for the current page")]
    fn edges(&self) -> &Vec<InputEdge> {
        &self.edges
    }

    #[graphql(description = "Input instances returned for the current page")]
    fn nodes(&self) -> &Vec<Input> {
        &self.nodes
    }

    #[graphql(description = "Pagination metadata")]
    fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Informational statement that can be validated in the base layer blockchain"
)]
impl Notice {
    #[graphql(description = "Notice identifier")]
    fn id(&self) -> &juniper::ID {
        &self.id
    }

    #[graphql(
        description = "Notice index within the context of the input that produced it"
    )]
    fn index(&self) -> i32 {
        self.index
    }

    #[graphql(description = "Input whose processing produced the notice")]
    fn input(&self) -> &Input {
        &self.input
    }

    #[graphql(
        description = "Proof object that allows this notice to be validated in the base layer blockchain"
    )]
    fn proof(&self) -> &Option<Proof> {
        &self.proof
    }

    #[graphql(
        description = "Keccak hash in Ethereum hex binary format, starting with '0x'"
    )]
    fn keccak(&self) -> &str {
        self.keccak.as_str()
    }

    #[graphql(
        description = "Notice data as a payload in Ethereum hex binary format, starting with '0x'"
    )]
    fn payload(&self) -> &str {
        self.payload.as_str()
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Notice pagination entry"
)]
impl NoticeEdge {
    #[graphql(description = "Notice instance")]
    fn node(&self) -> &Notice {
        &self.node
    }

    #[graphql(description = "Pagination cursor")]
    fn cursor(&self) -> &String {
        &self.cursor
    }
}
implement_cursor!(NoticeEdge);

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue
    description = "Notice pagination result"
)]
impl NoticeConnection {
    #[graphql(description = "Total number of entries that match the query")]
    fn total_count(&self) -> i32 {
        self.total_count
    }

    #[graphql(description = "Pagination entries returned for the current page")]
    fn edges(&self) -> &Vec<NoticeEdge> {
        &self.edges
    }

    #[graphql(description = "Notice instances returned for the current page")]
    fn nodes(&self) -> &Vec<Notice> {
        &self.nodes
    }

    #[graphql(description = "Pagination metadata")]
    fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Application log or diagnostic information"
)]
impl Report {
    #[graphql(description = "Report identifier")]
    fn id(&self) -> &juniper::ID {
        &self.id
    }

    #[graphql(
        description = "Report index within the context of the input that produced it"
    )]
    fn index(&self) -> i32 {
        self.index
    }

    #[graphql(description = "Input whose processing produced the report")]
    fn input(&self) -> &Input {
        &self.input
    }

    #[graphql(
        description = "Report data as a payload in Ethereum hex binary format, starting with '0x'"
    )]
    fn payload(&self) -> &str {
        self.payload.as_str()
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Report pagination entry"
)]
impl ReportEdge {
    #[graphql(description = "Report instance")]
    fn node(&self) -> &Report {
        &self.node
    }

    #[graphql(description = "Pagination cursor")]
    fn cursor(&self) -> &String {
        &self.cursor
    }
}
implement_cursor!(ReportEdge);

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue
    description = "Report pagination result"
)]
impl ReportConnection {
    #[graphql(description = "Total number of entries that match the query")]
    fn total_count(&self) -> i32 {
        self.total_count
    }

    #[graphql(description = "Pagination entries returned for the current page")]
    fn edges(&self) -> &Vec<ReportEdge> {
        &self.edges
    }

    #[graphql(description = "Report instances returned for the current page")]
    fn nodes(&self) -> &Vec<Report> {
        &self.nodes
    }

    #[graphql(description = "Pagination metadata")]
    fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Data that can be used as proof to validate notices and execute vouchers on the base layer blockchain"
)]
impl Proof {
    #[graphql(
        description = "Merkle root of all input's output metadata hashes, given in Ethereum hex binary format (32 bytes), starting with '0x'"
    )]
    fn output_hashes_root_hash(&self) -> &str {
        self.output_hashes_root_hash.as_str()
    }

    #[graphql(
        description = "Merkle root of all epoch's voucher metadata hashes, given in Ethereum hex binary format (32 bytes), starting with '0x'"
    )]
    fn vouchers_epoch_root_hash(&self) -> &str {
        self.vouchers_epoch_root_hash.as_str()
    }

    #[graphql(
        description = "Merkle root of all epoch's notice metadata hashes, given in Ethereum hex binary format (32 bytes), starting with '0x'"
    )]
    fn notices_epoch_root_hash(&self) -> &str {
        self.notices_epoch_root_hash.as_str()
    }

    #[graphql(
        description = "Hash of the machine state claimed for this epoch, given in Ethereum hex binary format (32 bytes), starting with '0x'"
    )]
    fn machine_state_hash(&self) -> &str {
        self.machine_state_hash.as_str()
    }

    #[graphql(
        description = "Array of hashes representing a proof that this output metadata is in the metadata memory range, where each hash is given in Ethereum hex binary format (32 bytes), starting with '0x'"
    )]
    fn keccak_in_hashes_siblings(&self) -> &Vec<String> {
        self.keccak_in_hashes_siblings.as_ref()
    }

    #[graphql(
        description = "Array of hashes representing a proof that this output metadata is in the epoch's output memory range, where each hash is given in Ethereum hex binary format (32 bytes), starting with '0x'"
    )]
    fn output_hashes_in_epoch_siblings(&self) -> &Vec<String> {
        self.output_hashes_in_epoch_siblings.as_ref()
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Representation of a transaction that can be carried out on the base layer blockchain, such as a transfer of assets"
)]
impl Voucher {
    #[graphql(description = "Voucher identifier")]
    fn id(&self) -> &juniper::ID {
        &self.id
    }

    #[graphql(
        description = "Voucher index within the context of the input that produced it"
    )]
    fn index(&self) -> i32 {
        self.index
    }

    #[graphql(description = "Input whose processing produced the notice")]
    fn input(&self) -> &Input {
        &self.input
    }

    #[graphql(
        description = "Proof object that allows this voucher to be validated and executed in the base layer blockchain"
    )]
    fn proof(&self) -> &Option<Proof> {
        &self.proof
    }

    #[graphql(
        description = "Transaction destination address in Ethereum hex binary format (20 bytes), starting with '0x'"
    )]
    fn destination(&self) -> &str {
        self.destination.as_str()
    }

    #[graphql(
        description = "Transaction payload in Ethereum hex binary format, starting with '0x'"
    )]
    fn payload(&self) -> &str {
        self.payload.as_str()
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue,
    description = "Voucher pagination entry"
)]
impl VoucherEdge {
    #[graphql(description = "Voucher instance")]
    fn node(&self) -> &Voucher {
        &self.node
    }

    #[graphql(description = "Pagination cursor")]
    fn cursor(&self) -> &String {
        &self.cursor
    }
}
implement_cursor!(VoucherEdge);

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue
    description = "Voucher pagination result"
)]
impl VoucherConnection {
    #[graphql(description = "Total number of entries that match the query")]
    fn total_count(&self) -> i32 {
        self.total_count
    }

    #[graphql(description = "Pagination entries returned for the current page")]
    fn edges(&self) -> &Vec<VoucherEdge> {
        &self.edges
    }

    #[graphql(description = "Voucher instances returned for the current page")]
    fn nodes(&self) -> &Vec<Voucher> {
        &self.nodes
    }

    #[graphql(description = "Pagination metadata")]
    fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

#[graphql_object(
    context = Context,
    Scalar = RollupsGraphQLScalarValue
    description = "Top level queries"
)]
impl Query {
    #[graphql(description = "Get epoch based on its identifier")]
    fn epoch(
        #[graphql(description = "Epoch identifier")] id: juniper::ID,
    ) -> FieldResult<Epoch> {
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        get_epoch(&conn, Some(id), None)
    }

    #[graphql(description = "Get epoch based on its index")]
    fn epoch_i(
        #[graphql(description = "Epoch index")] index: i32,
    ) -> FieldResult<Epoch> {
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        get_epoch(&conn, None, Some(index))
    }

    #[graphql(description = "Get input based on its identifier")]
    fn input(
        #[graphql(description = "Input identifier")] id: juniper::ID,
    ) -> FieldResult<Input> {
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        get_input(&conn, Some(id), None)
    }

    #[graphql(description = "Get notice based on its identifier")]
    fn notice(
        #[graphql(description = "Notice identifier")] id: juniper::ID,
    ) -> FieldResult<Notice> {
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        get_notice(&conn, Some(id), None, None)
    }

    #[graphql(description = "Get report based on its identifier")]
    fn report(
        #[graphql(description = "Report identifier")] id: juniper::ID,
    ) -> FieldResult<Report> {
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;

        get_report(&conn, Some(id), None, None)
    }

    #[graphql(description = "Get voucher based on its identifier")]
    fn voucher(
        #[graphql(description = "Voucher identifier")] id: juniper::ID,
    ) -> FieldResult<Voucher> {
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        get_voucher(&conn, Some(id), None, None)
    }

    #[graphql(description = "Get epochs with support for pagination")]
    fn epochs(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
    ) -> FieldResult<EpochConnection> {
        let conn = executor.context().db_pool.get()?;
        let epochs: Vec<Epoch> =
            get_epochs(&conn, Pagination::new(first, last, after, before))?
                .into_values()
                .collect();
        // Epoch id and index are correlated and strictly increasing, no
        // need to sort epoch by id
        let edges: Vec<EpochEdge> = epochs
            .clone()
            .into_iter()
            .map(|epoch| EpochEdge {
                cursor: epoch.id.to_string(),
                node: epoch,
            })
            .collect();

        let total_count = database::schema::epochs::dsl::epochs
            .count()
            .get_result::<i64>(&conn)? as i32;
        let page_info = calculate_page_info(&edges, total_count);
        Ok(EpochConnection {
            page_info,
            total_count: total_count as i32,
            edges,
            nodes: epochs,
        })
    }

    #[graphql(
        description = "Get inputs with support for pagination (filtering not implemented at the moment)"
    )]
    fn inputs(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            InputFilter,
        >,
    ) -> FieldResult<InputConnection> {
        let conn = executor.context().db_pool.get()?;
        get_inputs(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            None,
        )
    }

    #[graphql(
        description = "Get vouchers with support for pagination (filtering not implemented at the moment)"
    )]
    fn vouchers(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            VoucherFilter,
        >,
    ) -> FieldResult<VoucherConnection> {
        let conn = executor.context().db_pool.get()?;
        get_vouchers(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            None,
            None,
        )
    }

    #[graphql(
        description = "Get notices with support for pagination (filtering not implemented at the moment)"
    )]
    fn notices(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            NoticeFilter,
        >,
    ) -> FieldResult<NoticeConnection> {
        let conn = executor.context().db_pool.get()?;
        get_notices(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            None,
            None,
        )
    }

    #[graphql(
        description = "Get reports with support for pagination (filtering not implemented at the moment)"
    )]
    fn reports(
        &self,
        #[graphql(
            description = "Get at most the first `n` entries (forward pagination)"
        )]
        first: Option<i32>,
        #[graphql(
            description = "Get at most the last `n` entries (backward pagination)"
        )]
        last: Option<i32>,
        #[graphql(
            description = "Get entries that come after the provided cursor (forward pagination)"
        )]
        after: Option<String>,
        #[graphql(
            description = "Get entries that come before the provided cursor (backward pagination)"
        )]
        before: Option<String>,
        #[graphql(description = "Filter entries to retrieve")] r#where: Option<
            ReportFilter,
        >,
    ) -> FieldResult<ReportConnection> {
        let conn = executor.context().db_pool.get()?;
        get_reports(
            &conn,
            Pagination::new(first, last, after, before),
            r#where,
            None,
            None,
        )
    }
}

impl juniper::ScalarValue for RollupsGraphQLScalarValue {
    type Visitor = RollupsGraphQLScalarValueVisitor;

    fn as_int(&self) -> Option<i32> {
        match *self {
            Self::Int(ref i) => Some(*i),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<String> {
        match *self {
            Self::String(ref s) => Some(s.clone()),
            _ => None,
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match *self {
            Self::String(ref s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_float(&self) -> Option<f64> {
        match *self {
            Self::Int(ref i) => Some(*i as f64),
            Self::Float(ref f) => Some(*f),
            _ => None,
        }
    }

    fn as_boolean(&self) -> Option<bool> {
        match *self {
            Self::Boolean(ref b) => Some(*b),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct RollupsGraphQLScalarValueVisitor;

impl<'de> serde::de::Visitor<'de> for RollupsGraphQLScalarValueVisitor {
    type Value = RollupsGraphQLScalarValue;

    fn expecting(
        &self,
        formatter: &mut std::fmt::Formatter,
    ) -> std::fmt::Result {
        formatter.write_str("a valid input value")
    }

    fn visit_bool<E>(
        self,
        value: bool,
    ) -> Result<RollupsGraphQLScalarValue, E> {
        Ok(RollupsGraphQLScalarValue::Boolean(value))
    }

    fn visit_i32<E>(self, value: i32) -> Result<RollupsGraphQLScalarValue, E>
    where
        E: serde::de::Error,
    {
        Ok(RollupsGraphQLScalarValue::Int(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<RollupsGraphQLScalarValue, E>
    where
        E: serde::de::Error,
    {
        if value <= i32::max_value() as i64 {
            self.visit_i32(value as i32)
        } else {
            Ok(RollupsGraphQLScalarValue::BigInt(value))
        }
    }

    fn visit_u32<E>(self, value: u32) -> Result<RollupsGraphQLScalarValue, E>
    where
        E: serde::de::Error,
    {
        if value <= i32::max_value() as u32 {
            self.visit_i32(value as i32)
        } else {
            self.visit_u64(value as u64)
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<RollupsGraphQLScalarValue, E>
    where
        E: serde::de::Error,
    {
        if value <= i64::MAX as u64 {
            self.visit_i64(value as i64)
        } else {
            Ok(RollupsGraphQLScalarValue::Float(value as f64))
        }
    }

    fn visit_f64<E>(self, value: f64) -> Result<RollupsGraphQLScalarValue, E> {
        Ok(RollupsGraphQLScalarValue::Float(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<RollupsGraphQLScalarValue, E>
    where
        E: serde::de::Error,
    {
        self.visit_string(value.into())
    }

    fn visit_string<E>(
        self,
        value: String,
    ) -> Result<RollupsGraphQLScalarValue, E> {
        Ok(RollupsGraphQLScalarValue::String(value))
    }
}
