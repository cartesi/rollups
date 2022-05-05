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
 */

/// Receive messages from data service and insert them in database
use crate::config::IndexerConfig;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rollups_data::database::{
    schema::notices::dsl::*, schema::state::dsl::*, DbNotice, Message,
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace, warn};

pub fn format_endpoint(config: &IndexerConfig) -> String {
    format!(
        "postgres://{}:{}@{}:{}/{}",
        urlencoding::encode(&config.database.postgres_user),
        urlencoding::encode(&config.database.postgres_password),
        urlencoding::encode(&config.database.postgres_hostname),
        config.database.postgres_port,
        urlencoding::encode(&config.database.postgres_db)
    )
}

/// Update current epoch index if smaller than db stored epoch index
/// Return new epoch index in database
fn update_current_epoch_index(
    conn: &PgConnection,
    new_epoch_index: i32,
) -> Result<i32, crate::error::Error> {
    let current_db_epoch_index = get_current_db_epoch(conn)?;

    if current_db_epoch_index < new_epoch_index {
        let (_, new_db_index): (String, i32) = diesel::update(
            state
                .filter(name.eq(rollups_data::database::CURRENT_EPOCH_INDEX))
                .filter(value_i32.lt(new_epoch_index)),
        )
        .set(value_i32.eq(new_epoch_index))
        .get_result(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?;
        debug!("New epoch index written to database: {}", new_db_index);
        Ok(new_db_index)
    } else {
        Ok(current_db_epoch_index)
    }
}

/// Insert notice to database if it does not exist
fn insert_notice(
    db_notice: &DbNotice,
    conn: &PgConnection,
) -> Result<(), crate::error::Error> {
    // Check if notice is already in database
    if notices
        .filter(epoch_index.eq(&db_notice.epoch_index))
        .filter(session_id.eq(&db_notice.session_id))
        .filter(input_index.eq(&db_notice.input_index))
        .filter(notice_index.eq(&db_notice.notice_index))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?
        > 0
    {
        // Notice already in the database, skip insert
        trace!("Notice session_id {} epoch_index {} input_index {} notice_index {}  already in the database",
                db_notice.session_id, db_notice.epoch_index, db_notice.input_index, db_notice.notice_index);
        return Ok(());
    }

    // Write notice to database
    // Id field is auto incremented in table on insert
    match diesel::insert_into(notices)
        .values((
            session_id.eq(&db_notice.session_id),
            epoch_index.eq(&db_notice.epoch_index),
            input_index.eq(&db_notice.input_index),
            notice_index.eq(&db_notice.notice_index),
            keccak.eq(&db_notice.keccak),
            payload.eq(&db_notice.payload),
            timestamp.eq(&db_notice.timestamp),
        ))
        .execute(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })
    {
        Ok(_) => {
            trace!("Notice session_id {} epoch_index {} input_index {} notice_index {}  written successfully",
                db_notice.session_id, db_notice.epoch_index, db_notice.input_index, db_notice.notice_index);
            Ok(())
        }
        Err(e) => {
            error!("Failed to write notice to db, details: {}", e.to_string());
            Err(e)
        }
    }
}

/// Get last known processed epoch from the database
pub fn get_current_db_epoch(
    conn: &PgConnection,
) -> Result<i32, crate::error::Error> {
    let current_epoch = state
        .filter(name.eq(rollups_data::database::CURRENT_EPOCH_INDEX))
        .select(value_i32)
        .get_result::<i32>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?;
    trace!(
        "Epoch epoch index {} read from database as current",
        current_epoch
    );
    Ok(current_epoch)
}

async fn db_loop(
    config: IndexerConfig,
    mut message_rx: mpsc::Receiver<rollups_data::database::Message>,
) -> Result<(), crate::error::Error> {
    info!("starting db loop");
    let postgres_endpoint = format_endpoint(&config);

    loop {
        tokio::select! {
            Some(response) = message_rx.recv() => {
                // Connect to database. In case of error, continue trying with increasing retry period
                let conn = {
                    let pe = postgres_endpoint.clone();
                    match tokio::task::spawn_blocking(move || {
                       rollups_data::database::connect_to_database_with_retry(&pe)
                    }).await.map_err(|e| crate::error::Error::TokioError { source: e }) {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Failed to connect to database {}, error: {}", &postgres_endpoint, e.to_string());
                            continue;
                        }
                    }
                };
                match response {
                    Message::Notice(notice) => {
                        debug!("Notice message received session_id {} epoch_index {} input_index {} notice_index {}, writing to db",
                            &notice.session_id, notice.epoch_index, notice.input_index, notice.notice_index);
                        // Spawn tokio blocking task, diesel access to db is blocking
                        let _res = tokio::task::spawn_blocking(move || {
                            if let Err(_err) = insert_notice(&notice, &conn) {
                                //ignore error, continue
                                warn!("Notice session_id {} epoch_index {} input_index {} notice_index {} is lost",
                                    &notice.session_id, &notice.epoch_index, &notice.input_index, &notice.notice_index);
                            }
                            if let Err(err) = update_current_epoch_index(&conn, notice.epoch_index) {
                                warn!("Failed to update database epoch index {}, details: {}", notice.epoch_index, err.to_string());
                            }
                        }).await;
                    }
                }
            }
        }
    }
}

/// Create and run new instance of db service
pub async fn run(
    config: IndexerConfig,
    message_rx: mpsc::Receiver<rollups_data::database::Message>,
) -> Result<(), crate::error::Error> {
    db_loop(config, message_rx).await?;
    Ok(())
}
