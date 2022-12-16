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
use state_fold_types::ethabi::ethereum_types::{Address, H256};
use std::future::Future;
use std::sync::Arc;

use types::deployment_files::{
    dapp_deployment::DappDeployment, rollups_deployment::RollupsDeployment,
};

mod common;
use common::{
    connect_to_database, create_database, perform_diesel_setup,
    test_data::get_test_epoch_status_01, PATH_TO_MIGRATION_FOLDER, POSTGRES_DB,
    POSTGRES_HOSTNAME, POSTGRES_PASSWORD, POSTGRES_PORT, POSTGRES_USER,
};

use crate::common::input_test_data::get_test_block_state_01;
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
            dapp_deployment: DappDeployment {
                dapp_address: Address::default(),
                deploy_block_hash: H256::zero(),
            },
            rollups_deployment: RollupsDeployment {
                history_address: Address::default(),
                authority_address: Address::default(),
                input_box_address: Address::default(),
            },
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
                        assert_eq!(proof.keccak_in_hashes_siblings, vec![
                            "0xae39ce8537aca75e2eff3e38c98011dfe934e700a0967732fc07b430dd656a23",
                            "0x3fc9a15f5b4869c872f81087bb6104b7d63e6f9ab47f2c43f3535eae7172aa7f",
                            "0x17d2dd614cddaa4d879276b11e0672c9560033d3e8453a1d045339d34ba601b9",
                            "0xc37b8b13ca95166fb7af16988a70fcc90f38bf9126fd833da710a47fb37a55e6",
                            "0x8e7a427fa943d9966b389f4f257173676090c6e95f43e2cb6d65f8758111e309",
                            "0x30b0b9deb73e155c59740bacf14a6ff04b64bb8e201a506409c3fe381ca4ea90",
                            "0xcd5deac729d0fdaccc441d09d7325f41586ba13c801b7eccae0f95d8f3933efe",
                            "0xd8b96e5b7f6f459e9cb6a2f41bf276c7b85c10cd4662c04cbbb365434726c0a0",
                            "0xc9695393027fb106a8153109ac516288a88b28a93817899460d6310b71cf1e61",
                            "0x63e8806fa0d4b197a259e8c3ac28864268159d0ac85f8581ca28fa7d2c0c03eb",
                            "0x91e3eee5ca7a3da2b3053c9770db73599fb149f620e3facef95e947c0ee860b7",
                            "0x2122e31e4bbd2b7c783d79cc30f60c6238651da7f0726f767d22747264fdb046",
                            "0xf7549f26cc70ed5e18baeb6c81bb0625cb95bb4019aeecd40774ee87ae29ec51",
                            "0x7a71f6ee264c5d761379b3d7d617ca83677374b49d10aec50505ac087408ca89",
                            "0x2b573c267a712a52e1d06421fe276a03efb1889f337201110fdc32a81f8e1524",
                            "0x99af665835aabfdc6740c7e2c3791a31c3cdc9f5ab962f681b12fc092816a62f",
                        ]);
                        assert_eq!(proof.output_hashes_in_epoch_siblings, vec![
                            "0x27d86025599a41233848702f0cfc0437b445682df51147a632a0a083d2d38b5e",
                            "0x13e466a8935afff58bb533b3ef5d27fba63ee6b0fd9e67ff20af9d50deee3f8b",
                            "0x890740a8eb06ce9be422cb8da5cdafc2b58c0a5e24036c578de2a433c828ff7d",
                            "0x3b8ec09e026fdc305365dfc94e189a81b38c7597b3d941c279f042e8206e0bd8",
                            "0xecd50eee38e386bd62be9bedb990706951b65fe053bd9d8a521af753d139e2da",
                            "0xdefff6d330bb5403f63b14f33b578274160de3a50df4efecf0e0db73bcdd3da5",
                            "0x617bdd11f7c0a11f49db22f629387a12da7596f9d1704d7465177c63d88ec7d7",
                            "0x292c23a9aa1d8bea7e2435e555a4a60e379a5a35f3f452bae60121073fb6eead",
                            "0xe1cea92ed99acdcb045a6726b2f87107e8a61620a232cf4d7d5b5766b3952e10",
                            "0x7ad66c0a68c72cb89e4fb4303841966e4062a76ab97451e3b9fb526a5ceb7f82",
                            "0xe026cc5a4aed3c22a58cbd3d2ac754c9352c5436f638042dca99034e83636516",
                            "0x3d04cffd8b46a874edf5cfae63077de85f849a660426697b06a829c70dd1409c",
                            "0xad676aa337a485e4728a0b240d92b3ef7b3c372d06d189322bfd5f61f1e7203e",
                            "0xa2fca4a49658f9fab7aa63289c91b7c7b6c832a6d0e69334ff5b0a3483d09dab",
                            "0x4ebfd9cd7bca2505f7bef59cc1c12ecc708fff26ae4af19abe852afe9e20c862",
                            "0x2def10d13dd169f550f578bda343d9717a138562e0093b380a1120789d53cf10",
                            "0x776a31db34a1a0a7caaf862cffdfff1789297ffadc380bd3d39281d340abd3ad",
                            "0xe2e7610b87a5fdf3a72ebe271287d923ab990eefac64b6e59d79f8b7e08c46e3",
                            "0x504364a5c6858bf98fff714ab5be9de19ed31a976860efbd0e772a2efe23e2e0",
                            "0x4f05f4acb83f5b65168d9fef89d56d4d77b8944015e6b1eed81b0238e2d0dba3",
                            "0x44a6d974c75b07423e1d6d33f481916fdd45830aea11b6347e700cd8b9f0767c",
                            "0xedf260291f734ddac396a956127dde4c34c0cfb8d8052f88ac139658ccf2d507",
                            "0x6075c657a105351e7f0fce53bc320113324a522e8fd52dc878c762551e01a46e",
                            "0x6ca6a3f763a9395f7da16014725ca7ee17e4815c0ff8119bf33f273dee11833b",
                            "0x1c25ef10ffeb3c7d08aa707d17286e0b0d3cbcb50f1bd3b6523b63ba3b52dd0f",
                            "0xfffc43bd08273ccf135fd3cacbeef055418e09eb728d727c4d5d5c556cdea7e3",
                            "0xc5ab8111456b1f28f3c7a0a604b4553ce905cb019c463ee159137af83c350b22",
                            "0x0ff273fcbf4ae0f2bd88d6cf319ff4004f8d7dca70d4ced4e74d2c74139739e6",
                            "0x7fa06ba11241ddd5efdc65d4e39c9f6991b74fd4b81b62230808216c876f827c",
                            "0x7e275adf313a996c7e2950cac67caba02a5ff925ebf9906b58949f3e77aec5b9",
                            "0x8f6162fa308d2b3a15dc33cffac85f13ab349173121645aedf00f471663108be",
                            "0x78ccaaab73373552f207a63599de54d7d8d0c1805f86ce7da15818d09f4cff62",
                        ]);
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
        test_process_state_response(
            state_response,
            &Address::default(),
            &message_tx,
        )
        .await
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
        test_process_state_response(
            state_response,
            &Address::default(),
            &message_tx,
        )
        .await
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
