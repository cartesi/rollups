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
use crate::database::{DbEpoch, DbInput, DbNotice};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use juniper::{graphql_object, FieldResult};
use snafu::ResultExt;
use std::sync::Arc;
use tracing::warn;

pub use super::types::*;

/// Helper trait for Edge types
trait Cursor {
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
pub struct Context {
    // Connection is not thread safe to share between threads, we use connection pool
    pub db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}
impl juniper::Context for Context {}

#[graphql_object(context = Context, Scalar = RollupsGraphQLScalarValue)]
impl Epoch {
    fn id(&self) -> &juniper::ID {
        &self.id
    }

    fn index(&self) -> i32 {
        self.index
    }

    fn inputs(
        &self,
        first: Option<i32>,
        last: Option<i32>,
        after: Option<String>,
        before: Option<String>,
    ) -> FieldResult<InputConnection> {
        let conn = executor.context().db_pool.get()?;
        println!("Checkpoint 0");
        let mut inputs: Vec<Input> = get_inputs_by_cursor(
            &conn,
            first,
            last,
            after,
            before,
            Some(self.index),
        )?
        .into_iter()
        .map(|(_, input)| input)
        .collect();
        inputs.sort(); // sort by id, they are sorted by index
        let edges: Vec<InputEdge> = inputs
            .clone()
            .into_iter()
            .map(|input| InputEdge {
                cursor: input.id.to_string(),
                node: input,
            })
            .collect();
        let total_count = database::schema::inputs::dsl::inputs
            .count()
            .get_result::<i64>(&conn)? as i32;
        let page_info = calculate_page_info(&edges, total_count);
        Ok(InputConnection {
            page_info,
            total_count: total_count as i32,
            edges,
            nodes: inputs,
        })
    }
}

#[graphql_object(context = Context, Scalar = RollupsGraphQLScalarValue)]
impl EpochEdge {
    fn node(&self) -> &Epoch {
        &self.node
    }

    fn cursor(&self) -> &String {
        &self.cursor
    }
}
implement_cursor!(EpochEdge);

#[graphql_object(context = Context, Scalar = RollupsGraphQLScalarValue)]
impl EpochConnection {
    fn total_count(&self) -> i32 {
        self.total_count
    }

    fn edges(&self) -> &Vec<EpochEdge> {
        &self.edges
    }

    fn nodes(&self) -> &Vec<Epoch> {
        &self.nodes
    }

    fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

/// Attribute for filtering sql queries
#[allow(dead_code)]
enum DbFilterType {
    Id(i32),
    EpochIndex(i32),
    EpochAndInputIndex(i32, i32),
}

fn get_epoch_from_db(
    val: DbFilterType,
    conn: &PgConnection,
) -> Result<Epoch, super::error::Error> {
    use crate::database::schema::epochs::dsl::*;
    let mut query = epochs.into_boxed();
    match val {
        DbFilterType::Id(epoch_id) => query = query.filter(id.eq(epoch_id)),
        DbFilterType::EpochIndex(index) => {
            query = query.filter(epoch_index.eq(index))
        }
        _ => {
            return Err(super::error::Error::InvalidParameterError {});
        }
    }
    let query_result = query
        .load::<DbEpoch>(conn)
        .context(super::error::DatabaseError)?;
    if let Some(db_epoch) = query_result.get(0) {
        Ok(Epoch {
            id: juniper::ID::new(db_epoch.epoch_index.to_string()),
            index: db_epoch.epoch_index,
        })
    } else {
        match val {
            DbFilterType::Id(epoch_id) => {
                Err(super::error::Error::ItemNotFound {
                    item_type: "epoch".to_string(),
                    id: epoch_id.to_string(),
                })
            }
            DbFilterType::EpochIndex(index) => {
                Err(super::error::Error::EpochNotFound { index })
            }
            _ => Err(super::error::Error::InvalidParameterError {}),
        }
    }
}

/// Create ordered map (by indexes) of index->epoch
fn process_db_epochs(
    db_epochs: Vec<DbEpoch>,
) -> FieldResult<std::collections::BTreeMap<i32, Epoch>> {
    let result: std::collections::BTreeMap<i32, Epoch> = db_epochs
        .iter()
        .map(|db_epoch| {
            (
                db_epoch.epoch_index,
                Epoch {
                    id: juniper::ID::new(db_epoch.id.to_string()),
                    index: db_epoch.epoch_index,
                },
            )
        })
        .collect();
    Ok(result)
}

/// Get epochs from database and return ordered map of <epoch index, Epoch>
fn get_epochs(
    conn: &PgConnection,
    first: Option<i32>,
    last: Option<i32>,
    after: Option<String>,
    before: Option<String>,
) -> FieldResult<std::collections::BTreeMap<i32, Epoch>> {
    use crate::database::schema::epochs::dsl::*;

    let mut query = epochs.into_boxed();
    let start_pos = if let Some(first) = first {
        let first = std::cmp::max(0, first - 1);
        query = query.offset(first.into());
        first
    } else {
        0
    };
    if let Some(last) = last {
        query = query.limit((last - start_pos).into());
        Some(last)
    } else {
        None
    };
    if let Some(after) = after {
        if let Ok(after_i32) = after.parse::<i32>() {
            query = query.filter(id.gt(after_i32));
        }
    };
    if let Some(before) = before {
        if let Ok(before_i32) = before.parse::<i32>() {
            query = query.filter(id.lt(before_i32));
        }
    };
    query = query.order_by(id.asc());
    let db_epochs = query.load::<DbEpoch>(conn)?;
    process_db_epochs(db_epochs)
}

/// Get map (ordered by index) of epochs from database for every index from the list
fn get_epochs_by_indexes(
    indexes: Vec<i32>,
    conn: &PgConnection,
) -> FieldResult<std::collections::BTreeMap<i32, Epoch>> {
    use crate::database::schema::epochs::dsl::*;
    let mut query = epochs.into_boxed();
    query = query.filter(epoch_index.eq_any(indexes));
    query = query.order_by(epoch_index.asc());
    let db_epochs = query.load::<DbEpoch>(conn)?;
    process_db_epochs(db_epochs)
}

/// Get single input (by id or by index) from database
fn get_input_from_db(
    val: DbFilterType,
    conn: &PgConnection,
) -> Result<Input, crate::graphql::error::Error> {
    use crate::database::schema::inputs::dsl::*;
    let mut query = inputs.into_boxed();
    match val {
        DbFilterType::Id(input_id) => query = query.filter(id.eq(input_id)),
        DbFilterType::EpochAndInputIndex(ep_index, in_index) => {
            query = query.filter(epoch_index.eq(ep_index));
            query = query.filter(input_index.eq(in_index));
        }
        _ => {
            return Err(super::error::Error::InvalidParameterError {});
        }
    }
    let query_result = query
        .load::<DbInput>(conn)
        .context(super::error::DatabaseError)?;
    if let Some(db_input) = query_result.get(0) {
        let epoch = get_epoch_from_db(
            DbFilterType::EpochIndex(db_input.epoch_index),
            conn,
        )?;
        Ok(Input {
            id: juniper::ID::new(db_input.id.to_string()),
            index: db_input.input_index,
            epoch,
            block_number: db_input.block_number,
        })
    } else {
        match val {
            DbFilterType::Id(input_id) => {
                warn!("Unable to find input in database with id {}", input_id);
                Err(super::error::Error::ItemNotFound {
                    item_type: "input".to_string(),
                    id: input_id.to_string(),
                })
            }
            DbFilterType::EpochAndInputIndex(ep_index, in_index) => {
                warn!("Unable to find input in database, epoch_index={} input_index {}", ep_index, in_index);
                Err(super::error::Error::InputNotFound {
                    epoch_index: ep_index,
                    index: in_index,
                })
            }
            _ => Err(super::error::Error::InvalidParameterError {}),
        }
    }
}

/// Create ordered (by epoch indexes and input indexes) map of (epoch index, input index)->input
fn process_db_inputs(
    db_inputs: Vec<DbInput>,
    conn: &PgConnection,
) -> FieldResult<std::collections::BTreeMap<(i32, i32), Input>> {
    //Get all epochs related to those inputs
    let mut epoch_indexes = std::collections::HashSet::<i32>::new();
    db_inputs.iter().for_each(|db_input| {
        epoch_indexes.insert(db_input.epoch_index);
    });
    let epochs =
        get_epochs_by_indexes(epoch_indexes.into_iter().collect(), conn)?;

    let result: Result<
        std::collections::BTreeMap<(i32, i32), Input>,
        super::error::Error,
    > = db_inputs
        .into_iter()
        .map(|db_input| {
            Ok((
                (db_input.epoch_index, db_input.input_index),
                Input {
                    id: juniper::ID::from(db_input.id.to_string()),
                    index: db_input.input_index as i32,
                    block_number: db_input.block_number,
                    epoch: match epochs.get(&db_input.epoch_index).ok_or_else(
                        || {
                            warn!(
                                "Unable to get epoch with index: {}",
                                db_input.epoch_index
                            );
                            Err(super::error::Error::IndexNotFound {
                                item_type: "epoch".to_string(),
                                index: db_input.epoch_index,
                            })
                        },
                    ) {
                        Ok(val) => val.clone(),
                        Err(e) => {
                            warn!(
                                "Unable to get epoch {} for input id: {}",
                                db_input.epoch_index, db_input.id
                            );
                            return e;
                        }
                    },
                },
            ))
        })
        .collect();
    result.map_err(|e| e.into())
}

/// Get inputs from database and return map of <(epoch index, input index), Input>
fn get_inputs_by_cursor(
    conn: &PgConnection,
    first: Option<i32>,
    last: Option<i32>,
    after: Option<String>,
    before: Option<String>,
    ep_index: Option<i32>,
) -> FieldResult<std::collections::BTreeMap<(i32, i32), Input>> {
    use crate::database::schema::inputs::dsl::*;
    let mut query = inputs.into_boxed();
    let start_pos = if let Some(first) = first {
        let first = std::cmp::max(0, first - 1);
        query = query.offset(first.into());
        first
    } else {
        0
    };
    if let Some(last) = last {
        query = query.limit((last - start_pos).into());
        Some(last)
    } else {
        None
    };
    if let Some(after) = after {
        if let Ok(after_i32) = after.parse::<i32>() {
            query = query.filter(id.gt(after_i32));
        }
    };
    if let Some(before) = before {
        if let Ok(before_i32) = before.parse::<i32>() {
            query = query.filter(id.lt(before_i32));
        }
    };
    if let Some(ep_index) = ep_index {
        query = query.filter(epoch_index.eq(ep_index));
    };
    query = query.order_by(id.asc());
    let db_inputs = query.load::<DbInput>(conn)?;
    process_db_inputs(db_inputs, conn)
}

/// Get notices from database and return ordered map of <notice id, Notice>
fn process_db_notices(
    db_notices: Vec<DbNotice>,
    conn: &PgConnection,
) -> FieldResult<std::collections::BTreeMap<i32, Notice>> {
    //Get all inputs related to those notices
    let mut input_indexes = std::collections::HashSet::<i32>::new();
    db_notices.iter().for_each(|db_notice| {
        input_indexes.insert(db_notice.input_index);
    });
    let inputs =
        get_inputs_by_indexes(input_indexes.into_iter().collect(), conn)?;

    let result: Result<
        std::collections::BTreeMap<i32, Notice>,
        super::error::Error,
    > = db_notices
        .into_iter()
        .map(|db_notice| {
            Ok((
                db_notice.id,
                Notice {
                    id: juniper::ID::new(db_notice.id.to_string()),
                    index: db_notice.input_index as i32,
                    session_id: db_notice.session_id,
                    input: match inputs.get(&(db_notice.epoch_index, db_notice.input_index)).ok_or(Err(
                        super::error::Error::InputNotFound {
                            epoch_index: db_notice.epoch_index,
                            index: db_notice.input_index,
                        },
                    )) {
                        Ok(val) => val.clone(),
                        Err(e) => {
                            warn!("Unable to get input index={} from epoch index={} for notice id={}",
                                db_notice.input_index, db_notice.epoch_index, db_notice.id );
                            return e;
                        }
                    },
                    keccak: db_notice.keccak,
                    payload: "0x".to_string()
                        + hex::encode(
                            db_notice.payload.as_ref().unwrap_or(&Vec::new()),
                        )
                        .as_str(),
                },
            ))
        })
        .collect();
    result.map_err(|e| e.into())
}

/// Get notices from database and return ordered map of <notice id, Notice>
fn get_notices_by_cursor(
    conn: &PgConnection,
    first: Option<i32>,
    last: Option<i32>,
    after: Option<String>,
    before: Option<String>,
    input_index: Option<i32>,
    epoch_index: Option<i32>,
) -> FieldResult<std::collections::BTreeMap<i32, Notice>> {
    use crate::database::schema::notices;
    let mut query = notices::dsl::notices.into_boxed();
    let start_pos = if let Some(first) = first {
        let first = std::cmp::max(0, first - 1);
        query = query.offset(first.into());
        first
    } else {
        0
    };
    if let Some(last) = last {
        query = query.limit((last - start_pos).into());
        Some(last)
    } else {
        None
    };
    if let Some(after) = after {
        if let Ok(after_i32) = after.parse::<i32>() {
            query = query.filter(notices::dsl::id.gt(after_i32));
        }
    };
    if let Some(before) = before {
        if let Ok(before_i32) = before.parse::<i32>() {
            query = query.filter(notices::dsl::id.lt(before_i32));
        }
    };
    if let Some(in_index) = input_index {
        query = query.filter(
            crate::database::schema::notices::dsl::input_index.eq(in_index),
        );
    };
    if let Some(ep_index) = epoch_index {
        query = query.filter(
            crate::database::schema::notices::dsl::epoch_index.eq(ep_index),
        );
    };
    query = query.order_by(notices::dsl::id.asc());
    let db_notices = query.load::<DbNotice>(conn)?;
    process_db_notices(db_notices, conn)
}

/// Get inputs from database for every index from the list
fn get_inputs_by_indexes(
    indexes: Vec<i32>,
    conn: &PgConnection,
) -> FieldResult<std::collections::BTreeMap<(i32, i32), Input>> {
    use crate::database::schema::inputs::dsl::*;
    let mut query = inputs.into_boxed();
    query = query.filter(input_index.eq_any(indexes));
    query = query.order_by(id.asc());
    let db_inputs = query.load::<DbInput>(conn)?;
    process_db_inputs(db_inputs, conn)
}

/// Calculate pagination info structure based on edges list
/// Uses provided total_count to calculate `has_next_page`
fn calculate_page_info<T>(edges: &[T], total_count: i32) -> PageInfo
where
    T: Cursor,
{
    let start_cursor = match edges.get(0) {
        Some(edge) => edge.cursor().clone(),
        _ => String::from(""),
    };
    let end_cursor = match edges.iter().last() {
        Some(edge) => edge.cursor().clone(),
        _ => String::from(""),
    };
    let has_previous_page = match edges.get(0) {
        Some(val) => val.cursor().parse::<i32>().unwrap_or_default() > 1,
        None => false,
    };
    let has_next_page = match edges.iter().last() {
        Some(val) => {
            val.cursor().parse::<i32>().unwrap_or_default() < total_count
        }
        None => false,
    };
    PageInfo {
        has_previous_page,
        has_next_page,
        start_cursor,
        end_cursor,
    }
}

#[graphql_object(context = Context, Scalar = RollupsGraphQLScalarValue)]
impl Input {
    fn id(&self) -> &juniper::ID {
        &self.id
    }

    fn index(&self) -> i32 {
        self.index
    }

    fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    fn block_number(&self) -> &i64 {
        &self.block_number
    }

    /// Get notices from this particular input
    /// with additional ability to filter and paginate them
    fn notices(
        &self,
        first: Option<i32>,
        last: Option<i32>,
        after: Option<String>,
        before: Option<String>,
    ) -> FieldResult<NoticeConnection> {
        let conn = executor.context().db_pool.get()?;
        let notices: Vec<Notice> = get_notices_by_cursor(
            &conn,
            first,
            last,
            after,
            before,
            Some(self.index),
            Some(self.epoch.index),
        )?
        .into_values()
        .collect();

        let total_count = database::schema::notices::dsl::notices
            .filter(
                crate::database::schema::notices::dsl::input_index
                    .eq(&self.index),
            )
            .count()
            .get_result::<i64>(&conn)? as i32;
        let edges: Vec<NoticeEdge> = notices
            .clone()
            .into_iter()
            .map(|notice| NoticeEdge {
                cursor: notice.id.to_string(),
                node: notice,
            })
            .collect();
        let page_info = calculate_page_info(&edges, total_count);
        Ok(NoticeConnection {
            page_info,
            total_count: total_count as i32,
            edges,
            nodes: notices,
        })
    }
}

#[graphql_object(context = Context, Scalar = RollupsGraphQLScalarValue)]
impl InputEdge {
    fn node(&self) -> &Input {
        &self.node
    }

    fn cursor(&self) -> &String {
        &self.cursor
    }
}
implement_cursor!(InputEdge);

#[graphql_object(context = Context, Scalar = RollupsGraphQLScalarValue)]
impl InputConnection {
    fn total_count(&self) -> i32 {
        self.total_count
    }

    fn edges(&self) -> &Vec<InputEdge> {
        &self.edges
    }

    fn nodes(&self) -> &Vec<Input> {
        &self.nodes
    }

    fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

#[graphql_object(context = Context, Scalar = RollupsGraphQLScalarValue)]
impl Notice {
    fn id(&self) -> &juniper::ID {
        &self.id
    }

    fn index(&self) -> i32 {
        self.index
    }

    fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    fn input(&self) -> &Input {
        &self.input
    }

    fn keccak(&self) -> &str {
        self.keccak.as_str()
    }

    #[graphql(
        description = "Payload in Ethereum hex binary format, starting with '0x'"
    )]
    fn payload(&self) -> &str {
        self.payload.as_str()
    }
}

#[graphql_object(context = Context, Scalar = RollupsGraphQLScalarValue)]
impl NoticeEdge {
    fn node(&self) -> &Notice {
        &self.node
    }

    fn cursor(&self) -> &String {
        &self.cursor
    }
}
implement_cursor!(NoticeEdge);

#[graphql_object(context = Context, Scalar = RollupsGraphQLScalarValue)]
impl NoticeConnection {
    fn total_count(&self) -> i32 {
        self.total_count
    }

    fn edges(&self) -> &Vec<NoticeEdge> {
        &self.edges
    }

    fn nodes(&self) -> &Vec<Notice> {
        &self.nodes
    }

    fn page_info(&self) -> &PageInfo {
        &self.page_info
    }
}

#[graphql_object(context = Context, Scalar = RollupsGraphQLScalarValue)]
impl Query {
    fn epoch(id: juniper::ID) -> FieldResult<Epoch> {
        use crate::database::{schema, DbEpoch};
        let epoch_id: i32 = id.parse::<i32>().map_err(|e| {
            super::error::Error::InvalidIdError {
                item: "epoch".to_string(),
                source: e,
            }
        })?;
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;

        let mut query = schema::epochs::dsl::epochs.into_boxed();
        query = query.filter(schema::epochs::dsl::id.eq(epoch_id));
        let query_result = query
            .load::<DbEpoch>(&conn)
            .context(super::error::DatabaseError)?;

        if let Some(db_epoch) = query_result.get(0) {
            Ok(Epoch {
                id: juniper::ID::new(db_epoch.id.to_string()),
                index: db_epoch.epoch_index,
            })
        } else {
            Err(super::error::Error::ItemNotFound {
                item_type: "epoch".to_string(),
                id: epoch_id.to_string(),
            }
            .into())
        }
    }

    fn input(id: juniper::ID) -> FieldResult<Input> {
        use crate::database::{schema, DbInput};
        let input_id = id.parse::<i32>().map_err(|e| {
            super::error::Error::InvalidIdError {
                item: "input".to_string(),
                source: e,
            }
        })?;
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        let query = schema::inputs::dsl::inputs
            .into_boxed()
            .filter(schema::inputs::dsl::id.eq(input_id));
        let db_inputs = query
            .load::<DbInput>(&conn)
            .map_err(|e| super::error::Error::DatabaseError { source: e })?;

        if let Some(db_input) = db_inputs.get(0) {
            let epoch = get_epoch_from_db(
                DbFilterType::EpochIndex(db_input.epoch_index),
                &conn,
            )?;
            Ok(Input {
                id: juniper::ID::from(db_input.id.to_string()),
                index: db_input.input_index as i32,
                epoch,
                block_number: db_input.block_number,
            })
        } else {
            Err(super::error::Error::ItemNotFound {
                item_type: "input".to_string(),
                id: input_id.to_string(),
            }
            .into())
        }
    }

    fn notice(id: juniper::ID) -> FieldResult<Notice> {
        use crate::database::{schema, DbNotice};
        let notice_id = id.parse::<i32>().map_err(|e| {
            super::error::Error::InvalidIdError {
                item: "notice".to_string(),
                source: e,
            }
        })?;
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError {
                message: e.to_string(),
            }
        })?;
        let query = schema::notices::dsl::notices
            .into_boxed()
            .filter(schema::notices::dsl::id.eq(notice_id));
        let db_notices = query
            .load::<DbNotice>(&conn)
            .map_err(|e| super::error::Error::DatabaseError { source: e })?;
        if let Some(db_notice) = db_notices.get(0) {
            let input = get_input_from_db(
                DbFilterType::EpochAndInputIndex(
                    db_notice.epoch_index,
                    db_notice.input_index,
                ),
                &conn,
            )?;
            Ok(Notice {
                id: juniper::ID::new(db_notice.id.to_string()),
                session_id: db_notice.session_id.clone(),
                index: db_notice.notice_index as i32,
                input,
                keccak: db_notice.keccak.clone(),
                payload: "0x".to_string()
                    + hex::encode(
                        db_notice.payload.as_ref().unwrap_or(&Vec::new()),
                    )
                    .as_str(),
            })
        } else {
            Err(super::error::Error::ItemNotFound {
                item_type: "notice".to_string(),
                id: notice_id.to_string(),
            }
            .into())
        }
    }

    fn epochs(
        &self,
        first: Option<i32>,
        last: Option<i32>,
        after: Option<String>,
        before: Option<String>,
    ) -> FieldResult<EpochConnection> {
        let conn = executor.context().db_pool.get()?;
        let epochs: Vec<Epoch> = get_epochs(&conn, first, last, after, before)?
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

    fn inputs(
        &self,
        first: Option<i32>,
        last: Option<i32>,
        after: Option<String>,
        before: Option<String>,
    ) -> FieldResult<InputConnection> {
        let conn = executor.context().db_pool.get()?;
        let mut inputs: Vec<Input> =
            get_inputs_by_cursor(&conn, first, last, after, before, None)?
                .into_iter()
                .map(|(_, input)| input)
                .collect();
        inputs.sort(); // sort by id, they are sorted by index
        let edges: Vec<InputEdge> = inputs
            .clone()
            .into_iter()
            .map(|input| InputEdge {
                cursor: input.id.to_string(),
                node: input,
            })
            .collect();
        let total_count = database::schema::inputs::dsl::inputs
            .count()
            .get_result::<i64>(&conn)? as i32;
        let page_info = calculate_page_info(&edges, total_count);
        Ok(InputConnection {
            page_info,
            total_count: total_count as i32,
            edges,
            nodes: inputs,
        })
    }

    fn notices(
        &self,
        first: Option<i32>,
        last: Option<i32>,
        after: Option<String>,
        before: Option<String>,
    ) -> FieldResult<NoticeConnection> {
        let conn = executor.context().db_pool.get()?;
        let notices: Vec<Notice> = get_notices_by_cursor(
            &conn, first, last, after, before, None, None,
        )?
        .into_values()
        .collect();
        let edges: Vec<NoticeEdge> = notices
            .clone()
            .into_iter()
            .map(|notice| NoticeEdge {
                cursor: notice.id.to_string(),
                node: notice,
            })
            .collect();
        let total_count = database::schema::notices::dsl::notices
            .count()
            .get_result::<i64>(&conn)? as i32;
        let page_info = calculate_page_info(&edges, total_count);
        Ok(NoticeConnection {
            page_info,
            total_count: total_count as i32,
            edges,
            nodes: notices,
        })
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
