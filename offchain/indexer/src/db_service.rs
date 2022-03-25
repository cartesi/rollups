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
use crate::grpc::{cartesi_machine, cartesi_server_manager};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::{Connection, Insertable, Queryable};
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace};

use crate::db_schema::{MerkleTreeProofs, Notices};
use crate::db_schema::{MerkleTreeProofs::dsl::*, Notices::dsl::*};

use chrono::Utc;
pub fn val_to_hex_str<T: std::fmt::LowerHex>(val: &T) -> String {
    format!("{:#x}", val)
}

pub fn establish_connection(database_url: &str) -> Result<PgConnection, diesel::result::ConnectionError>  {
    PgConnection::establish(&database_url)
}

#[derive(Debug)]
pub struct NoticeInfo {
    pub session_id: String,
    pub epoch_index: u64,
    pub input_index: u64,
    pub notice_index: u64,
}

#[derive(Insertable, Queryable, Debug, PartialEq)]
#[table_name = "MerkleTreeProofs"]
pub struct DbMerkleTreeProof {
    id: uuid::Uuid,
    target_address: String,
    log2_target_size: String, //todo should be uint64, change database schema
    target_hash: String,
    log2_root_size: String, //todo should be uint64, change database schema
    root_hash: String,
    sibling_hashes: serde_json::Value,
    createdAt: chrono::DateTime<Utc>, //todo rename field, created_at
    updatedAt: chrono::DateTime<Utc>, // todo rename field, updated_at
}

impl From<&cartesi_machine::MerkleTreeProof> for DbMerkleTreeProof {
    fn from(proof: &cartesi_machine::MerkleTreeProof) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            target_address: proof.target_address.to_string(),
            log2_target_size: proof.log2_target_size.to_string(),
            target_hash: hex::encode(
                &proof
                    .target_hash
                    .as_ref()
                    .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                    .data,
            ),
            log2_root_size: hex::encode(proof.log2_root_size.to_string()),
            root_hash: hex::encode(
                &proof
                    .root_hash
                    .as_ref()
                    .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                    .data,
            ),
            sibling_hashes: proof
                .sibling_hashes
                .iter()
                .map(|h| h.data.clone())
                .collect(),
            createdAt: chrono::offset::Utc::now(),
            updatedAt: chrono::offset::Utc::now(),
        }
    }
}

#[derive(Insertable, Queryable, Debug, PartialEq)]
#[table_name = "Notices"]
pub struct DbNotice {
    pub session_id: String,
    pub epoch_index: String, //todo should be uint64, change database schema
    pub input_index: String, //todo should be uint64, change database  schema
    pub notice_index: String, //todo should be uint64, change database schema
    pub keccak: String,      //todo change to Vec[u8]/real binary data?
    pub payload: String, // todo  natively it is Vec[u8], should we keep it in database in binary format?
    pub keccak_in_notice_hashes: uuid::Uuid,
    createdAt: chrono::DateTime<Utc>, //todo rename field, created_at
    updatedAt: chrono::DateTime<Utc>, // todo rename field, updated_at
}

impl DbNotice {
    fn new(
        notice_info: &NoticeInfo,
        notice: &cartesi_server_manager::Notice,
        merke_tree_proof_id: &uuid::Uuid,
    ) -> Self {
        DbNotice {
            session_id: notice_info.session_id.to_string(),
            epoch_index: notice_info.epoch_index.to_string(),
            input_index: notice_info.input_index.to_string(),
            notice_index: notice_info.notice_index.to_string(),
            keccak: hex::encode(
                &notice
                    .keccak
                    .as_ref()
                    .unwrap_or(&cartesi_machine::Hash { data: vec![] })
                    .data,
            ),
            payload: hex::encode(&notice.payload),
            keccak_in_notice_hashes: *merke_tree_proof_id,
            createdAt: chrono::offset::Utc::now(), //todo rename field, created_at
            updatedAt: chrono::offset::Utc::now(), // todo rename field, updated_at
        }
    }
}

#[derive(Debug)]
pub enum Message {
    Notice(NoticeInfo, cartesi_server_manager::Notice),
}

async fn db_loop(
    config: IndexerConfig,
    mut message_rx: mpsc::Receiver<Message>,
) -> Result<(), crate::error::Error> {
    info!("starting db loop");
    loop {
        tokio::select! {
            Some(response) = message_rx.recv() => {
                match response {
                    Message::Notice(notice_info, notice) => {
                        debug!("Notice message received session_id {} epoch_index {} input_index {} notice_index {}, writing to db",
                            &notice_info.session_id, notice_info.epoch_index, notice_info.input_index, notice_info.notice_index);
                        let conn = match establish_connection(&config.postgres_endpoint) {
                            Ok(connection) => connection,
                            Err(e) => {
                                error!("Failed to connect to postgres database, details: {}", e.to_string());
                                continue;
                            }
                        };
                        // Spawn tokio blocking task, diesel access to db is blocking
                        let _res = tokio::task::spawn_blocking(move || {
                            // Write merkle tree proof to database
                            let db_merkle_tree_proof = DbMerkleTreeProof::from(notice.keccak_in_notice_hashes.as_ref().unwrap());
                            let merke_tree_proof_id: uuid::Uuid = match diesel::insert_into(MerkleTreeProofs).values(&db_merkle_tree_proof)
                                .returning(id)
                                .get_result::<uuid::Uuid>(&conn).map_err(|e| crate::error::Error::DieselError {source: e }) {
                                Ok(proof_id) => {
                                    trace!("Merkle tree proof id {} written to db successfully", proof_id.to_string());
                                    proof_id
                                },
                                Err(e) => {
                                    error!("Failed to write merkle tree proof to db, details: {}", e.to_string());
                                    return ();
                                }
                            };
                            // Write notice to database
                            let db_notice = DbNotice::new(&notice_info, &notice, &merke_tree_proof_id);
                            match diesel::insert_into(Notices).values(&db_notice)
                                .execute(&conn).map_err(|e| crate::error::Error::DieselError {source: e }) {
                                Ok(_) => {
                                    trace!("Notice session_id {} epoch_index {} input_index {} notice_index {}  written successfully",
                                        db_notice.session_id, db_notice.epoch_index, db_notice.input_index, db_notice.notice_index);
                                },
                                Err(e) => {
                                    error!("Failed to write notice to db, details: {}", e.to_string());
                                }
                            };
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
    message_rx: mpsc::Receiver<Message>,
) -> Result<(), crate::error::Error> {
    db_loop(config, message_rx).await?;
    Ok(())
}
