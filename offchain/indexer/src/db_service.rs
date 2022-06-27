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
use crate::config::{IndexerConfig, PostgresConfig};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rollups_data::database::{
    schema, DbInput, DbNotice, DbProof, DbReport, DbVoucher, Message,
};
use snafu::ResultExt;
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace, warn};

pub enum EpochIndexType {
    Notice,
    Input,
    Report,
    Voucher,
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
        EpochIndexType::Voucher => {
            rollups_data::database::CURRENT_VOUCHER_EPOCH_INDEX
        }
        EpochIndexType::Report => {
            rollups_data::database::CURRENT_REPORT_EPOCH_INDEX
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

/// Insert proof to database if it does not exist
pub(crate) fn insert_proof(
    db_proof: &DbProof,
    conn: &PgConnection,
) -> Result<Option<i32>, crate::error::Error> {
    use schema::proofs::dsl::*;
    // Check if proof is already in database
    if proofs
        .filter(output_hashes_root_hash.eq(&db_proof.output_hashes_root_hash))
        .filter(vouchers_epoch_root_hash.eq(&db_proof.vouchers_epoch_root_hash))
        .filter(notices_epoch_root_hash.eq(&db_proof.notices_epoch_root_hash))
        .filter(machine_state_hash.eq(&db_proof.machine_state_hash))
        .filter(
            keccak_in_hashes_siblings.eq(&db_proof.keccak_in_hashes_siblings),
        )
        .filter(
            output_hashes_in_epoch_siblings
                .eq(&db_proof.output_hashes_in_epoch_siblings),
        )
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?
        > 0
    {
        // Proof already in the database, skip insert, get proof id
        let proof_id = proofs
            .filter(
                output_hashes_root_hash.eq(&db_proof.output_hashes_root_hash),
            )
            .filter(
                vouchers_epoch_root_hash.eq(&db_proof.vouchers_epoch_root_hash),
            )
            .filter(
                notices_epoch_root_hash.eq(&db_proof.notices_epoch_root_hash),
            )
            .filter(machine_state_hash.eq(&db_proof.machine_state_hash))
            .filter(
                keccak_in_hashes_siblings
                    .eq(&db_proof.keccak_in_hashes_siblings),
            )
            .filter(
                output_hashes_in_epoch_siblings
                    .eq(&db_proof.output_hashes_in_epoch_siblings),
            )
            .select(id)
            .get_result::<i32>(conn)
            .map_err(|e| crate::error::Error::DieselError { source: e })?;
        trace!("Proof output_hashes_root_hash {} vouchers_epoch_root_hash {} notices_epoch_root_hash {} machine_state_hash {}  already in the database with id={}",
                db_proof.output_hashes_root_hash, db_proof.vouchers_epoch_root_hash, db_proof.notices_epoch_root_hash, db_proof.machine_state_hash, proof_id);
        return Ok(Some(proof_id));
    }

    // Write notice to database
    // Id field is auto incremented in table on insert
    match diesel::insert_into(proofs)
        .values((
            output_hashes_root_hash.eq(&db_proof.output_hashes_root_hash),
            vouchers_epoch_root_hash.eq(&db_proof.vouchers_epoch_root_hash),
            notices_epoch_root_hash.eq(&db_proof.notices_epoch_root_hash),
            machine_state_hash.eq(&db_proof.machine_state_hash),
            keccak_in_hashes_siblings.eq(&db_proof.keccak_in_hashes_siblings),
            output_hashes_in_epoch_siblings
                .eq(&db_proof.output_hashes_in_epoch_siblings),
        ))
        .get_result::<DbProof>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })
    {
        Ok(proof) => {
            info!("Proof output_hashes_root_hash {} vouchers_epoch_root_hash {} notices_epoch_root_hash {} machine_state_hash {} inserted to database, new id ={}",
                db_proof.output_hashes_root_hash, db_proof.vouchers_epoch_root_hash, db_proof.notices_epoch_root_hash, db_proof.machine_state_hash, proof.id);
            Ok(Some(proof.id))
        }
        Err(e) => {
            error!("Failed to write proof to db, details: {}", e.to_string());
            Err(e)
        }
    }
}

/// Update notice in the database
fn update_notice(
    db_notice: &DbNotice,
    conn: &PgConnection,
) -> Result<(), crate::error::Error> {
    use schema::notices::dsl::*;
    let changed_rows = diesel::update(notices.filter(id.eq(&db_notice.id)))
        .set((
            session_id.eq(&db_notice.session_id),
            epoch_index.eq(db_notice.epoch_index),
            input_index.eq(&db_notice.input_index),
            notice_index.eq(&db_notice.notice_index),
            proof_id.eq(&db_notice.proof_id),
            keccak.eq(&db_notice.keccak),
            payload.eq(&db_notice.payload),
        ))
        .execute(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?;
    info!(
        "Notice with id {} updated, number of affected records {}",
        db_notice.id, changed_rows
    );
    Ok(())
}

/// Insert notice to database if it does not exist
pub(crate) fn insert_notice(
    db_notice: &DbNotice,
    conn: &PgConnection,
) -> Result<Option<i32>, crate::error::Error> {
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
        return Ok(None);
    }

    // Write notice to database
    // Id field is auto incremented in table on insert
    match diesel::insert_into(notices)
        .values((
            session_id.eq(&db_notice.session_id),
            epoch_index.eq(&db_notice.epoch_index),
            input_index.eq(&db_notice.input_index),
            notice_index.eq(&db_notice.notice_index),
            proof_id.eq(&db_notice.proof_id),
            keccak.eq(&db_notice.keccak),
            payload.eq(&db_notice.payload),
        ))
        .get_result::<DbNotice>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })
    {
        Ok(notice) => {
            info!("Notice session_id {} epoch_index {} input_index {} notice_index {} written successfully to, assigned id={}",
                db_notice.session_id, db_notice.epoch_index, db_notice.input_index, db_notice.notice_index, notice.id);
            Ok(Some(notice.id))
        }
        Err(e) => {
            error!("Failed to write notice to db, details: {}", e.to_string());
            Err(e)
        }
    }
}

/// Update notice in the database
fn update_voucher(
    db_voucher: &DbVoucher,
    conn: &PgConnection,
) -> Result<(), crate::error::Error> {
    use schema::vouchers::dsl::*;
    diesel::update(vouchers.filter(id.eq(&db_voucher.id)))
        .set((
            epoch_index.eq(db_voucher.epoch_index),
            input_index.eq(&db_voucher.input_index),
            voucher_index.eq(&db_voucher.voucher_index),
            proof_id.eq(&db_voucher.proof_id),
            destination.eq(&db_voucher.destination),
            payload.eq(&db_voucher.payload),
        ))
        .execute(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?;
    Ok(())
}

/// Insert voucher to database if it does not exist
pub(crate) fn insert_voucher(
    db_voucher: &DbVoucher,
    conn: &PgConnection,
) -> Result<Option<i32>, crate::error::Error> {
    use schema::vouchers::dsl::*;
    // Check if voucher is already in database
    if vouchers
        .filter(epoch_index.eq(&db_voucher.epoch_index))
        .filter(input_index.eq(&db_voucher.input_index))
        .filter(voucher_index.eq(&db_voucher.voucher_index))
        .filter(destination.eq(&db_voucher.destination))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?
        > 0
    {
        // Voucher is already in the database, skip insert
        trace!("Voucher epoch_index {} input_index {} notice_index {}  already in the database",
                db_voucher.epoch_index, db_voucher.input_index, db_voucher.voucher_index);
        return Ok(None);
    }

    // Write voucher to database
    // Id field is auto incremented in table on insert
    match diesel::insert_into(vouchers)
        .values((
            epoch_index.eq(&db_voucher.epoch_index),
            input_index.eq(&db_voucher.input_index),
            voucher_index.eq(&db_voucher.voucher_index),
            proof_id.eq(&db_voucher.proof_id),
            destination.eq(&db_voucher.destination),
            payload.eq(&db_voucher.payload),
        ))
        .get_result::<DbVoucher>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })
    {
        Ok(new_voucher) => {
            trace!("Voucher epoch_index {} input_index {} voucher_index {}  written successfully to db with id {}",
                db_voucher.epoch_index, db_voucher.input_index, db_voucher.voucher_index, new_voucher.id);
            Ok(Some(new_voucher.id))
        }
        Err(e) => {
            error!("Failed to write voucher to db, details: {}", e.to_string());
            Err(e)
        }
    }
}

/// Insert report to database if it does not exist
pub(crate) fn insert_report(
    db_report: &DbReport,
    conn: &PgConnection,
) -> Result<(), crate::error::Error> {
    use schema::reports::dsl::*;
    // Check if report is already in database
    if reports
        .filter(epoch_index.eq(&db_report.epoch_index))
        .filter(input_index.eq(&db_report.input_index))
        .filter(report_index.eq(&db_report.report_index))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })?
        > 0
    {
        // Report already in the database, skip insert
        trace!("Report epoch_index {} input_index {} report_index {} already in the database",
                db_report.epoch_index, db_report.input_index, db_report.report_index);
        return Ok(());
    }

    // Write report to database
    // Id field is auto incremented in table on insert
    match diesel::insert_into(reports)
        .values((
            epoch_index.eq(&db_report.epoch_index),
            input_index.eq(&db_report.input_index),
            report_index.eq(&db_report.report_index),
            payload.eq(&db_report.payload),
        ))
        .execute(conn)
        .map_err(|e| crate::error::Error::DieselError { source: e })
    {
        Ok(_) => {
            trace!("Report epoch_index {} input_index {} report_index {}  written successfully to db",
                db_report.epoch_index, db_report.input_index, db_report.report_index);
            Ok(())
        }
        Err(e) => {
            error!("Failed to write report to db, details: {}", e.to_string());
            Err(e)
        }
    }
}

/// Insert epoch to database if it does not exist
pub(crate) fn insert_epoch(
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
pub(crate) fn insert_input(
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
        EpochIndexType::Voucher => {
            rollups_data::database::CURRENT_VOUCHER_EPOCH_INDEX
        }
        EpochIndexType::Report => {
            rollups_data::database::CURRENT_REPORT_EPOCH_INDEX
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
    postgres_config: &PostgresConfig,
    epoch_index_type: EpochIndexType,
) -> Result<i32, crate::error::Error> {
    let conn = rollups_data::database::connect_to_database_with_retry_async(
        &postgres_config.postgres_hostname,
        postgres_config.postgres_port,
        &postgres_config.postgres_user,
        &postgres_config.postgres_password,
        &postgres_config.postgres_db,
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

    let db_config = &config.database;

    let mut db_pool = rollups_data::database::create_db_pool_with_retry(
        &db_config.postgres_hostname,
        db_config.postgres_port,
        &db_config.postgres_user,
        &db_config.postgres_password,
        &db_config.postgres_db,
    );

    loop {
        tokio::select! {
            Some(response) = message_rx.recv() => {
                // Try to get connection to dabase, recreate db pool if failed
                let conn = match db_pool.get() {
                    Ok(conn) => Some(conn),
                    Err(e) => {
                        error!("Failed to get connection from db pool, postgres://{}@{}:{}/{}, error: {}",
                            &db_config.postgres_user,
                            &db_config.postgres_hostname,
                            config.database.postgres_port,
                            &db_config.postgres_db,
                            e.to_string()
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                        // Recreate connection pool
                        match rollups_data::database::create_db_pool_with_retry_async(
                            &db_config.postgres_hostname,
                            db_config.postgres_port,
                            &db_config.postgres_user,
                            &db_config.postgres_password,
                            &db_config.postgres_db,
                        ).await {
                            Ok(pool) => {
                                db_pool = pool;
                                db_pool.get().ok()
                            },
                            Err(_e) => {
                                error!("Failed to recreate db pool, postgres://{}@{}:{}/{}",
                                &db_config.postgres_user,
                                &db_config.postgres_hostname,
                                config.database.postgres_port,
                                &db_config.postgres_db);
                                None
                            }
                        }
                    }
                };
                let conn = if let Some(c) = conn {
                    c
                } else {
                    error!("Unable to connect to postgres://{}@{}:{}/{} to save message, received message {:?} lost",
                        &db_config.postgres_user,
                        &db_config.postgres_hostname,
                        config.database.postgres_port,
                        &db_config.postgres_db,
                        response
                    );
                    // Message lost, wait for another message in the next loop
                    continue;
                };
                match response {
                    Message::Notice(proof, mut notice) => {
                        debug!("Notice message received session_id {} epoch_index {} input_index {} notice_index {}, writing to db",
                            &notice.session_id, notice.epoch_index, notice.input_index, notice.notice_index);

                        // Spawn tokio blocking task, diesel access to db is blocking
                        let _res = tokio::task::spawn_blocking(move || {
                            // Insert notice to database
                            match insert_notice(&notice, &conn) {
                                Ok(new_notice_id) => {
                                    if let Some(notice_id) = new_notice_id {
                                        notice.id = notice_id;
                                        // Insert related notice proof to database, update notice record with proof_id
                                        match insert_proof(&proof, &conn) {
                                                Ok(new_proof_id) => {
                                                    notice.proof_id = new_proof_id;
                                                    if let Err(err) = update_notice(&notice, &conn) {
                                                        warn!("Failed to update notice with id {} for proof id {:?}, error {}",
                                                            &notice.id, notice.proof_id, err.to_string());
                                                    }
                                                },
                                                Err(err) => {
                                                    //ignore error, continue
                                                    warn!("Proof output_hashes_root_hash {} vouchers_epoch_root_hash {} notices_epoch_root_hash {} machine_state_hash {} is lost, error: {}",
                                                        &proof.output_hashes_root_hash, &proof.vouchers_epoch_root_hash, &proof.notices_epoch_root_hash,
                                                    &proof.machine_state_hash, err.to_string());
                                                }
                                            };
                                        }

                                    }
                                Err(err) => {
                                    //ignore error, continue
                                    warn!("Notice session_id {} epoch_index {} input_index {} notice_index {} is lost, error: {}",
                                        &notice.session_id, &notice.epoch_index, &notice.input_index, &notice.notice_index, err.to_string());
                                }
                            }
                            if let Err(err) = update_current_epoch_index(&conn, notice.epoch_index, EpochIndexType::Notice) {
                                warn!("Failed to update notice database epoch index {}, details: {}", notice.epoch_index, err.to_string());
                            }
                        }).await;
                    }
                    Message::Report(report) => {
                        debug!("Report message received epoch_index {} input_index {} report_index {}, writing to db",
                            report.epoch_index, report.input_index, report.report_index);
                        // Spawn tokio blocking task, diesel access to db is blocking
                        let _res = tokio::task::spawn_blocking(move || {
                            if let Err(err) = insert_report(&report, &conn) {
                                //ignore error, continue
                                warn!("Report epoch_index {} input_index {} report_index {} is lost, error: {}",
                                    &report.epoch_index, &report.input_index, &report.report_index, err.to_string());
                            }
                            if let Err(err) = update_current_epoch_index(&conn, report.epoch_index, EpochIndexType::Report) {
                                warn!("Failed to update report database epoch index {}, details: {}", report.epoch_index, err.to_string());
                            }
                        }).await;
                    }
                    Message::Voucher(proof, mut voucher) => {
                        debug!("Voucher message received epoch_index {} input_index {} voucher_index {}, writing to db",
                            voucher.epoch_index, voucher.input_index, voucher.voucher_index);

                        // Spawn tokio blocking task, diesel access to db is blocking
                        let _res = tokio::task::spawn_blocking(move || {
                            // Insert proof to database, assign proof id to voucher
                            match insert_proof(&proof, &conn) {
                                Ok(proof_id) => {
                                    voucher.proof_id = proof_id;
                                },
                                Err(err) => {
                                    //ignore error, continue
                                    warn!("Proof output_hashes_root_hash {} vouchers_epoch_root_hash {} notices_epoch_root_hash {} machine_state_hash {} is lost, error: {}",
                                        &proof.output_hashes_root_hash, &proof.vouchers_epoch_root_hash, &proof.notices_epoch_root_hash,
                                        &proof.machine_state_hash, err.to_string());
                                }
                            };

                            // Insert voucher to database
                            match insert_voucher(&voucher, &conn) {
                                Ok(new_voucher_id) => {
                                    if let Some(voucher_id) = new_voucher_id {
                                        voucher.id = voucher_id;
                                        // Insert related voucher proof to database, update voucher record with proof_id
                                        match insert_proof(&proof, &conn) {
                                            Ok(new_proof_id) => {
                                                voucher.proof_id = new_proof_id;
                                                if let Err(err) = update_voucher(&voucher, &conn) {
                                                    warn!("Failed to update voucher with id {} for proof id {:?}, error {}",
                                                        &voucher.id, voucher.proof_id, err.to_string());
                                                }
                                            },
                                            Err(err) => {
                                                //ignore error, continue
                                                warn!("Proof output_hashes_root_hash {} vouchers_epoch_root_hash {} notices_epoch_root_hash {} machine_state_hash {} is lost, error: {}",
                                                    &proof.output_hashes_root_hash, &proof.vouchers_epoch_root_hash, &proof.notices_epoch_root_hash,
                                                &proof.machine_state_hash, err.to_string());
                                            }
                                        };
                                    }
                                }
                                Err(err) => {
                                     //ignore error, continue
                                    warn!("Voucher epoch_index {} input_index {} voucher_index {} is lost, error: {}",
                                        &voucher.epoch_index, &voucher.input_index, &voucher.voucher_index, err.to_string());
                                }
                            }


                            if let Err(err) = update_current_epoch_index(&conn, voucher.epoch_index, EpochIndexType::Voucher) {
                                warn!("Failed to update voucher database epoch index {}, details: {}", voucher.epoch_index, err.to_string());
                            }
                        }).await;
                    }
                    Message::Input(input) => {
                        debug!("Input message received id {} input_index {} epoch_index {} sender {} block_number {} timestamp {}",
                            &input.id, &input.input_index, &input.epoch_index, &input.sender, &input.block_number, &input.timestamp);
                        // Spawn tokio blocking task, diesel access to db is blocking
                        let _res = tokio::task::spawn_blocking(move || {
                            // Insert epoch if not in database
                            if let Err(err) = insert_epoch(input.epoch_index, &conn) {
                                //ignore error, continue
                                warn!("Epoch index {} is lost, error: {}", &input.epoch_index, err.to_string());
                            }
                            if let Err(err) = insert_input(&input, &conn) {
                                //ignore error, continue
                                warn!("Input id {} input_index {} epoch_index {} sender {} block_number {} timestamp {} is lost, error: {}",
                                    &input.id, &input.input_index, &input.epoch_index, &input.sender, &input.block_number, &input.timestamp,
                                    err.to_string());
                            }
                            if let Err(err) = update_current_epoch_index(&conn, input.epoch_index, EpochIndexType::Input) {
                                warn!("Failed to update input database epoch index {}, details: {}", input.epoch_index, err.to_string());
                            }
                        }).await;
                    }
                }
            },
            else => {()}
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

pub mod testing {
    use super::*;

    pub fn test_insert_notice(
        db_notice: &DbNotice,
        conn: &PgConnection,
    ) -> Result<Option<i32>, crate::error::Error> {
        insert_notice(db_notice, conn)
    }

    pub fn test_insert_voucher(
        db_voucher: &DbVoucher,
        conn: &PgConnection,
    ) -> Result<Option<i32>, crate::error::Error> {
        insert_voucher(db_voucher, conn)
    }

    pub fn test_insert_report(
        db_report: &DbReport,
        conn: &PgConnection,
    ) -> Result<(), crate::error::Error> {
        insert_report(db_report, conn)
    }

    pub fn test_insert_input(
        db_input: &DbInput,
        conn: &PgConnection,
    ) -> Result<(), crate::error::Error> {
        insert_input(db_input, conn)
    }

    pub fn test_insert_proof(
        db_proof: &DbProof,
        conn: &PgConnection,
    ) -> Result<Option<i32>, crate::error::Error> {
        insert_proof(db_proof, conn)
    }

    pub fn test_update_notice(
        db_notice: &DbNotice,
        conn: &PgConnection,
    ) -> Result<(), crate::error::Error> {
        update_notice(db_notice, conn)
    }

    pub fn test_update_voucher(
        db_voucher: &DbVoucher,
        conn: &PgConnection,
    ) -> Result<(), crate::error::Error> {
        update_voucher(db_voucher, conn)
    }
}
