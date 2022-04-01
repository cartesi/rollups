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

use crate::config::IndexerConfig;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use rollups_data::database::{schema::notices::dsl::*, DbNotice, Message};
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace, warn};

// Insert notice to database if it not exists
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
    match diesel::insert_into(notices)
        .values(db_notice)
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

async fn db_loop(
    config: IndexerConfig,
    mut message_rx: mpsc::Receiver<rollups_data::database::Message>,
) -> Result<(), crate::error::Error> {
    info!("starting db loop");
    let mut connection_manager: ConnectionManager<PgConnection> =
        ConnectionManager::new(&config.postgres_endpoint);

    loop {
        tokio::select! {
            Some(response) = message_rx.recv() => {
                // Connect to database. In case of error, continue trying with increasing retry period
                let mut conn = rollups_data::database::connect_to_database_with_retry(&mut connection_manager).await;
                match response {
                    Message::Notice(notice) => {
                        debug!("Notice message received session_id {} epoch_index {} input_index {} notice_index {}, writing to db",
                            &notice.session_id, notice.epoch_index, notice.input_index, notice.notice_index);
                        // Spawn tokio blocking task, diesel access to db is blocking
                        let _res = tokio::task::spawn_blocking(move || {
                            if let Err(_err) = insert_notice(&notice, &mut conn) {
                                //ignore error, continue
                                warn!("Notice session_id {} epoch_index {} input_index {} notice_index {} is lost",
                                    &notice.session_id, &notice.epoch_index, &notice.input_index, &notice.notice_index);
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
