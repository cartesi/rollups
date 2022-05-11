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
use rollups_data::database::{schema, DbInput, DbNotice, Message};
use snafu::ResultExt;
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

pub enum EpochIndexType {
    Notice,
    Input,
}

/// Update current epoch index if smaller than db stored epoch index
/// Return new epoch index in database
fn update_current_epoch_index(
    conn: &PgConnection,
    new_epoch_index: i32,
    epoch_index_type: EpochIndexType,
) -> Result<i32, crate::error::Error> {
    use schema::state::dsl::*;

    let field = match epoch_index_type {
        EpochIndexType::Notice => {
            rollups_data::database::CURRENT_NOTICE_EPOCH_INDEX
        }
        EpochIndexType::Input => {
            rollups_data::database::CURRENT_INPUT_EPOCH_INDEX
        }
    };

    let current_db_epoch_index = get_current_db_epoch(conn, epoch_index_type)?;

    if current_db_epoch_index < new_epoch_index {
        let (_, new_db_index): (String, i32) = diesel::update(
            state
                .filter(name.eq(field))
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
    use schema::notices::dsl::*;
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
            trace!("Notice session_id {} epoch_index {} input_index {} notice_index {}  written successfully to db",
                db_notice.session_id, db_notice.epoch_index, db_notice.input_index, db_notice.notice_index);
            Ok(())
        }
        Err(e) => {
            error!("Failed to write notice to db, details: {}", e.to_string());
            Err(e)
        }
    }
}

/// Insert epoch to database if it does not exist
fn insert_epoch(
    index: i32,
    conn: &PgConnection,
) -> Result<(), crate::error::Error> {
    use schema::epochs::dsl::*;
    // Check if epoch is already in database
    if epochs
        .filter(epoch_index.eq(&index))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?
        > 0
    {
        // Epoch already in the database, skip insert
        trace!("Epoch epoch_index {} already in the database", index);
        return Ok(());
    }

    // Write epoch to database
    // Id field is auto incremented in table on insert
    match diesel::insert_into(epochs)
        .values(epoch_index.eq(&index))
        .execute(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })
    {
        Ok(_) => {
            trace!("Epoch epoch_index {} written successfully to db", index);
            Ok(())
        }
        Err(e) => {
            error!("Failed to write epoch to db, details: {}", e.to_string());
            Err(e)
        }
    }
}

/// Insert input to database if it does not exist
fn insert_input(
    db_input: &DbInput,
    conn: &PgConnection,
) -> Result<(), crate::error::Error> {
    use schema::inputs::dsl::*;
    // Check if input is already in database
    if inputs
        .filter(epoch_index.eq(&db_input.epoch_index))
        .filter(input_index.eq(&db_input.input_index))
        .filter(sender.eq(&db_input.sender))
        .filter(payload.eq(&db_input.payload))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?
        > 0
    {
        // Input already in the database, skip insert
        trace!("Input epoch_index {} input_index {} sender {} payload {:?}  already in the database",
                db_input.epoch_index, db_input.epoch_index, db_input.sender, db_input.payload);
        return Ok(());
    }

    // Write input to database
    // Id field is auto incremented in table on insert
    match diesel::insert_into(inputs)
        .values((
            epoch_index.eq(&db_input.epoch_index),
            input_index.eq(&db_input.input_index),
            sender.eq(&db_input.sender),
            block_number.eq(&db_input.block_number),
            payload.eq(&db_input.payload),
            timestamp.eq(&db_input.timestamp),
        ))
        .execute(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })
    {
        Ok(_) => {
            trace!("Input epoch_index {} input_index {} sender {} payload {:?} written successfully to db",
                db_input.epoch_index, db_input.epoch_index, db_input.sender, db_input.payload);
            Ok(())
        }
        Err(e) => {
            error!("Failed to write input to db, details: {}", e.to_string());
            Err(e)
        }
    }
}

/// Get last known processed notice epoch from the database
pub fn get_current_db_epoch(
    conn: &PgConnection,
    epoch_index_type: EpochIndexType,
) -> Result<i32, crate::error::Error> {
    use schema::state::dsl::*;

    let field = match epoch_index_type {
        EpochIndexType::Notice => {
            rollups_data::database::CURRENT_NOTICE_EPOCH_INDEX
        }
        EpochIndexType::Input => {
            rollups_data::database::CURRENT_INPUT_EPOCH_INDEX
        }
    };

    let current_epoch = state
        .filter(name.eq(field))
        .select(value_i32)
        .get_result::<i32>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?;
    trace!(
        "Epoch epoch index {} read from database as current",
        current_epoch
    );
    Ok(current_epoch)
}

pub async fn get_current_db_epoch_async(
    postgres_endpoint: &str,
    epoch_index_type: EpochIndexType,
) -> Result<i32, crate::error::Error> {
    let conn = rollups_data::database::connect_to_database_with_retry_async(
        postgres_endpoint.into(),
    )
    .await
    .context(crate::error::TokioError)?;

    tokio::task::spawn_blocking(move || {
        Ok(
            crate::db_service::get_current_db_epoch(&conn, epoch_index_type)?
                as i32,
        )
    })
    .await
    .context(crate::error::TokioError)?
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
                            if let Err(err) = update_current_epoch_index(&conn, notice.epoch_index, EpochIndexType::Notice) {
                                warn!("Failed to update notice database epoch index {}, details: {}", notice.epoch_index, err.to_string());
                            }
                        }).await;
                    }
                    Message::Input(input) => {
                        debug!("Input message received id {} input_index {} epoch_index {} sender {} block_number {} timestamp {}",
                            &input.id, &input.input_index, &input.epoch_index, &input.sender, &input.block_number, &input.timestamp);
                        // Spawn tokio blocking task, diesel access to db is blocking
                        let _res = tokio::task::spawn_blocking(move || {
                            // Insert epoch if not in database
                            if let Err(_err) = insert_epoch(input.epoch_index, &conn) {
                                //ignore error, continue
                                warn!("Epoch index {} is lost", &input.epoch_index);
                            }
                            if let Err(_err) = insert_input(&input, &conn) {
                                //ignore error, continue
                                warn!("Input id {} input_index {} epoch_index {} sender {} block_number {} timestamp {} is lost",
                                    &input.id, &input.input_index, &input.epoch_index, &input.sender, &input.block_number, &input.timestamp);
                            }
                            if let Err(err) = update_current_epoch_index(&conn, input.epoch_index, EpochIndexType::Input) {
                                warn!("Failed to update input database epoch index {}, details: {}", input.epoch_index, err.to_string());
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
