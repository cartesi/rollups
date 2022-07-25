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

use super::resolvers::*;
use crate::database;
use crate::database::{
    DbEpoch, DbInput, DbNotice, DbProof, DbReport, DbVoucher,
};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use juniper::{graphql_value, FieldError, FieldResult};
use snafu::ResultExt;
use tracing::warn;

/// Attribute for filtering sql queries
#[allow(dead_code)]
pub(super) enum DbFilterType {
    Id(i32),
    EpochIndex(i32),
    EpochAndInputIndex(i32, i32),
}

/// Calculate pagination info structure based on edges list
/// Uses provided total_count to calculate `has_next_page`
pub(super) fn calculate_page_info<T>(edges: &[T], total_count: i32) -> PageInfo
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

pub(super) fn get_epoch(
    conn: &PgConnection,
    id: Option<juniper::ID>,
    index: Option<i32>,
) -> FieldResult<Epoch> {
    // Either id or indexes must be provided
    if id.is_some() && index.is_some() {
        return Err(super::error::Error::InvalidParameterError {}.into());
    }

    use crate::database::schema;
    let mut query = schema::epochs::dsl::epochs.into_boxed();
    if let Some(ref id) = id {
        let epoch_id: i32 = id.parse::<i32>().map_err(|e| {
            super::error::Error::InvalidIdError {
                item: "epoch".to_string(),
                source: e,
            }
        })?;
        query = query.filter(schema::epochs::dsl::id.eq(epoch_id));
    } else if let Some(epoch_index) = index {
        query = query.filter(schema::epochs::dsl::epoch_index.eq(epoch_index));
    } else {
        return Err(super::error::Error::InvalidParameterError {}.into());
    }

    let query_result = query
        .load::<DbEpoch>(conn)
        .context(super::error::DatabaseError)?;
    if let Some(db_epoch) = query_result.get(0) {
        Ok(Epoch {
            id: juniper::ID::new(db_epoch.id.to_string()),
            index: db_epoch.epoch_index,
        })
    }
    // Error, epoch not found
    else if let Some(epoch_id) = id {
        Err(super::error::Error::ItemNotFound {
            item_type: "epoch".to_string(),
            id: epoch_id.to_string(),
        }
        .into())
    } else if let Some(epoch_index) = index {
        Err(super::error::Error::EpochNotFound { index: epoch_index }.into())
    } else {
        // Should not get here
        Err(super::error::Error::InvalidParameterError {}.into())
    }
}

pub(super) fn get_epoch_from_db(
    conn: &PgConnection,
    val: DbFilterType,
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
            id: juniper::ID::new(db_epoch.id.to_string()),
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
pub(super) fn process_db_epochs(
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
pub(super) fn get_epochs(
    conn: &PgConnection,
    pagination: Pagination,
) -> FieldResult<std::collections::BTreeMap<i32, Epoch>> {
    use crate::database::schema::epochs::dsl::*;

    let mut query = epochs.into_boxed();
    let mut query_count = epochs.into_boxed();
    let first = if let Some(first) = pagination.first {
        if first < 0 {
            return Err(FieldError::new(
                "Parameter `first` is less than 0",
                graphql_value!({ "error": "Invalid argument" }),
            ));
        }
        query = query.limit(first.into());
        query_count = query_count.limit(first.into());
        first
    } else {
        0
    };

    if let Some(after) = pagination.after {
        if let Ok(after_i32) = after.parse::<i32>() {
            query = query.filter(id.gt(after_i32));
            query_count = query_count.filter(id.gt(after_i32));
        }
    };
    if let Some(before) = pagination.before {
        if let Ok(before_i32) = before.parse::<i32>() {
            query = query.filter(id.lt(before_i32));
            query_count = query_count.filter(id.lt(before_i32));
        }
    };

    if let Some(last) = pagination.last {
        if last < 0 {
            return Err(FieldError::new(
                "Parameter `last` is less than 0",
                graphql_value!({ "error": "Invalid argument" }),
            ));
        }
        // Get count prior to slicing so that we can take last from that count
        let number_count = query_count.count().get_result::<i64>(conn)? as i32;
        let offset = if first > 0 {
            // Should not be used by user but return according to spec
            query = query.limit((std::cmp::min(first, last)).into());
            std::cmp::max(0, std::cmp::min(first - last, number_count - last))
        } else {
            std::cmp::max(0, number_count - last)
        };
        query = query.offset(offset.into());
    }

    query = query.order_by(id.asc());
    let db_epochs = query.load::<DbEpoch>(conn)?;
    process_db_epochs(db_epochs)
}

/// Get map (ordered by index) of epochs from database for every index from the list
pub(super) fn get_epochs_by_indexes(
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

// Get input by id or by epoch and input index
pub(super) fn get_input(
    conn: &PgConnection,
    id: Option<juniper::ID>,
    indexes: Option<(i32, i32)>,
) -> FieldResult<Input> {
    // Either id or indexes must be provided
    if id.is_some() && indexes.is_some() {
        return Err(super::error::Error::InvalidParameterError {}.into());
    }

    use crate::database::schema;
    let mut query = schema::inputs::dsl::inputs.into_boxed();

    if let Some(ref id) = id {
        let input_id = id.parse::<i32>().map_err(|e| {
            super::error::Error::InvalidIdError {
                item: "input".to_string(),
                source: e,
            }
        })?;

        query = query.filter(schema::inputs::dsl::id.eq(input_id));
    } else if let Some((epoch_index, input_index)) = indexes {
        query = query.filter(schema::inputs::dsl::input_index.eq(input_index));
        query = query.filter(schema::inputs::dsl::epoch_index.eq(epoch_index));
    } else {
        return Err(super::error::Error::InvalidParameterError {}.into());
    }

    let db_inputs = query
        .load::<DbInput>(conn)
        .map_err(|e| super::error::Error::DatabaseError { source: e })?;

    if let Some(db_input) = db_inputs.get(0) {
        let epoch = get_epoch_from_db(
            conn,
            DbFilterType::EpochIndex(db_input.epoch_index),
        )?;
        Ok(Input {
            id: juniper::ID::from(db_input.id.to_string()),
            index: db_input.input_index as i32,
            epoch,
            msg_sender: db_input.sender.clone(),
            timestamp: db_input.timestamp.timestamp(),
            block_number: db_input.block_number,
        })
    }
    // Return error, input not found
    else if let Some(input_id) = id {
        Err(super::error::Error::ItemNotFound {
            item_type: "input".to_string(),
            id: input_id.to_string(),
        }
        .into())
    } else if let Some((epoch_index, input_index)) = indexes {
        Err(super::error::Error::InputNotFound {
            epoch_index,
            index: input_index,
        }
        .into())
    } else {
        // Should not get here
        Err(super::error::Error::InvalidParameterError {}.into())
    }
}

/// Get single input (by id or by index) from database
pub(super) fn get_input_from_db(
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
            conn,
            DbFilterType::EpochIndex(db_input.epoch_index),
        )?;
        Ok(Input {
            id: juniper::ID::new(db_input.id.to_string()),
            index: db_input.input_index,
            epoch,
            msg_sender: db_input.sender.clone(),
            timestamp: db_input.timestamp.timestamp(),
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
pub(super) fn process_db_inputs(
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
                    msg_sender: db_input.sender,
                    timestamp: db_input.timestamp.timestamp(),
                    block_number: db_input.block_number,
                    epoch: match epochs.get(&db_input.epoch_index).ok_or_else(
                        || {
                            warn!(
                                "Unable to get epoch with index: {}",
                                db_input.epoch_index
                            );
                            Err(super::error::Error::EpochNotFound {
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
pub(super) fn get_inputs_by_cursor(
    conn: &PgConnection,
    pagination: Pagination,
    ep_index: Option<i32>,
) -> FieldResult<std::collections::BTreeMap<(i32, i32), Input>> {
    use crate::database::schema::inputs::dsl::*;
    let mut query = inputs.into_boxed();
    let mut query_count = inputs.into_boxed();

    let first = if let Some(first) = pagination.first {
        if first < 0 {
            return Err(FieldError::new(
                "Parameter `first` is less than 0",
                graphql_value!({ "error": "Invalid argument" }),
            ));
        }
        query = query.limit(first.into());
        query_count = query_count.limit(first.into());
        first
    } else {
        0
    };

    if let Some(after) = pagination.after {
        if let Ok(after_i32) = after.parse::<i32>() {
            query = query.filter(id.gt(after_i32));
            query_count = query_count.filter(id.gt(after_i32));
        }
    };
    if let Some(before) = pagination.before {
        if let Ok(before_i32) = before.parse::<i32>() {
            query = query.filter(id.lt(before_i32));
            query_count = query_count.filter(id.lt(before_i32));
        }
    };
    if let Some(ep_index) = ep_index {
        query = query.filter(epoch_index.eq(ep_index));
        query_count = query_count.filter(epoch_index.eq(ep_index));
    };

    if let Some(last) = pagination.last {
        if last < 0 {
            return Err(FieldError::new(
                "Parameter `last` is less than 0",
                graphql_value!({ "error": "Invalid argument" }),
            ));
        }
        // Get count so that we can take last from that count
        let number_count = query_count.count().get_result::<i64>(conn)? as i32;
        let offset = if first > 0 {
            // Should not be used by user but return according to spec
            query = query.limit((std::cmp::min(first, last)).into());
            std::cmp::max(0, std::cmp::min(first - last, number_count - last))
        } else {
            std::cmp::max(0, number_count - last)
        };
        query = query.offset(offset.into());
    }

    query = query.order_by(id.asc());
    let db_inputs = query.load::<DbInput>(conn)?;
    process_db_inputs(db_inputs, conn)
}

pub(super) fn get_inputs(
    conn: &PgConnection,
    pagination: Pagination,
    r#_where: Option<InputFilter>,
    epoch_index: Option<i32>,
) -> FieldResult<InputConnection> {
    let mut inputs: Vec<Input> =
        get_inputs_by_cursor(conn, pagination, epoch_index)?
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

    let total_input_count = if let Some(epoch_index) = epoch_index {
        // number of inputs in epoch
        database::schema::inputs::dsl::inputs
            .filter(
                crate::database::schema::inputs::dsl::epoch_index
                    .eq(&epoch_index),
            )
            .count()
            .get_result::<i64>(conn)? as i32
    } else {
        database::schema::inputs::dsl::inputs
            .count()
            .get_result::<i64>(conn)? as i32
    };

    let page_info = calculate_page_info(&edges, total_input_count);
    Ok(InputConnection {
        page_info,
        total_count: total_input_count as i32,
        edges,
        nodes: inputs,
    })
}

/// Get inputs from database for every index from the list
pub(super) fn get_inputs_by_indexes(
    conn: &PgConnection,
    indexes: Vec<i32>,
) -> FieldResult<std::collections::BTreeMap<(i32, i32), Input>> {
    use crate::database::schema::inputs::dsl::*;
    let mut query = inputs.into_boxed();
    query = query.filter(input_index.eq_any(indexes));
    query = query.order_by(id.asc());
    let db_inputs = query.load::<DbInput>(conn)?;
    process_db_inputs(db_inputs, conn)
}

/// Get notices from database and return ordered map of <notice id, Notice>
pub(super) fn process_db_notices(
    conn: &PgConnection,
    db_notices: Vec<DbNotice>,
) -> FieldResult<std::collections::BTreeMap<i32, Notice>> {
    //Get all inputs related to those notices
    let mut input_indexes = std::collections::HashSet::<i32>::new();
    db_notices.iter().for_each(|db_notice| {
        input_indexes.insert(db_notice.input_index);
    });
    let inputs =
        get_inputs_by_indexes(conn, input_indexes.into_iter().collect())?;

    let result: Result<
        std::collections::BTreeMap<i32, Notice>,
        super::error::Error,
    > = db_notices
        .into_iter()
        .map(|db_notice| {
            let proof: Option<Proof> = if let Some(proof_id) = db_notice.proof_id {
                get_proof_from_db(conn, proof_id).ok()
            } else {
                None
            };
            Ok((
                db_notice.id,
                Notice {
                    id: juniper::ID::new(db_notice.id.to_string()),
                    index: db_notice.notice_index as i32,
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
                    proof,
                    keccak: db_notice.keccak, // In ethereum "0x" binary format already
                    // Payload in database is in raw format, make it Ethereum hex binary format again
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

pub(super) fn get_notice(
    conn: &PgConnection,
    id: Option<juniper::ID>,
    indexes: Option<(i32, i32)>, //Option(epoch index, input index)
    notice_index: Option<i32>,
) -> FieldResult<Notice> {
    // Either id or indexes must be provided
    if id.is_some() && indexes.is_some() {
        return Err(super::error::Error::InvalidParameterError {}.into());
    }

    use crate::database::schema;
    let mut query = schema::notices::dsl::notices.into_boxed();
    if let Some(ref id) = id {
        let notice_id = id.parse::<i32>().map_err(|e| {
            super::error::Error::InvalidIdError {
                item: "notice".to_string(),
                source: e,
            }
        })?;
        query = query.filter(schema::notices::dsl::id.eq(notice_id));
    } else if let Some((epoch_index, input_index)) = indexes {
        query = query.filter(schema::notices::dsl::input_index.eq(input_index));
        query = query.filter(schema::notices::dsl::epoch_index.eq(epoch_index));
    } else {
        return Err(super::error::Error::InvalidParameterError {}.into());
    }

    if let Some(notice_index) = notice_index {
        query =
            query.filter(schema::notices::dsl::notice_index.eq(notice_index));
    }

    let db_notices = query
        .load::<DbNotice>(conn)
        .map_err(|e| super::error::Error::DatabaseError { source: e })?;
    if let Some(db_notice) = db_notices.get(0) {
        let input = get_input_from_db(
            DbFilterType::EpochAndInputIndex(
                db_notice.epoch_index,
                db_notice.input_index,
            ),
            conn,
        )?;
        let proof: Option<Proof> = if let Some(proof_id) = db_notice.proof_id {
            get_proof_from_db(conn, proof_id).ok()
        } else {
            None
        };
        Ok(Notice {
            id: juniper::ID::new(db_notice.id.to_string()),
            index: db_notice.notice_index as i32,
            input,
            proof,
            keccak: db_notice.keccak.clone(),
            payload: "0x".to_string()
                + hex::encode(
                    db_notice.payload.as_ref().unwrap_or(&Vec::new()),
                )
                .as_str(),
        })
    }
    // Notice not found, return error
    else if let Some(notice_id) = id {
        Err(super::error::Error::ItemNotFound {
            item_type: "notice".to_string(),
            id: notice_id.to_string(),
        }
        .into())
    } else if let Some((epoch_index, notice_index)) = indexes {
        Err(super::error::Error::NoticeNotFound {
            epoch_index,
            index: notice_index,
        }
        .into())
    } else {
        // Should not get here
        Err(super::error::Error::InvalidParameterError {}.into())
    }
}

/// Get notices from database and return ordered map of <notice id, Notice>
pub(super) fn get_notices_by_cursor(
    conn: &PgConnection,
    pagination: Pagination,
    epoch_index: Option<i32>,
    input_index: Option<i32>,
) -> FieldResult<std::collections::BTreeMap<i32, Notice>> {
    use crate::database::schema::notices;
    let mut query = notices::dsl::notices.into_boxed();
    let mut query_count = notices::dsl::notices.into_boxed();

    let first = if let Some(first) = pagination.first {
        if first < 0 {
            return Err(FieldError::new(
                "Parameter `first` is less than 0",
                graphql_value!({ "error": "Invalid argument" }),
            ));
        }
        query = query.limit(first.into());
        query_count = query_count.limit(first.into());
        first
    } else {
        0
    };

    if let Some(after) = pagination.after {
        if let Ok(after_i32) = after.parse::<i32>() {
            query = query.filter(notices::dsl::id.gt(after_i32));
            query_count = query_count.filter(notices::dsl::id.gt(after_i32));
        }
    };
    if let Some(before) = pagination.before {
        if let Ok(before_i32) = before.parse::<i32>() {
            query = query.filter(notices::dsl::id.lt(before_i32));
            query_count = query_count.filter(notices::dsl::id.lt(before_i32));
        }
    };
    if let Some(in_index) = input_index {
        query = query.filter(
            crate::database::schema::notices::dsl::input_index.eq(in_index),
        );
        query_count = query_count.filter(
            crate::database::schema::notices::dsl::input_index.eq(in_index),
        );
    };
    if let Some(ep_index) = epoch_index {
        query = query.filter(
            crate::database::schema::notices::dsl::epoch_index.eq(ep_index),
        );
        query_count = query_count.filter(
            crate::database::schema::notices::dsl::epoch_index.eq(ep_index),
        );
    };
    if let Some(last) = pagination.last {
        if last < 0 {
            return Err(FieldError::new(
                "Parameter `last` is less than 0",
                graphql_value!({ "error": "Invalid argument" }),
            ));
        }
        // Get count prior to slicing so that we can take last from that count
        let number_count = query_count.count().get_result::<i64>(conn)? as i32;
        let offset = if first > 0 {
            // Should not be used by user but return according to spec
            query = query.limit((std::cmp::min(first, last)).into());
            std::cmp::max(0, std::cmp::min(first - last, number_count - last))
        } else {
            std::cmp::max(0, number_count - last)
        };
        query = query.offset(offset.into());
    }
    query = query.order_by(notices::dsl::id.asc());
    let db_notices = query.load::<DbNotice>(conn)?;
    process_db_notices(conn, db_notices)
}

pub(super) fn get_notices(
    conn: &PgConnection,
    pagination: Pagination,
    r#_where: Option<NoticeFilter>,
    epoch_index: Option<i32>,
    input_index: Option<i32>,
) -> FieldResult<NoticeConnection> {
    let notices: Vec<Notice> =
        get_notices_by_cursor(conn, pagination, epoch_index, input_index)?
            .into_values()
            .collect();

    let total_count = if let Some(input_index) = input_index {
        // number of notices in input
        database::schema::notices::dsl::notices
            .filter(
                crate::database::schema::notices::dsl::input_index
                    .eq(&input_index),
            )
            .count()
            .get_result::<i64>(conn)? as i32
    } else if let Some(epoch_index) = epoch_index {
        // number of notices in epoch
        database::schema::notices::dsl::notices
            .filter(
                crate::database::schema::notices::dsl::epoch_index
                    .eq(&epoch_index),
            )
            .count()
            .get_result::<i64>(conn)? as i32
    } else {
        // total number of notices
        database::schema::notices::dsl::notices
            .count()
            .get_result::<i64>(conn)? as i32
    };
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

/// Get reports from database and return ordered map of <report id, Report>
pub(super) fn process_db_reports(
    conn: &PgConnection,
    db_reports: Vec<DbReport>,
) -> FieldResult<std::collections::BTreeMap<i32, Report>> {
    //Get all inputs related to those reports
    let mut input_indexes = std::collections::HashSet::<i32>::new();
    db_reports.iter().for_each(|db_report| {
        input_indexes.insert(db_report.input_index);
    });
    let inputs =
        get_inputs_by_indexes(conn, input_indexes.into_iter().collect())?;

    let result: Result<
        std::collections::BTreeMap<i32, Report>,
        super::error::Error,
    > = db_reports
        .into_iter()
        .map(|db_report| {
            Ok((
                db_report.id,
                Report {
                    id: juniper::ID::new(db_report.id.to_string()),
                    index: db_report.report_index as i32,
                    input: match inputs.get(&(db_report.epoch_index, db_report.input_index)).ok_or(Err(
                        super::error::Error::InputNotFound {
                            epoch_index: db_report.epoch_index,
                            index: db_report.input_index,
                        },
                    )) {
                        Ok(val) => val.clone(),
                        Err(e) => {
                            warn!("Unable to get input index={} from epoch index={} for report id={}",
                                db_report.input_index, db_report.epoch_index, db_report.id );
                            return e;
                        }
                    },
                    // Payload in database is in raw format, make it Ethereum hex binary format again
                    payload: "0x".to_string()
                        + hex::encode(
                        db_report.payload.as_ref().unwrap_or(&Vec::new()),
                    )
                        .as_str(),
                },
            ))
        })
        .collect();
    result.map_err(|e| e.into())
}

pub(super) fn get_report(
    conn: &PgConnection,
    id: Option<juniper::ID>,
    indexes: Option<(i32, i32)>, //Option(epoch index, input index)
    report_index: Option<i32>,
) -> FieldResult<Report> {
    // Either id or indexes must be provided
    if id.is_some() && indexes.is_some() {
        return Err(super::error::Error::InvalidParameterError {}.into());
    }

    use crate::database::schema;
    let mut query = schema::reports::dsl::reports.into_boxed();
    if let Some(ref id) = id {
        let report_id = id.parse::<i32>().map_err(|e| {
            super::error::Error::InvalidIdError {
                item: "report".to_string(),
                source: e,
            }
        })?;
        query = query.filter(schema::reports::dsl::id.eq(report_id));
    } else if let Some((epoch_index, input_index)) = indexes {
        query = query.filter(schema::reports::dsl::input_index.eq(input_index));
        query = query.filter(schema::reports::dsl::epoch_index.eq(epoch_index));
    } else {
        return Err(super::error::Error::InvalidParameterError {}.into());
    }

    if let Some(report_index) = report_index {
        query =
            query.filter(schema::reports::dsl::report_index.eq(report_index));
    }

    let db_reports = query
        .load::<DbReport>(conn)
        .map_err(|e| super::error::Error::DatabaseError { source: e })?;
    if let Some(db_report) = db_reports.get(0) {
        let input = get_input_from_db(
            DbFilterType::EpochAndInputIndex(
                db_report.epoch_index,
                db_report.input_index,
            ),
            conn,
        )?;
        Ok(Report {
            id: juniper::ID::new(db_report.id.to_string()),
            index: db_report.report_index as i32,
            input,
            // Payload is in raw format, make it Ethereum hex binary format
            payload: "0x".to_string()
                + hex::encode(
                    db_report.payload.as_ref().unwrap_or(&Vec::new()),
                )
                .as_str(),
        })
    }
    // Report not found, return error
    else if let Some(report_id) = id {
        Err(super::error::Error::ItemNotFound {
            item_type: "report".to_string(),
            id: report_id.to_string(),
        }
        .into())
    } else if let Some((epoch_index, report_index)) = indexes {
        Err(super::error::Error::ReportNotFound {
            epoch_index,
            index: report_index,
        }
        .into())
    } else {
        // Should not get here
        Err(super::error::Error::InvalidParameterError {}.into())
    }
}

/// Get reports from database and return ordered map of <report id, Report>
pub(super) fn get_reports_by_cursor(
    conn: &PgConnection,
    pagination: Pagination,
    epoch_index: Option<i32>,
    input_index: Option<i32>,
) -> FieldResult<std::collections::BTreeMap<i32, Report>> {
    use crate::database::schema::reports;
    let mut query = reports::dsl::reports.into_boxed();
    let mut query_count = reports::dsl::reports.into_boxed();

    let first = if let Some(first) = pagination.first {
        if first < 0 {
            return Err(FieldError::new(
                "Parameter `first` is less than 0",
                graphql_value!({ "error": "Invalid argument" }),
            ));
        }
        query = query.limit(first.into());
        query_count = query_count.limit(first.into());
        first
    } else {
        0
    };

    if let Some(after) = pagination.after {
        if let Ok(after_i32) = after.parse::<i32>() {
            query = query.filter(reports::dsl::id.gt(after_i32));
            query_count = query_count.filter(reports::dsl::id.gt(after_i32));
        }
    };
    if let Some(before) = pagination.before {
        if let Ok(before_i32) = before.parse::<i32>() {
            query = query.filter(reports::dsl::id.lt(before_i32));
            query_count = query_count.filter(reports::dsl::id.lt(before_i32));
        }
    };
    if let Some(in_index) = input_index {
        query = query.filter(
            crate::database::schema::reports::dsl::input_index.eq(in_index),
        );
        query_count = query_count.filter(
            crate::database::schema::reports::dsl::input_index.eq(in_index),
        );
    };
    if let Some(ep_index) = epoch_index {
        query = query.filter(
            crate::database::schema::reports::dsl::epoch_index.eq(ep_index),
        );
        query_count = query_count.filter(
            crate::database::schema::reports::dsl::epoch_index.eq(ep_index),
        );
    };
    if let Some(last) = pagination.last {
        if last < 0 {
            return Err(FieldError::new(
                "Parameter `last` is less than 0",
                graphql_value!({ "error": "Invalid argument" }),
            ));
        }
        // Get count prior to slicing so that we can take last from that count
        let number_count = query_count.count().get_result::<i64>(conn)? as i32;
        let offset = if first > 0 {
            // Should not be used by user but return according to spec
            query = query.limit((std::cmp::min(first, last)).into());
            std::cmp::max(0, std::cmp::min(first - last, number_count - last))
        } else {
            std::cmp::max(0, number_count - last)
        };
        query = query.offset(offset.into());
    }
    query = query.order_by(reports::dsl::id.asc());
    let db_reports = query.load::<DbReport>(conn)?;
    process_db_reports(conn, db_reports)
}

pub(super) fn get_reports(
    conn: &PgConnection,
    pagination: Pagination,
    r#_where: Option<ReportFilter>,
    epoch_index: Option<i32>,
    input_index: Option<i32>,
) -> FieldResult<ReportConnection> {
    let reports: Vec<Report> =
        get_reports_by_cursor(conn, pagination, epoch_index, input_index)?
            .into_values()
            .collect();

    let total_count = if let Some(input_index) = input_index {
        // number of reports in input
        database::schema::reports::dsl::reports
            .filter(
                crate::database::schema::reports::dsl::input_index
                    .eq(&input_index),
            )
            .count()
            .get_result::<i64>(conn)? as i32
    } else if let Some(epoch_index) = epoch_index {
        // number of reports in epoch
        database::schema::reports::dsl::reports
            .filter(
                crate::database::schema::reports::dsl::epoch_index
                    .eq(&epoch_index),
            )
            .count()
            .get_result::<i64>(conn)? as i32
    } else {
        // total number of reports
        database::schema::reports::dsl::reports
            .count()
            .get_result::<i64>(conn)? as i32
    };
    let edges: Vec<ReportEdge> = reports
        .clone()
        .into_iter()
        .map(|report| ReportEdge {
            cursor: report.id.to_string(),
            node: report,
        })
        .collect();
    let page_info = calculate_page_info(&edges, total_count);
    Ok(ReportConnection {
        page_info,
        total_count: total_count as i32,
        edges,
        nodes: reports,
    })
}

/// Get single input (by id or by index) from database
pub(super) fn get_proof_from_db(
    conn: &PgConnection,
    proof_id: i32,
) -> Result<Proof, crate::graphql::error::Error> {
    use crate::database::schema::proofs::dsl::*;
    let mut query = proofs.into_boxed();
    query = query.filter(crate::database::schema::proofs::dsl::id.eq(proof_id));

    let query_result = query
        .load::<DbProof>(conn)
        .context(super::error::DatabaseError)?;
    if let Some(db_proof) = query_result.get(0) {
        Ok(Proof {
            output_hashes_root_hash: db_proof.output_hashes_root_hash.clone(),
            vouchers_epoch_root_hash: db_proof.vouchers_epoch_root_hash.clone(),
            notices_epoch_root_hash: db_proof.notices_epoch_root_hash.clone(),
            machine_state_hash: db_proof.machine_state_hash.clone(),
            keccak_in_hashes_siblings: db_proof
                .keccak_in_hashes_siblings
                .clone(),
            output_hashes_in_epoch_siblings: db_proof
                .output_hashes_in_epoch_siblings
                .clone(),
        })
    } else {
        Err(super::error::Error::ItemNotFound {
            item_type: "proof".to_string(),
            id: proof_id.to_string(),
        })
    }
}

pub(super) fn get_voucher(
    conn: &PgConnection,
    id: Option<juniper::ID>,
    indexes: Option<(i32, i32)>, //Opetion(epoch index, input index)
    voucher_index: Option<i32>,
) -> FieldResult<Voucher> {
    // Either id or indexes must be provided
    if id.is_some() && indexes.is_some() {
        return Err(super::error::Error::InvalidParameterError {}.into());
    }

    use crate::database::schema;
    let mut query = schema::vouchers::dsl::vouchers.into_boxed();
    if let Some(ref id) = id {
        let voucher_id = id.parse::<i32>().map_err(|e| {
            super::error::Error::InvalidIdError {
                item: "voucher".to_string(),
                source: e,
            }
        })?;
        query = query.filter(schema::vouchers::dsl::id.eq(voucher_id));
    } else if let Some((epoch_index, input_index)) = indexes {
        query =
            query.filter(schema::vouchers::dsl::input_index.eq(input_index));
        query =
            query.filter(schema::vouchers::dsl::epoch_index.eq(epoch_index));
    } else {
        return Err(super::error::Error::InvalidParameterError {}.into());
    }

    if let Some(voucher_index) = voucher_index {
        query = query
            .filter(schema::vouchers::dsl::voucher_index.eq(voucher_index));
    }

    let db_vouchers = query
        .load::<DbVoucher>(conn)
        .map_err(|e| super::error::Error::DatabaseError { source: e })?;
    if let Some(db_voucher) = db_vouchers.get(0) {
        let input = get_input_from_db(
            DbFilterType::EpochAndInputIndex(
                db_voucher.epoch_index,
                db_voucher.input_index,
            ),
            conn,
        )?;
        let proof: Option<Proof> = if let Some(proof_id) = db_voucher.proof_id {
            get_proof_from_db(conn, proof_id).ok()
        } else {
            None
        };
        Ok(Voucher {
            id: juniper::ID::new(db_voucher.id.to_string()),
            index: db_voucher.voucher_index as i32,
            input,
            proof,
            destination: db_voucher.destination.clone(),
            payload: "0x".to_string()
                + hex::encode(
                    db_voucher.payload.as_ref().unwrap_or(&Vec::new()),
                )
                .as_str(),
        })
    }
    // Notice not found, return error
    else if let Some(notice_id) = id {
        Err(super::error::Error::ItemNotFound {
            item_type: "notice".to_string(),
            id: notice_id.to_string(),
        }
        .into())
    } else if let Some((epoch_index, notice_index)) = indexes {
        Err(super::error::Error::VoucherNotFound {
            epoch_index,
            index: notice_index,
        }
        .into())
    } else {
        // Should not get here
        Err(super::error::Error::InvalidParameterError {}.into())
    }
}

/// Get vouchers from database and return ordered map of <notice id, Voucher>
pub(super) fn process_db_vouchers(
    conn: &PgConnection,
    db_vouchers: Vec<DbVoucher>,
) -> FieldResult<std::collections::BTreeMap<i32, Voucher>> {
    //Get all inputs related to those vouchers
    let mut input_indexes = std::collections::HashSet::<i32>::new();
    db_vouchers.iter().for_each(|db_voucher| {
        input_indexes.insert(db_voucher.input_index);
    });
    let inputs =
        get_inputs_by_indexes(conn, input_indexes.into_iter().collect())?;

    let result: Result<
        std::collections::BTreeMap<i32, Voucher>,
        super::error::Error,
    > = db_vouchers
        .into_iter()
        .map(|db_voucher| {
            let proof: Option<Proof> = if let Some(proof_id) = db_voucher.proof_id {
                get_proof_from_db(conn, proof_id).ok()
            } else {
                None
            };
            Ok((
                db_voucher.id,
                Voucher {
                    id: juniper::ID::new(db_voucher.id.to_string()),
                    index: db_voucher.voucher_index as i32,
                    input: match inputs.get(&(db_voucher.epoch_index, db_voucher.input_index)).ok_or(Err(
                        super::error::Error::InputNotFound {
                            epoch_index: db_voucher.epoch_index,
                            index: db_voucher.input_index,
                        },
                    )) {
                        Ok(val) => val.clone(),
                        Err(e) => {
                            warn!("Unable to get input index={} from epoch index={} for voucher id={}",
                                db_voucher.input_index, db_voucher.epoch_index, db_voucher.id );
                            return e;
                        }
                    },
                    proof,
                    destination: db_voucher.destination.clone(),
                    // Payload in database is in raw format, make it Ethereum hex binary format again
                    payload: "0x".to_string()
                        + hex::encode(
                        db_voucher.payload.as_ref().unwrap_or(&Vec::new()),
                    )
                        .as_str(),
                },
            ))
        })
        .collect();
    result.map_err(|e| e.into())
}

/// Get vouchers from database and return ordered map of <voucher id, Voucher>
pub(super) fn get_vouchers_by_cursor(
    conn: &PgConnection,
    pagination: Pagination,
    epoch_index: Option<i32>,
    input_index: Option<i32>,
) -> FieldResult<std::collections::BTreeMap<i32, Voucher>> {
    use crate::database::schema::vouchers;
    let mut query = vouchers::dsl::vouchers.into_boxed();
    let mut query_count = vouchers::dsl::vouchers.into_boxed();

    let first = if let Some(first) = pagination.first {
        if first < 0 {
            return Err(FieldError::new(
                "Parameter `first` is less than 0",
                graphql_value!({ "error": "Invalid argument" }),
            ));
        }
        query = query.limit(first.into());
        query_count = query_count.limit(first.into());
        first
    } else {
        0
    };

    if let Some(after) = pagination.after {
        if let Ok(after_i32) = after.parse::<i32>() {
            query = query.filter(vouchers::dsl::id.gt(after_i32));
            query_count = query_count.filter(vouchers::dsl::id.gt(after_i32));
        }
    };
    if let Some(before) = pagination.before {
        if let Ok(before_i32) = before.parse::<i32>() {
            query = query.filter(vouchers::dsl::id.lt(before_i32));
            query_count = query_count.filter(vouchers::dsl::id.lt(before_i32));
        }
    };
    if let Some(in_index) = input_index {
        query = query.filter(
            crate::database::schema::vouchers::dsl::input_index.eq(in_index),
        );
        query_count = query_count.filter(
            crate::database::schema::vouchers::dsl::input_index.eq(in_index),
        );
    };
    if let Some(ep_index) = epoch_index {
        query = query.filter(
            crate::database::schema::vouchers::dsl::epoch_index.eq(ep_index),
        );
        query_count = query_count.filter(
            crate::database::schema::vouchers::dsl::epoch_index.eq(ep_index),
        );
    };
    if let Some(last) = pagination.last {
        if last < 0 {
            return Err(FieldError::new(
                "Parameter `last` is less than 0",
                graphql_value!({ "error": "Invalid argument" }),
            ));
        }
        // Get count prior to slicing so that we can take last from that count
        let number_count = query_count.count().get_result::<i64>(conn)? as i32;
        let offset = if first > 0 {
            // Should not be used by user but return according to spec
            query = query.limit((std::cmp::min(first, last)).into());
            std::cmp::max(0, std::cmp::min(first - last, number_count - last))
        } else {
            std::cmp::max(0, number_count - last)
        };
        query = query.offset(offset.into());
    }
    query = query.order_by(vouchers::dsl::id.asc());
    let db_vouchers = query.load::<DbVoucher>(conn)?;
    process_db_vouchers(conn, db_vouchers)
}

pub(super) fn get_vouchers(
    conn: &PgConnection,
    pagination: Pagination,
    r#_where: Option<VoucherFilter>,
    epoch_index: Option<i32>,
    input_index: Option<i32>,
) -> FieldResult<VoucherConnection> {
    let vouchers: Vec<Voucher> =
        get_vouchers_by_cursor(conn, pagination, epoch_index, input_index)?
            .into_values()
            .collect();

    let total_count = if let Some(input_index) = input_index {
        // number of vouchers in input
        database::schema::vouchers::dsl::vouchers
            .filter(
                crate::database::schema::vouchers::dsl::input_index
                    .eq(&input_index),
            )
            .count()
            .get_result::<i64>(conn)? as i32
    } else if let Some(epoch_index) = epoch_index {
        // number of vouchers in epoch
        database::schema::vouchers::dsl::vouchers
            .filter(
                crate::database::schema::vouchers::dsl::epoch_index
                    .eq(&epoch_index),
            )
            .count()
            .get_result::<i64>(conn)? as i32
    } else {
        // total number of vouchers
        database::schema::vouchers::dsl::vouchers
            .count()
            .get_result::<i64>(conn)? as i32
    };
    let edges: Vec<VoucherEdge> = vouchers
        .clone()
        .into_iter()
        .map(|voucher| VoucherEdge {
            cursor: voucher.id.to_string(),
            node: voucher,
        })
        .collect();
    let page_info = calculate_page_info(&edges, total_count);
    Ok(VoucherConnection {
        page_info,
        total_count: total_count as i32,
        edges,
        nodes: vouchers,
    })
}
