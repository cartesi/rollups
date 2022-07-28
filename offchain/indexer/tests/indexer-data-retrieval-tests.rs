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

use async_mutex::Mutex;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use rstest::*;
use serial_test::serial;
use state_fold_types::ethabi::ethereum_types::{Address, U256};
use std::future::Future;
use std::sync::Arc;

mod common;
use common::{
    connect_to_database, create_database, perform_diesel_setup,
    test_data::get_test_epoch_status_01, PATH_TO_MIGRATION_FOLDER, POSTGRES_DB,
    POSTGRES_HOSTNAME, POSTGRES_PASSWORD, POSTGRES_PORT, POSTGRES_USER,
};

use crate::common::test_data::get_test_block_state_01;
use indexer::data_service::testing::{
    test_process_epoch_status_response, test_process_state_response,
};
use indexer::{
    config::{IndexerConfig, PostgresConfig},
    http::HealthStatus,
};
use rollups_data::database::{DbInput, DbNotice, DbReport, Message};

#[allow(dead_code)]
struct Context {
    postgres_endpoint: String,
    indexer_config: IndexerConfig,
}

impl Drop for Context {
    fn drop(&mut self) {}
}

fn new_health_status() -> Arc<Mutex<HealthStatus>> {
    Arc::new(Mutex::new(HealthStatus {
        server_manager: Ok(()),
        state_server: Ok(()),
        postgres: Ok(()),
        indexer_status: Ok(()),
    }))
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
        indexer_config: IndexerConfig {
            session_id: "test_session_1".to_string(),
            mm_endpoint: String::new(),
            state_server_endpoint: "".to_string(),
            dapp_contract_address: Address::default(),
            initial_epoch: U256::from(0),
            confirmations: 0,
            interval: 10,
            database: PostgresConfig {
                postgres_migration_folder: PATH_TO_MIGRATION_FOLDER.to_string(),
                postgres_db: POSTGRES_DB.to_string(),
                postgres_user: POSTGRES_USER.to_string(),
                postgres_password: POSTGRES_PASSWORD.to_string(),
                postgres_hostname: POSTGRES_HOSTNAME.to_string(),
                postgres_port: POSTGRES_PORT,
            },
            health_endpoint: ("127.0.0.1".to_string(), 8080),
        },
    }
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_notice_processing(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_migrated_db.await;
    let epoch_index = 0;
    let epoch_status_response = get_test_epoch_status_01(
        &context.indexer_config.session_id,
        epoch_index,
    )
    .await;
    let (message_tx, mut message_rx) =
        tokio::sync::mpsc::channel::<rollups_data::database::Message>(128);
    println!("Processing notices...");
    let session_id = context.indexer_config.session_id.clone();

    tokio::spawn(async move {
        test_process_epoch_status_response(
            epoch_status_response,
            &message_tx,
            &session_id,
            epoch_index,
        )
        .await
    })
    .await
    .unwrap()
    .unwrap();
    println!("Waiting for processed notices...");
    loop {
        if let Some(message) = message_rx.recv().await {
            match message {
                Message::Notice(proof, notice) => {
                    if let Some(proof) = proof {
                        assert_eq!(proof.machine_state_hash, "0x2510d0c35cf16959188e78c078477efc3c6cb65dd83182534a5a8594eb931d0e");
                        assert_eq!(proof.output_hashes_root_hash, "0x2dc67e41d15edb6548fae2bad020dac70b1df0c6372de66ae6a8b97669d0780d");
                        assert_eq!(proof.vouchers_epoch_root_hash, "0x45e736cc98814c3e09141fa61122137749589a6b127032fbac63346c7f7bf8a1");
                        assert_eq!(proof.output_hashes_root_hash, "0x2dc67e41d15edb6548fae2bad020dac70b1df0c6372de66ae6a8b97669d0780d");
                    } else {
                        panic!("missing proof");
                    }
                    assert_eq!(notice.id, 0);
                    assert_eq!(
                        notice.session_id.as_str(),
                        context.indexer_config.session_id.as_str()
                    );
                    assert_eq!(notice.epoch_index, epoch_index as i32);
                    assert_eq!(notice.input_index, 3);
                    assert_eq!(notice.keccak, "0x4f17f2cb8140cda8e21af799e56b63038dc3bbe33651bd88b55457a039d42023");
                    assert_eq!(
                        &notice.payload.unwrap()[..],
                        [
                            91, 91, 34, 80, 101, 116, 101, 114, 34, 44, 32, 51,
                            50, 93, 93
                        ]
                    );
                    break;
                }
                Message::Input(_input) => {
                    panic!("Input unexpected!");
                }
                _ => {
                    continue;
                }
            }
        } else {
            panic!("Failed to receive notice!");
        }
    }

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_notice_retrieval_and_insertion(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_migrated_db.await;
    let epoch_index = 0;
    let epoch_status_response = get_test_epoch_status_01(
        &context.indexer_config.session_id,
        epoch_index,
    )
    .await;
    let (message_tx, message_rx) =
        tokio::sync::mpsc::channel::<rollups_data::database::Message>(128);
    let session_id = context.indexer_config.session_id.clone();
    println!("Processing notices...");
    tokio::spawn(async move {
        test_process_epoch_status_response(
            epoch_status_response,
            &message_tx,
            &session_id,
            epoch_index,
        )
        .await
    })
    .await
    .unwrap()
    .unwrap();

    println!("Running db service...");
    let indexer_config = context.indexer_config.clone();

    async fn check_written_notices(
        indexer_config: IndexerConfig,
        postgres_endpoint: &str,
    ) -> Result<(), indexer::error::Error> {
        use rollups_data::database::schema::notices;
        println!("Waiting for notice to be written in the database");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        println!("Reading notice from db");
        let conn = connect_to_database(&postgres_endpoint).unwrap();
        let read_notices = notices::dsl::notices
            .order_by(notices::dsl::id.asc())
            .load::<DbNotice>(&conn)
            .unwrap();
        println!("Checking notice");
        let notice = &read_notices[0];
        assert_eq!(notice.id, 1);
        assert_eq!(
            notice.session_id.as_str(),
            indexer_config.session_id.as_str()
        );
        assert_eq!(notice.epoch_index, 0);
        assert_eq!(notice.input_index, 3);
        assert_eq!(notice.keccak, "0x4f17f2cb8140cda8e21af799e56b63038dc3bbe33651bd88b55457a039d42023");
        assert_eq!(
            notice.payload.as_ref().unwrap()[..],
            [91, 91, 34, 80, 101, 116, 101, 114, 34, 44, 32, 51, 50, 93, 93]
        );
        println!("Finishing test");
        Ok(())
    }

    let postgres_endpoint = context.postgres_endpoint.clone();
    tokio::select! {
        db_service_result = indexer::db_service::run(indexer_config.clone(), message_rx, new_health_status()) => {
            match db_service_result {
                Ok(_) => {println!("db service terminated successfully"); Ok(())},
                Err(e) => {println!("db service terminated with error: {}", e); Err(Box::new(e))}
            }
        },
        read_from_db = check_written_notices(indexer_config.clone(), &postgres_endpoint) => {
            match read_from_db {
                Ok(_) => {Ok(())},
                Err(e) => {Err(Box::new(e))}
            }
        }
    }
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_report_processing(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_migrated_db.await;
    let epoch_index = 0;
    let epoch_status_response = get_test_epoch_status_01(
        &context.indexer_config.session_id,
        epoch_index,
    )
    .await;
    let (message_tx, mut message_rx) =
        tokio::sync::mpsc::channel::<rollups_data::database::Message>(128);
    println!("Processing reports...");
    tokio::spawn(async move {
        test_process_epoch_status_response(
            epoch_status_response,
            &message_tx,
            &context.indexer_config.session_id,
            epoch_index,
        )
        .await
    })
    .await
    .unwrap()
    .unwrap();
    println!("Waiting for processed reports...");
    loop {
        if let Some(message) = message_rx.recv().await {
            match message {
                Message::Report(report) => {
                    println!("REPORT {:?}", report);
                    assert_eq!(report.id, 0);
                    assert_eq!(report.epoch_index, 0);
                    assert_eq!(report.input_index, 4);
                    assert_eq!(report.report_index, 0);
                    assert_eq!(report.report_index, 0);

                    assert_eq!(
                        &report.payload.unwrap()[..],
                        [
                            69, 114, 114, 111, 114, 32, 101, 120, 101, 99, 117,
                            116, 105, 110, 103, 32, 115, 116, 97, 116, 101,
                            109, 101, 110, 116, 32, 39, 83, 69, 76, 69, 39, 58,
                            32, 110, 101, 97, 114, 32, 34, 83, 69, 76, 69, 34,
                            58, 32, 115, 121, 110, 116, 97, 120, 32, 101, 114,
                            114, 111, 114
                        ]
                    );
                    break;
                }
                Message::Input(_input) => {
                    panic!("Input unexpected!");
                }
                _ => {
                    continue;
                }
            }
        } else {
            panic!("Failed to receive notice!");
        }
    }

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_report_retrieval_and_insertion(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_migrated_db.await;
    let epoch_index = 0;
    let epoch_status_response = get_test_epoch_status_01(
        &context.indexer_config.session_id,
        epoch_index,
    )
    .await;
    let (message_tx, message_rx) =
        tokio::sync::mpsc::channel::<rollups_data::database::Message>(128);
    let session_id = context.indexer_config.session_id.clone();
    println!("Processing reports...");
    tokio::spawn(async move {
        test_process_epoch_status_response(
            epoch_status_response,
            &message_tx,
            &session_id,
            epoch_index,
        )
        .await
    })
    .await
    .unwrap()
    .unwrap();

    println!("Running db service...");
    let indexer_config = context.indexer_config.clone();

    async fn check_written_reports(
        postgres_endpoint: &str,
    ) -> Result<(), indexer::error::Error> {
        use rollups_data::database::schema::reports;
        println!("Waiting for report to be written in the database");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        println!("Reading report from db");
        let conn = connect_to_database(&postgres_endpoint).unwrap();
        let read_reports = reports::dsl::reports
            .order_by(reports::dsl::id.asc())
            .load::<DbReport>(&conn)
            .unwrap();
        println!("Checking report");
        let report = &read_reports[0];
        assert_eq!(report.id, 1);
        assert_eq!(report.epoch_index, 0);
        assert_eq!(report.input_index, 4);
        assert_eq!(report.report_index, 0);
        assert_eq!(report.report_index, 0);

        assert_eq!(
            report.payload.as_ref().unwrap()[..],
            [
                69, 114, 114, 111, 114, 32, 101, 120, 101, 99, 117, 116, 105,
                110, 103, 32, 115, 116, 97, 116, 101, 109, 101, 110, 116, 32,
                39, 83, 69, 76, 69, 39, 58, 32, 110, 101, 97, 114, 32, 34, 83,
                69, 76, 69, 34, 58, 32, 115, 121, 110, 116, 97, 120, 32, 101,
                114, 114, 111, 114
            ]
        );
        println!("Finishing test");
        Ok(())
    }

    let postgres_endpoint = context.postgres_endpoint.clone();
    tokio::select! {
        db_service_result = indexer::db_service::run(indexer_config.clone(), message_rx, new_health_status()) => {
            match db_service_result {
                Ok(_) => {println!("db service terminated successfully"); Ok(())},
                Err(e) => {println!("db service terminated with error: {}", e); Err(Box::new(e))}
            }
        },
        read_from_db = check_written_reports(&postgres_endpoint) => {
            match read_from_db {
                Ok(_) => {Ok(())},
                Err(e) => {Err(Box::new(e))}
            }
        }
    }
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_input_processing(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _context = context_migrated_db.await;

    let state_response = get_test_block_state_01().await;
    let (message_tx, mut message_rx) =
        tokio::sync::mpsc::channel::<rollups_data::database::Message>(128);
    println!("Processing inputs...");
    tokio::spawn(async move {
        test_process_state_response(state_response, &message_tx).await
    })
    .await
    .unwrap()
    .unwrap();
    println!("Waiting for processed inputs...");
    let mut input_counter = 0;
    loop {
        if let Some(message) = message_rx.recv().await {
            match message {
                Message::Input(input) => {
                    println!("INPUT {:?}", input);
                    input_counter += 1;
                    if input_counter == 2 {
                        assert_eq!(input.id, 0);
                        assert_eq!(input.epoch_index, 0);
                        assert_eq!(input.input_index, 1);
                        assert_eq!(
                            input.sender.as_str(),
                            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
                        );
                        assert_eq!(input.block_number, 34);
                        assert_eq!(
                            &input.payload[..],
                            [
                                67, 82, 69, 65, 84, 69, 32, 84, 65, 66, 76, 69,
                                32, 80, 101, 114, 115, 111, 110, 115, 32, 40,
                                110, 97, 109, 101, 32, 116, 101, 120, 116, 44,
                                32, 97, 103, 101, 32, 105, 110, 116, 41
                            ]
                        );
                    } else if input_counter == 4 {
                        assert_eq!(input.id, 0);
                        assert_eq!(input.epoch_index, 0);
                        assert_eq!(input.input_index, 3);
                        assert_eq!(
                            input.sender.as_str(),
                            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
                        );
                        assert_eq!(input.block_number, 36);
                        assert_eq!(
                            &input.payload[..],
                            [
                                83, 69, 76, 69, 67, 84, 32, 42, 32, 70, 82, 79,
                                77, 32, 80, 101, 114, 115, 111, 110, 115
                            ]
                        );
                    } else if input_counter == 5 {
                        break;
                    }
                }
                _ => {
                    continue;
                }
            }
        } else {
            panic!("Failed to receive input!");
        }
    }

    Ok(())
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_input_retrieval_and_insertion(
    context_migrated_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_migrated_db.await;
    let state_response = get_test_block_state_01().await;
    let (message_tx, message_rx) =
        tokio::sync::mpsc::channel::<rollups_data::database::Message>(128);
    tokio::spawn(async move {
        println!("Processing inputs...");
        test_process_state_response(state_response, &message_tx).await
    })
    .await
    .unwrap()
    .unwrap();

    println!("Running db service...");
    let indexer_config = context.indexer_config.clone();

    async fn check_written_inputs(
        postgres_endpoint: &str,
    ) -> Result<(), indexer::error::Error> {
        use rollups_data::database::schema::inputs;
        println!("Waiting for input to be written in the database");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        println!("Reading input from db");
        let conn = connect_to_database(&postgres_endpoint).unwrap();
        let read_inputs = inputs::dsl::inputs
            .order_by(inputs::dsl::id.asc())
            .load::<DbInput>(&conn)
            .unwrap();
        println!("Checking inputs");
        let input = &read_inputs[1];
        assert_eq!(input.id, 2);
        assert_eq!(input.epoch_index, 0);
        assert_eq!(input.input_index, 1);
        assert_eq!(
            input.sender.as_str(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
        assert_eq!(input.block_number, 34);
        assert_eq!(
            &input.payload[..],
            [
                67, 82, 69, 65, 84, 69, 32, 84, 65, 66, 76, 69, 32, 80, 101,
                114, 115, 111, 110, 115, 32, 40, 110, 97, 109, 101, 32, 116,
                101, 120, 116, 44, 32, 97, 103, 101, 32, 105, 110, 116, 41
            ]
        );

        let input = &read_inputs[3];
        assert_eq!(input.id, 4);
        assert_eq!(input.epoch_index, 0);
        assert_eq!(input.input_index, 3);
        assert_eq!(
            input.sender.as_str(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
        assert_eq!(input.block_number, 36);
        assert_eq!(
            &input.payload[..],
            [
                83, 69, 76, 69, 67, 84, 32, 42, 32, 70, 82, 79, 77, 32, 80,
                101, 114, 115, 111, 110, 115
            ]
        );

        println!("Finishing test");
        Ok(())
    }

    let postgres_endpoint = context.postgres_endpoint.clone();
    tokio::select! {
        db_service_result = indexer::db_service::run(indexer_config.clone(), message_rx, new_health_status()) => {
            match db_service_result {
                Ok(_) => {println!("db service terminated successfully"); Ok(())},
                Err(e) => {println!("db service terminated with error: {}", e); Err(Box::new(e))}
            }
        },
        read_from_db = check_written_inputs(&postgres_endpoint) => {
            match read_from_db {
                Ok(_) => {Ok(())},
                Err(e) => {Err(Box::new(e))}
            }
        }
    }
}
