// Copyright (C) 2022 Cartesi Pte. Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use diesel::pg::PgConnection;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use diesel_migrations::Migration;
use indexer::db_service::testing::{
    test_insert_input, test_insert_notice, test_insert_proof,
    test_insert_report, test_insert_voucher, test_update_notice,
    test_update_voucher,
};
use rollups_data::database::{DbInput, DbNotice, DbProof, DbReport, DbVoucher};
use rstest::*;
use serial_test::serial;
use std::future::Future;

const POSTGRES_PORT: u16 = 5434;
const POSTGRES_HOSTNAME: &str = "127.0.0.1";
const POSTGRES_USER: &str = "postgres";
const POSTGRES_PASSWORD: &str = "password";
const POSTGRES_DB: &str = "test_indexer";
const PATH_TO_MIGRATION_FOLDER: &str = "../data/migrations/";

#[cfg(feature = "postgres")]
diesel_migrations::embed_migrations!(PATH_TO_MIGRATION_FOLDER);

struct Context {
    postgres_endpoint: String,
}

impl Drop for Context {
    fn drop(&mut self) {}
}

pub fn connect_to_database(
    postgres_endpoint: &str,
) -> Result<PgConnection, diesel::ConnectionError> {
    PgConnection::establish(&postgres_endpoint)
}

pub fn create_database(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
) -> Result<(), diesel::result::Error> {
    let endpoint = format!(
        "postgres://{}:{}@{}:{}",
        user,
        password,
        host,
        &port.to_string()
    );

    let conn = connect_to_database(&endpoint).unwrap();
    // Drop old database
    match diesel::sql_query(&format!("DROP DATABASE IF EXISTS {}", POSTGRES_DB))
        .execute(&conn)
    {
        Ok(res) => {
            println!("Database dropped, result {}", res);
        }
        Err(e) => {
            println!("Error dropping database: {}", e.to_string());
        }
    };

    // Create new database
    match diesel::sql_query(&format!("CREATE DATABASE {}", POSTGRES_DB))
        .execute(&conn)
    {
        Ok(res) => {
            println!("Database created, result {}", res);
        }
        Err(e) => {
            println!("Error creating database: {}", e.to_string());
        }
    };
    Ok(())
}

fn perform_diesel_setup(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
    database: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = format!(
        "postgres://{}:{}@{}:{}/{}",
        user,
        password,
        host,
        &port.to_string(),
        database
    );

    std::process::Command::new("diesel")
        .arg(&format!("setup"))
        .arg(&format!("--database-url={}", endpoint))
        .arg(&format!("--migration-dir={}", PATH_TO_MIGRATION_FOLDER))
        .output()
        .expect("Unable to launch Cartesi machine server");

    Ok(())
}

#[fixture]
async fn context_empty_db() -> Context {
    // Create database
    create_database(
        POSTGRES_USER,
        POSTGRES_PASSWORD,
        POSTGRES_HOSTNAME,
        POSTGRES_PORT,
    )
    .unwrap();

    Context {
        postgres_endpoint: format!(
            "postgres://{}:{}@{}:{}/{}",
            POSTGRES_USER,
            POSTGRES_PASSWORD,
            POSTGRES_HOSTNAME,
            POSTGRES_PORT,
            POSTGRES_DB
        ),
    }
}

#[fixture]
async fn context_migrated_db() -> Context {
    // Create database
    create_database(
        POSTGRES_USER,
        POSTGRES_PASSWORD,
        POSTGRES_HOSTNAME,
        POSTGRES_PORT,
    )
    .unwrap();

    // Perform diesel setup
    perform_diesel_setup(
        POSTGRES_USER,
        POSTGRES_PASSWORD,
        POSTGRES_HOSTNAME,
        POSTGRES_PORT,
        POSTGRES_DB,
    )
    .unwrap();

    Context {
        postgres_endpoint: format!(
            "postgres://{}:{}@{}:{}/{}",
            POSTGRES_USER,
            POSTGRES_PASSWORD,
            POSTGRES_HOSTNAME,
            POSTGRES_PORT,
            POSTGRES_DB
        ),
    }
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_database_creation_diesel_setup(
) -> Result<(), Box<dyn std::error::Error>> {
    // Create database
    create_database(
        POSTGRES_USER,
        POSTGRES_PASSWORD,
        POSTGRES_HOSTNAME,
        POSTGRES_PORT,
    )
    .unwrap();

    // Perform diesel setup
    perform_diesel_setup(
        POSTGRES_USER,
        POSTGRES_PASSWORD,
        POSTGRES_HOSTNAME,
        POSTGRES_PORT,
        POSTGRES_DB,
    )
    .unwrap();

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_database_connection(
    context_empty_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_empty_db.await;
    println!(
        "Trying to connect to database: {}",
        &context.postgres_endpoint
    );
    let _conn = connect_to_database(&context.postgres_endpoint).unwrap();
    println!("Connected to database: {}", &context.postgres_endpoint);
    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_perform_migration(
    context_empty_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_empty_db.await;
    println!("Performing database migration...");
    let path = std::env::current_dir()
        .unwrap()
        .as_path()
        .join(std::path::Path::new(PATH_TO_MIGRATION_FOLDER));
    println!("The migration search directory is {}", path.display());
    let migration_dir =
        diesel_migrations::search_for_migrations_directory(&path).unwrap();
    println!("Migration directory found {}", migration_dir.display());

    // Connect to database
    let conn = connect_to_database(&context.postgres_endpoint).unwrap();

    // Perform migrations
    for entry in migration_dir.read_dir()? {
        let migration_directory_name = entry?.path();
        println!(
            "Executing postgres migration from directory {:?}",
            migration_directory_name
        );

        let migration = diesel_migrations::migration_from(
            std::path::PathBuf::from(&migration_directory_name),
        )
        .map_err(|e| {
            eprintln!(
                "Failed to parse migration directory {:?} {}",
                &migration_directory_name,
                e.to_string()
            )
        })
        .unwrap();
        migration.revert(&conn).unwrap();
        migration.run(&conn).unwrap();
    }

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_notice_insertion(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    use rollups_data::database::schema::notices;
    let context = context_migrated_db.await;
    let conn = connect_to_database(&context.postgres_endpoint).unwrap();
    let mut new_notices = vec![
        DbNotice {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            notice_index: 0,
            payload: Some(vec![0x00, 0x11, 0x22, 0x33, 0x44]),
            keccak: "0xccf8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            session_id: "first_session".to_string(),
            proof_id: None,
        },
        DbNotice {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            notice_index: 2,
            payload: Some(vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x66, 0x77]),
            keccak: "0xaaf8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3bb".to_string(),
            session_id: "first_session".to_string(),
            proof_id: None,
        },
        DbNotice {
            id: 0,
            epoch_index: 0,
            input_index: 3,
            notice_index: 0,
            payload: Some(vec![0x11]),
            keccak: "0xf8f8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3aa".to_string(),
            session_id: "second_session".to_string(),
            proof_id: None,
        },
        DbNotice {
            id: 0,
            epoch_index: 1,
            input_index: 2,
            notice_index: 0,
            payload: Some(vec![]),
            keccak: "0xf8f8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f315".to_string(),
            session_id: "second_session".to_string(),
            proof_id: None,
        },
    ];

    for notice in new_notices.iter() {
        test_insert_notice(notice, &conn).unwrap();
    }

    let read_notices = notices::dsl::notices
        .order_by(notices::dsl::id.asc())
        .load::<DbNotice>(&conn)?;

    // Update autoincrement serial id
    new_notices[0].id = 1;
    new_notices[1].id = 2;
    new_notices[2].id = 3;
    new_notices[3].id = 4;
    assert_eq!(4, read_notices.len());
    assert_eq!(new_notices, read_notices);
    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_voucher_insertion(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    use rollups_data::database::schema::vouchers;
    let context = context_migrated_db.await;
    let conn = connect_to_database(&context.postgres_endpoint).unwrap();
    let mut new_vouchers = vec![
        DbVoucher {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            voucher_index: 0,
            payload: Some(vec![0x00, 0x11, 0x22, 0x33, 0x44]),
            destination: "0xf8f8a2f43c8376ccb0871305060d7b27b0554d2c"
                .to_string(),
            proof_id: None,
        },
        DbVoucher {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            voucher_index: 2,
            payload: Some(vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x66, 0x77]),
            destination: "0xccf8a2f43c8376ccb0871305060d7b27b0554aac"
                .to_string(),
            proof_id: None,
        },
        DbVoucher {
            id: 0,
            epoch_index: 0,
            input_index: 3,
            voucher_index: 0,
            payload: Some(vec![0x11]),
            destination: "0x00f8a2f43c8376ccb0871305060d7b27b0554aff"
                .to_string(),
            proof_id: None,
        },
        DbVoucher {
            id: 0,
            epoch_index: 1,
            input_index: 2,
            voucher_index: 0,
            payload: Some(vec![]),
            destination: "0x11aaa2f43c8376ccb0871305060d7b27b0554aff"
                .to_string(),
            proof_id: None,
        },
    ];

    for voucher in new_vouchers.iter() {
        test_insert_voucher(voucher, &conn).unwrap();
    }

    let read_vouchers = vouchers::dsl::vouchers
        .order_by(vouchers::dsl::id.asc())
        .load::<DbVoucher>(&conn)?;

    // Update autoincrement serial id
    new_vouchers[0].id = 1;
    new_vouchers[1].id = 2;
    new_vouchers[2].id = 3;
    new_vouchers[3].id = 4;
    assert_eq!(4, read_vouchers.len());
    assert_eq!(new_vouchers, read_vouchers);
    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_report_insertion(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    use rollups_data::database::schema::reports;
    let context = context_migrated_db.await;
    let conn = connect_to_database(&context.postgres_endpoint).unwrap();
    let mut new_reports = vec![
        DbReport {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            report_index: 0,
            payload: Some(vec![0x00, 0x11, 0x22, 0x33, 0x44]),
        },
        DbReport {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            report_index: 2,
            payload: Some(vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x66, 0x77]),
        },
        DbReport {
            id: 0,
            epoch_index: 0,
            input_index: 3,
            report_index: 0,
            payload: Some(vec![0x11]),
        },
        DbReport {
            id: 0,
            epoch_index: 1,
            input_index: 2,
            report_index: 0,
            payload: Some(vec![]),
        },
    ];

    for report in new_reports.iter() {
        test_insert_report(report, &conn).unwrap();
    }

    let read_reports = reports::dsl::reports
        .order_by(reports::dsl::id.asc())
        .load::<DbReport>(&conn)?;

    // Update autoincrement serial id
    new_reports[0].id = 1;
    new_reports[1].id = 2;
    new_reports[2].id = 3;
    new_reports[3].id = 4;
    assert_eq!(4, read_reports.len());
    assert_eq!(new_reports, read_reports);
    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_input_insertion(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    use rollups_data::database::schema::inputs;
    let context = context_migrated_db.await;
    let conn = connect_to_database(&context.postgres_endpoint).unwrap();
    let mut new_inputs = vec![
        DbInput {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            sender: "0x00f8a2f43c8376ccb0871305060d7b27b0554aff".to_string(),
            block_number: 111,
            timestamp: chrono::NaiveDateTime::from_timestamp(
                chrono::offset::Utc::now().timestamp(),
                0,
            ),
            payload: vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x66, 0x77],
            tx_hash: None,
        },
        DbInput {
            id: 0,
            epoch_index: 0,
            input_index: 4,
            sender: "0xaaf8a2f43c8376ccb0871305060d7b27b0554aff".to_string(),
            block_number: 114,
            timestamp: chrono::NaiveDateTime::from_timestamp(
                chrono::offset::Utc::now().timestamp(),
                0,
            ),
            payload: vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x66, 0x77],
            tx_hash: None,
        },
        DbInput {
            id: 0,
            epoch_index: 1,
            input_index: 3,
            sender: "0x00f8a2f43c8376ccb0871305060d7b27b0554acc".to_string(),
            block_number: 117,
            timestamp: chrono::NaiveDateTime::from_timestamp(
                chrono::offset::Utc::now().timestamp(),
                0,
            ),
            payload: vec![
                0x00, 0x11, 0x22, 0x33, 0x44, 0x66, 0x77, 0x11, 0x22, 0x33,
                0x44,
            ],
            tx_hash: None,
        },
        DbInput {
            id: 0,
            epoch_index: 2,
            input_index: 2,
            sender: "0x11f8a2f43c8376ccb0871305060d7b27b0554acc".to_string(),
            block_number: 143,
            timestamp: chrono::NaiveDateTime::from_timestamp(
                chrono::offset::Utc::now().timestamp(),
                0,
            ),
            payload: vec![
                0x00, 0x11, 0x22, 0x33, 0x44, 0x66, 0x77, 0x11, 0x22, 0x33,
                0x44, 0x11, 0x22,
            ],
            tx_hash: None,
        },
    ];

    for input in new_inputs.iter() {
        test_insert_input(input, &conn).unwrap();
    }

    let read_inputs = inputs::dsl::inputs
        .order_by(rollups_data::database::schema::inputs::dsl::id.asc())
        .load::<DbInput>(&conn)?;

    // Update autoincrement serial id
    new_inputs[0].id = 1;
    new_inputs[1].id = 2;
    new_inputs[2].id = 3;
    new_inputs[3].id = 4;
    assert_eq!(4, read_inputs.len());
    assert_eq!(new_inputs, read_inputs);
    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_proof_insertion(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    use rollups_data::database::schema::proofs;
    let context = context_migrated_db.await;
    let conn = connect_to_database(&context.postgres_endpoint).unwrap();
    let mut new_proofs = vec![
        DbProof {
            id: 0,
            machine_state_hash: "0xccf8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_root_hash: "0xaaffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            notices_epoch_root_hash: "0xaaffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            vouchers_epoch_root_hash: "0x11ffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_in_epoch_siblings: vec!["0x2222a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                "0x3333a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                "0x4444a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
            keccak_in_hashes_siblings: vec!["0x4444a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x5555a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x6666a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
        },
        DbProof {
            id: 0,
            machine_state_hash: "0xaaf8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_root_hash: "0xbbffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            notices_epoch_root_hash: "0xcccfa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            vouchers_epoch_root_hash: "0xeeefa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_in_epoch_siblings: vec!["0x1122a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x1133a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x1144a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
            keccak_in_hashes_siblings: vec!["0x2244a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x3355a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x4466a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
        },
        DbProof {
            id: 0,
            machine_state_hash: "0xccffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_root_hash: "0xaa5555f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            notices_epoch_root_hash: "0xaa6666f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            vouchers_epoch_root_hash: "0x117777f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_in_epoch_siblings: vec!["0x222222f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x2233a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x334444f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
            keccak_in_hashes_siblings: vec!["0x1144a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x2255a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0xaa66a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
        },
    ];

    for proof in new_proofs.iter() {
        test_insert_proof(proof, &conn).unwrap();
    }

    let read_proofs = proofs::dsl::proofs
        .order_by(proofs::dsl::id.asc())
        .load::<DbProof>(&conn)?;

    // Update autoincrement serial id
    new_proofs[0].id = 1;
    new_proofs[1].id = 2;
    new_proofs[2].id = 3;
    assert_eq!(3, read_proofs.len());
    assert_eq!(new_proofs, read_proofs);
    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_notice_proof_insertion(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    use rollups_data::database::schema::notices;
    use rollups_data::database::schema::proofs;
    let context = context_migrated_db.await;
    let conn = connect_to_database(&context.postgres_endpoint).unwrap();
    let mut new_notices = vec![
        DbNotice {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            notice_index: 0,
            payload: Some(vec![0x00, 0x11, 0x22, 0x33, 0x44]),
            keccak: "0xccf8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            session_id: "first_session".to_string(),
            proof_id: None,
        },
        DbNotice {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            notice_index: 2,
            payload: Some(vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x66, 0x77]),
            keccak: "0xaaf8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3bb".to_string(),
            session_id: "first_session".to_string(),
            proof_id: None,
        },
        DbNotice {
            id: 0,
            epoch_index: 0,
            input_index: 3,
            notice_index: 0,
            payload: Some(vec![0x11]),
            keccak: "0xf8f8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3aa".to_string(),
            session_id: "second_session".to_string(),
            proof_id: None,
        }
    ];

    for notice in new_notices.iter() {
        test_insert_notice(notice, &conn).unwrap();
    }
    new_notices[0].id = 1;
    new_notices[1].id = 2;
    new_notices[2].id = 3;

    let new_proofs = vec![
        DbProof {
            id: 0,
            machine_state_hash: "0xccf8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_root_hash: "0xaaffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            notices_epoch_root_hash: "0xaaffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            vouchers_epoch_root_hash: "0x11ffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_in_epoch_siblings: vec!["0x2222a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x3333a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x4444a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
            keccak_in_hashes_siblings: vec!["0x4444a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x5555a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x6666a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
        },
        DbProof {
            id: 0,
            machine_state_hash: "0xaaf8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_root_hash: "0xbbffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            notices_epoch_root_hash: "0xcccfa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            vouchers_epoch_root_hash: "0xeeefa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_in_epoch_siblings: vec!["0x1122a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x1133a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x1144a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
            keccak_in_hashes_siblings: vec!["0x2244a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x3355a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x4466a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
        },
        DbProof {
            id: 0,
            machine_state_hash: "0xccffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_root_hash: "0xaa5555f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            notices_epoch_root_hash: "0xaa6666f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            vouchers_epoch_root_hash: "0x117777f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_in_epoch_siblings: vec!["0x222222f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x2233a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x334444f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
            keccak_in_hashes_siblings: vec!["0x1144a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x2255a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0xaa66a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
        },
    ];

    for (index, proof) in new_proofs.iter().enumerate() {
        let new_proof_id = test_insert_proof(proof, &conn).unwrap();
        new_notices[index].proof_id = new_proof_id;
        test_update_notice(&new_notices[index], &conn).unwrap();
    }

    let read_proofs = proofs::dsl::proofs.load::<DbProof>(&conn)?;
    assert_eq!(3, read_proofs.len());

    let read_notices = notices::dsl::notices
        .order_by(notices::dsl::id.asc())
        .load::<DbNotice>(&conn)?;
    assert_eq!(Some(2), read_notices[1].proof_id);
    assert_eq!(3, read_notices.len());
    assert_eq!(new_notices, read_notices);
    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_voucher_proof_insertion(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    use rollups_data::database::schema::proofs;
    use rollups_data::database::schema::vouchers;
    let context = context_migrated_db.await;
    let conn = connect_to_database(&context.postgres_endpoint).unwrap();
    let mut new_vouchers = vec![
        DbVoucher {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            voucher_index: 0,
            payload: Some(vec![0x00, 0x11, 0x22, 0x33, 0x44]),
            destination: "0xf8f8a2f43c8376ccb0871305060d7b27b0554d2c"
                .to_string(),
            proof_id: None,
        },
        DbVoucher {
            id: 0,
            epoch_index: 0,
            input_index: 1,
            voucher_index: 2,
            payload: Some(vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x66, 0x77]),
            destination: "0xccf8a2f43c8376ccb0871305060d7b27b0554aac"
                .to_string(),
            proof_id: None,
        },
        DbVoucher {
            id: 0,
            epoch_index: 0,
            input_index: 3,
            voucher_index: 0,
            payload: Some(vec![0x11]),
            destination: "0x00f8a2f43c8376ccb0871305060d7b27b0554aff"
                .to_string(),
            proof_id: None,
        },
    ];

    for voucher in new_vouchers.iter() {
        test_insert_voucher(voucher, &conn).unwrap();
    }

    // Update autoincrement serial id
    new_vouchers[0].id = 1;
    new_vouchers[1].id = 2;
    new_vouchers[2].id = 3;
    let new_proofs = vec![
        DbProof {
            id: 0,
            machine_state_hash: "0xccf8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_root_hash: "0xaaffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            notices_epoch_root_hash: "0xaaffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            vouchers_epoch_root_hash: "0x11ffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_in_epoch_siblings: vec!["0x2222a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x3333a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x4444a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
            keccak_in_hashes_siblings: vec!["0x4444a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x5555a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x6666a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
        },
        DbProof {
            id: 0,
            machine_state_hash: "0xaaf8a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_root_hash: "0xbbffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            notices_epoch_root_hash: "0xcccfa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            vouchers_epoch_root_hash: "0xeeefa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_in_epoch_siblings: vec!["0x1122a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x1133a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x1144a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
            keccak_in_hashes_siblings: vec!["0x2244a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x3355a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x4466a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
        },
        DbProof {
            id: 0,
            machine_state_hash: "0xccffa2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_root_hash: "0xaa5555f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            notices_epoch_root_hash: "0xaa6666f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            vouchers_epoch_root_hash: "0x117777f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
            output_hashes_in_epoch_siblings: vec!["0x222222f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x2233a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                                  "0x334444f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
            keccak_in_hashes_siblings: vec!["0x1144a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0x2255a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string(),
                                            "0xaa66a2f43c8376ccb0871305060d7b27b0554d2cc72bccf41b2705608452f3ff".to_string()],
        },
    ];

    for (index, proof) in new_proofs.iter().enumerate() {
        let new_proof_id = test_insert_proof(proof, &conn).unwrap();
        new_vouchers[index].proof_id = new_proof_id;
        test_update_voucher(&new_vouchers[index], &conn).unwrap();
    }

    let read_proofs = proofs::dsl::proofs.load::<DbProof>(&conn)?;
    assert_eq!(3, read_proofs.len());

    let read_vouchers = vouchers::dsl::vouchers
        .order_by(vouchers::dsl::id.asc())
        .load::<DbVoucher>(&conn)?;
    assert_eq!(Some(2), read_vouchers[1].proof_id);
    assert_eq!(3, read_vouchers.len());
    assert_eq!(new_vouchers, read_vouchers);
    Ok(())
}
