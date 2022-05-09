use juniper::graphql_object;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use std::sync::Arc;

pub use super::schema::*;

use juniper::FieldResult;

pub struct Context {
    // Connection is not thread safe to share between threads, we use connection pool
    pub db_pool: Arc<Pool<ConnectionManager<PgConnection>>>,
}
impl juniper::Context for Context {}

pub struct Query;

#[graphql_object(context = Context, Scalar = juniper::DefaultScalarValue)]
impl Query {
    fn notice(notice_id: String) -> FieldResult<Notice> {
        let notice_id = notice_id
            .parse::<i32>()
            .map_err(|e| super::error::Error::InvalidIdError { source: e })?;
        use crate::database::{schema::notices::dsl::*, DbNotice};
        let conn = executor.context().db_pool.get().map_err(|e| {
            super::error::Error::DatabasePoolConnectionError { message: e.to_string() }
        })?;
        let query = notices.into_boxed().filter(id.eq(notice_id));
        let db_notices = query.load::<DbNotice>(&conn).map_err(|e| {
            super::error::Error::DatabaseError { source: e }
        })?;

        if let Some(db_notice) = db_notices.iter().nth(0) {
            Ok(Notice {
                id: db_notice.id,
                session_id: db_notice.session_id.clone(),
                epoch_index: db_notice.epoch_index as i32,
                input_index: db_notice.input_index as i32,
                notice_index: db_notice.notice_index as i32,
                keccak: db_notice.keccak.clone(),
                payload: "0x".to_string()
                    + hex::encode(
                        db_notice.payload.as_ref().unwrap_or(&Vec::new()),
                    )
                    .as_str(),
            })
        } else {
            Err(super::error::Error::ItemNotFound { id: notice_id.to_string() }.into())
        }
    }

    fn notices(
        notice_keys: Option<NoticeKeys>,
        first: Option<i32>,
        last: Option<i32>,
        after: Option<String>,
        before: Option<String>,
    ) -> FieldResult<NoticeConnection> {
        use crate::database::{schema::notices::dsl::*, DbNotice};
        // note: graphql does not have native Int64, so it is only 32 bit integer

        let conn = executor.context().db_pool.get()?;
        // Form database selection command based on graphql query parameters
        let mut query = notices.into_boxed();
        let mut total_count_query = notices.into_boxed();
        if let Some(notice_keys) = notice_keys {
            if let Some(_session_id) = notice_keys.session_id {
                query = query.filter(session_id.eq(_session_id.clone()));
                total_count_query =
                    total_count_query.filter(session_id.eq(_session_id));
            };

            if let Some(index) = notice_keys.epoch_index {
                query = query.filter(epoch_index.eq(index as i32));
                total_count_query =
                    total_count_query.filter(epoch_index.eq(index as i32));
            };

            if let Some(index) = notice_keys.input_index {
                query = query.filter(input_index.eq(index as i32));
                total_count_query =
                    total_count_query.filter(input_index.eq(index as i32));
            };

            if let Some(index) = notice_keys.notice_index {
                query = query.filter(notice_index.eq(index as i32));
                total_count_query =
                    total_count_query.filter(notice_index.eq(index as i32));
            };
        }

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
        // Retrieve data from database
        let result = query.load::<DbNotice>(&conn)?;

        let edges: Vec<NoticeEdge> = result
            .iter()
            .map(|db_notice| NoticeEdge {
                node: Notice {
                    id: db_notice.id,
                    session_id: db_notice.session_id.clone(),
                    epoch_index: db_notice.epoch_index as i32,
                    input_index: db_notice.input_index as i32,
                    notice_index: db_notice.notice_index as i32,
                    keccak: db_notice.keccak.clone(),
                    payload: "0x".to_string()
                        + hex::encode(
                            db_notice.payload.as_ref().unwrap_or(&Vec::new()),
                        )
                        .as_str(),
                },
                cursor: db_notice.id.to_string(),
            })
            .collect();

        let total_count =
            total_count_query.count().get_result::<i64>(&conn)? as i32;

        let start_cursor = match edges.iter().nth(0) {
            Some(edge) => edge.cursor.clone(),
            _ => String::from(""),
        };
        let end_cursor = match edges.iter().last() {
            Some(edge) => edge.cursor.clone(),
            _ => String::from(""),
        };

        let has_previous_page = match edges.iter().nth(0) {
            Some(val) => val.cursor.parse::<i32>().unwrap_or_default() > 1,
            None => false,
        };

        let has_next_page = match edges.iter().last() {
            Some(val) => {
                val.cursor.parse::<i32>().unwrap_or_default() < total_count
            }
            None => false,
        };

        Ok(NoticeConnection {
            page_info: PageInfo {
                has_previous_page,
                has_next_page,
                start_cursor,
                end_cursor,
            },
            total_count: total_count as i32,
            edges,
            nodes: Vec::new(),
        })
    }
}
