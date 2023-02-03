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
use diesel::{Connection, RunQueryDsl};
use rstest::*;
use serial_test::serial;
use std::future::Future;

#[allow(dead_code)]
struct Context {
    postgres_endpoint: String,
    reader_service_address: String,
    reader_binary_path: String,
}

impl Drop for Context {
    fn drop(&mut self) {
        try_stop_reader(&self.reader_binary_path);
    }
}

pub const POSTGRES_PORT: u16 = 5435;
pub const POSTGRES_HOSTNAME: &str = "127.0.0.1";
pub const POSTGRES_USER: &str = "postgres";
pub const POSTGRES_PASSWORD: &str = "password";
pub const POSTGRES_DB: &str = "test_reader";
pub const DB_TEST_FILE: &str = "./tests/data/test_db_graphql.tar";
pub const GRAPHQL_HOST: &str = "127.0.0.1";
pub const GRAPHQL_PORT: u16 = 4001;
pub const ROLLUPS_READER_BINARY_PATH: &str = "reader";

pub fn connect_to_database(
    postgres_endpoint: &str,
) -> Result<PgConnection, diesel::ConnectionError> {
    PgConnection::establish(&postgres_endpoint)
}

pub fn perform_db_restore(
    user: &str,
    password: &str,
    host: &str,
    port: u16,
    database: &str,
    backup_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    std::process::Command::new("pg_restore")
        .env("PGPASSWORD", password)
        .arg(&format!("--host={}", host))
        .arg(&format!("--port={}", port))
        .arg(&format!("--username={}", user))
        .arg(&format!("--dbname={}", database))
        .arg(&format!("--format=c"))
        .arg(&format!("-O"))
        .arg(&format!("{}", backup_file))
        .output()
        .expect("Unable to restore database");

    Ok(())
}

#[allow(dead_code)]
pub fn create_and_fill_database(
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

    // Restore data
    perform_db_restore(user, password, host, port, POSTGRES_DB, DB_TEST_FILE)
        .expect("db restore succedded");

    Ok(())
}

fn instantiate_external_reader_instance(
    db_user: &str,
    db_password: &str,
    db_hostname: &str,
    db_port: u16,
    db_name: &str,
    graphql_host: &str,
    graphql_port: u16,
    reader_binary_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting rollups reader...");
    std::process::Command::new(reader_binary_path)
        .arg(&format!("--postgres-hostname={}", db_hostname))
        .arg(&format!("--postgres-port={}", db_port))
        .arg(&format!("--postgres-user={}", db_user))
        .arg(&format!("--postgres-password={}", db_password))
        .arg(&format!("--postgres-db={}", db_name))
        .arg(&format!("--graphql-host={}", graphql_host))
        .arg(&format!("--graphql-port={}", graphql_port))
        .spawn()
        .expect("Unable to launch rollups reader");
    std::thread::sleep(std::time::Duration::from_secs(3));
    Ok(())
}

fn try_stop_reader(reader_binary_path: &str) {
    let result = std::process::Command::new("pkill")
        .arg("-f")
        .arg(reader_binary_path)
        .status()
        .unwrap();
    if !result.success() {
        eprint!("Error stopping rollups reader");
    }
}

#[fixture]
async fn context_reader_db() -> Context {
    // Create database
    create_and_fill_database(
        POSTGRES_USER,
        POSTGRES_PASSWORD,
        POSTGRES_HOSTNAME,
        POSTGRES_PORT,
    )
    .unwrap();

    let reader_binary_path = match std::env::var("ROLLUPS_READER_BINARY_PATH") {
        Ok(path) => path,
        Err(_e) => ROLLUPS_READER_BINARY_PATH.to_string(),
    };
    println!(
        "Instantiating reader service from binary path {}",
        reader_binary_path
    );

    // Start external reader service

    instantiate_external_reader_instance(
        POSTGRES_USER,
        POSTGRES_PASSWORD,
        POSTGRES_HOSTNAME,
        POSTGRES_PORT,
        POSTGRES_DB,
        GRAPHQL_HOST,
        GRAPHQL_PORT,
        &reader_binary_path,
    )
    .expect("rollups reader started, please set ROLLUPS_READER_PATH");

    let postgres_endpoint = format!(
        "postgres://{}:{}@{}:{}/{}",
        POSTGRES_USER,
        POSTGRES_PASSWORD,
        POSTGRES_HOSTNAME,
        POSTGRES_PORT,
        POSTGRES_DB
    );

    let reader_service =
        format!("http://{}:{}/graphql", GRAPHQL_HOST, GRAPHQL_PORT);
    println!("Reader started on address: {} ", &reader_service);

    Context {
        postgres_endpoint,
        reader_service_address: reader_service,
        reader_binary_path,
    }
}

async fn body_to_string(resp: hyper::Response<hyper::Body>) -> String {
    let chunk = hyper::body::to_bytes(resp)
        .await
        .expect("error in hyper body decomposition")
        .to_vec();
    String::from_utf8_lossy(&chunk).to_string()
}

fn build_request<'a>(
    reader_service_address: &'a str,
    body: String,
) -> hyper::Request<hyper::Body> {
    hyper::Request::builder()
        .method(hyper::Method::POST)
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .uri(reader_service_address)
        .body(hyper::Body::from(body))
        .expect("graphql request")
}

async fn process_response<'a>(res: hyper::Response<hyper::Body>) -> String {
    let mut response = body_to_string(res).await.replace(r#"\""#, r#"""#);
    response.retain(|c| !c.is_whitespace());
    println!("Response received:\n{}", response);
    response
}

fn process_expected_response_pattern(pattern: &str) -> String {
    let pattern = pattern.to_string();
    pattern
        .replace("\n", r#""#)
        .replace("\t", r#""#)
        .replace(" ", r#""#)
}

async fn perform_request_check_response(
    context: &Context,
    request: &str,
    expected_response: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = request.to_string();
    let request = request.replace("\n", r#"\n"#).replace("\t", r#"\t"#);
    let req = build_request(&context.reader_service_address, request);
    match hyper::Client::new().request(req).await {
        Ok(res) => {
            assert_eq!(
                process_response(res).await.as_str(),
                process_expected_response_pattern(expected_response).as_str()
            );
            Ok(())
        }
        Err(e) => Err(Box::new(e)),
    }
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_notice_id(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          notice(id:\"5\") {
            id
            index
            proof {
              outputHashesRootHash
              vouchersEpochRootHash
              noticesEpochRootHash
              machineStateHash
              keccakInHashesSiblings
            }
            input {
              id
              index
            }
            keccak
            payload
          }
        }"}"#;

    let expected_response = r#"
        {
          "data": {
            "notice": {
              "id": "5",
              "index": 0,
              "proof": {
                "outputHashesRootHash": "0x4fb87338f8a4b4a14c35a34d6503f45d7bc87fd2fe3aa05aa681c2b8d908f53b",
                "vouchersEpochRootHash": "0x7447f83839ccc19207cd44cbcea62c0fe21429294c397bf2968e3f90039334c8",
                "noticesEpochRootHash": "0x04ae6d1e4a43d5c5fdda99dbd70addfd1df8a519022fca5ffc47e8d387418a8f",
                "machineStateHash": "0x6b0c3f14b3831dc3c9b103fa855a23756bd01649f2f459e9e816cf6e4a3b520a",
                "keccakInHashesSiblings": [
                  "0x99af665835aabfdc6740c7e2c3791a31c3cdc9f5ab962f681b12fc092816a62f",
                  "0x2b573c267a712a52e1d06421fe276a03efb1889f337201110fdc32a81f8e1524",
                  "0x7a71f6ee264c5d761379b3d7d617ca83677374b49d10aec50505ac087408ca89",
                  "0xf7549f26cc70ed5e18baeb6c81bb0625cb95bb4019aeecd40774ee87ae29ec51",
                  "0x2122e31e4bbd2b7c783d79cc30f60c6238651da7f0726f767d22747264fdb046",
                  "0x91e3eee5ca7a3da2b3053c9770db73599fb149f620e3facef95e947c0ee860b7",
                  "0x63e8806fa0d4b197a259e8c3ac28864268159d0ac85f8581ca28fa7d2c0c03eb",
                  "0xc9695393027fb106a8153109ac516288a88b28a93817899460d6310b71cf1e61",
                  "0xd8b96e5b7f6f459e9cb6a2f41bf276c7b85c10cd4662c04cbbb365434726c0a0",
                  "0xcd5deac729d0fdaccc441d09d7325f41586ba13c801b7eccae0f95d8f3933efe",
                  "0x30b0b9deb73e155c59740bacf14a6ff04b64bb8e201a506409c3fe381ca4ea90",
                  "0x8e7a427fa943d9966b389f4f257173676090c6e95f43e2cb6d65f8758111e309",
                  "0xc37b8b13ca95166fb7af16988a70fcc90f38bf9126fd833da710a47fb37a55e6",
                  "0x17d2dd614cddaa4d879276b11e0672c9560033d3e8453a1d045339d34ba601b9",
                  "0x3fc9a15f5b4869c872f81087bb6104b7d63e6f9ab47f2c43f3535eae7172aa7f",
                  "0xae39ce8537aca75e2eff3e38c98011dfe934e700a0967732fc07b430dd656a23"
                ]
              },
              "input": {
                "id": "16",
                "index": 15
              },
              "keccak": "0x305ed2d53368d90b31aa94966d0f9989379267b8da09dee41355e9eb16b68897",
              "payload": "0x5b5b224a696d6d79222c2033315d2c205b224c61796c61222c2032385d2c205b224d617261222c2032395d2c205b224d697261222c2032345d2c205b224d617361222c2033315d5d"
            }
          }
        }"#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_notice_count(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
  notices {
    totalCount
    pageInfo {
      startCursor
      endCursor
      hasNextPage
      hasPreviousPage
    }
  }
}"}"#;

    let expected_response = r#"{
  "data": {
    "notices": {
      "totalCount": 12,
      "pageInfo": {
        "startCursor": "1",
        "endCursor": "12",
        "hasNextPage": false,
        "hasPreviousPage": false
      }
    }
  }
}"#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_notices_edges(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query":"query {
              notices {
                totalCount
                edges {
                  node {
                    id
                    index
                    input {
                      id
                      index
                      epoch {
                        id
                        index
                      }
                        }
                    keccak
                  }
                  cursor
                }
                pageInfo {
                  startCursor
                  endCursor
                  hasNextPage
                  hasPreviousPage
                }
              }
            }"}"#;

    let expected_response = r#"
         {
          "data": {
            "notices": {
              "totalCount": 12,
              "edges": [
                {
                  "node": {
                    "id": "1",
                    "index": 0,
                    "input": {
                      "id": "5",
                      "index": 4,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0xac00532afbe52b5428b9201fdc89cc1e555089c37e7ffee5f8d0bb12f90d0f79"
                  },
                  "cursor": "1"
                },
                {
                  "node": {
                    "id": "2",
                    "index": 0,
                    "input": {
                      "id": "7",
                      "index": 6,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0xcc80b6a8b477cafee279235b92ad1c766d611690767c73a5d34d2c1a5d5e02d0"
                  },
                  "cursor": "2"
                },
                {
                  "node": {
                    "id": "3",
                    "index": 0,
                    "input": {
                      "id": "11",
                      "index": 10,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0x965267f4e93d304297751bfbb849cde28a82d9da33d04d411657207c35c636a2"
                  },
                  "cursor": "3"
                },
                {
                  "node": {
                    "id": "4",
                    "index": 0,
                    "input": {
                      "id": "13",
                      "index": 12,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0x53d7fecd171a8132ecb0aed2f2c07ae4336f61b2a10bf9c2f47cacd5aca08857"
                  },
                  "cursor": "4"
                },
                {
                  "node": {
                    "id": "5",
                    "index": 0,
                    "input": {
                      "id": "16",
                      "index": 15,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0x305ed2d53368d90b31aa94966d0f9989379267b8da09dee41355e9eb16b68897"
                  },
                  "cursor": "5"
                },
                {
                  "node": {
                    "id": "6",
                    "index": 0,
                    "input": {
                      "id": "18",
                      "index": 17,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0x305ed2d53368d90b31aa94966d0f9989379267b8da09dee41355e9eb16b68897"
                  },
                  "cursor": "6"
                },
                {
                  "node": {
                    "id": "7",
                    "index": 0,
                    "input": {
                      "id": "22",
                      "index": 21,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0xb05a614552507742bf1d4cd444b4fab6201560649daa0116ec93160c31cb4ef7"
                  },
                  "cursor": "7"
                },
                {
                  "node": {
                    "id": "8",
                    "index": 0,
                    "input": {
                      "id": "23",
                      "index": 22,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0xb05a614552507742bf1d4cd444b4fab6201560649daa0116ec93160c31cb4ef7"
                  },
                  "cursor": "8"
                },
                {
                  "node": {
                    "id": "9",
                    "index": 0,
                    "input": {
                      "id": "27",
                      "index": 26,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0xb66fcbae74d8a74a2b90d35dd6dd4828ee643cb59643eced537a59e9cbb56cd6"
                  },
                  "cursor": "9"
                },
                {
                  "node": {
                    "id": "10",
                    "index": 0,
                    "input": {
                      "id": "29",
                      "index": 28,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0xc05c56e0708c5bcbefa98308a5a6386903222dec877613411218c0b8db7fa12a"
                  },
                  "cursor": "10"
                },
                {
                  "node": {
                    "id": "11",
                    "index": 0,
                    "input": {
                      "id": "31",
                      "index": 30,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0xc05c56e0708c5bcbefa98308a5a6386903222dec877613411218c0b8db7fa12a"
                  },
                  "cursor": "11"
                },
                {
                  "node": {
                    "id": "12",
                    "index": 0,
                    "input": {
                      "id": "33",
                      "index": 32,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0x352f88223360684646158ffa6cb6a21b1cfcda8679752ac6b312fbe51d64501e"
                  },
                  "cursor": "12"
                }
              ],
              "pageInfo": {
                "startCursor": "1",
                "endCursor": "12",
                "hasNextPage": false,
                "hasPreviousPage": false
              }
            }
          }
        }"#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_notices_edges_nodes_cursor(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query":"query {
    notices (first: 10, last: 3, after:\"2\",before:\"11\") {
    totalCount
    edges {
      node {
        id
        index
        input {
          id
          index
          epoch {
            id
            index
          }
    		}
        keccak
      }
      cursor
    }
    nodes {
        id
        index
        input {
          id
          index
          epoch {
            id
            index
          }
    		}
        keccak
    }
    pageInfo {
      startCursor
      endCursor
      hasNextPage
      hasPreviousPage
    }
  }
}"}"#;

    let expected_response = r#"
         {
          "data": {
            "notices": {
              "totalCount": 12,
              "edges": [
                {
                  "node": {
                    "id": "8",
                    "index": 0,
                    "input": {
                      "id": "23",
                      "index": 22,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0xb05a614552507742bf1d4cd444b4fab6201560649daa0116ec93160c31cb4ef7"
                  },
                  "cursor": "8"
                },
                {
                  "node": {
                    "id": "9",
                    "index": 0,
                    "input": {
                      "id": "27",
                      "index": 26,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0xb66fcbae74d8a74a2b90d35dd6dd4828ee643cb59643eced537a59e9cbb56cd6"
                  },
                  "cursor": "9"
                },
                {
                  "node": {
                    "id": "10",
                    "index": 0,
                    "input": {
                      "id": "29",
                      "index": 28,
                      "epoch": {
                        "id": "1",
                        "index": 0
                      }
                    },
                    "keccak": "0xc05c56e0708c5bcbefa98308a5a6386903222dec877613411218c0b8db7fa12a"
                  },
                  "cursor": "10"
                }
              ],
              "nodes": [
                {
                  "id": "8",
                  "index": 0,
                  "input": {
                    "id": "23",
                    "index": 22,
                    "epoch": {
                      "id": "1",
                      "index": 0
                    }
                  },
                  "keccak": "0xb05a614552507742bf1d4cd444b4fab6201560649daa0116ec93160c31cb4ef7"
                },
                {
                  "id": "9",
                  "index": 0,
                  "input": {
                    "id": "27",
                    "index": 26,
                    "epoch": {
                      "id": "1",
                      "index": 0
                    }
                  },
                  "keccak": "0xb66fcbae74d8a74a2b90d35dd6dd4828ee643cb59643eced537a59e9cbb56cd6"
                },
                {
                  "id": "10",
                  "index": 0,
                  "input": {
                    "id": "29",
                    "index": 28,
                    "epoch": {
                      "id": "1",
                      "index": 0
                    }
                  },
                  "keccak": "0xc05c56e0708c5bcbefa98308a5a6386903222dec877613411218c0b8db7fa12a"
                }
              ],
              "pageInfo": {
                "startCursor": "8",
                "endCursor": "10",
                "hasNextPage": true,
                "hasPreviousPage": true
              }
            }
          }
        }
        "#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_report_id(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          report(id:\"5\") {
            id
            index
            input {
              id
              index
              epoch {
                id
                index
                input (index: 0) {
                  id
                  index
                }
              }
              blockNumber
            }
            payload

          }
        }"}"#;

    let expected_response = r#"
        {
          "data": {
            "report": {
              "id": "5",
              "index": 0,
              "input": {
                "id": "17",
                "index": 16,
                "epoch": {
                  "id": "1",
                  "index": 0,
                  "input": {
                    "id": "1",
                    "index": 0
                  }
                },
                "blockNumber": 49
              },
              "payload": "0x4572726f7220657865637574696e672073746174656d656e74202753454c45273a206e656172202253454c45223a2073796e746178206572726f72"
            }
          }
        }"#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_reports_count(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
  reports  {
    totalCount
    nodes {
      id
      index
    }
    pageInfo {
      startCursor
      endCursor
      hasNextPage
      hasPreviousPage
    }
  }
}"}"#;

    let expected_response = r#"
    {
      "data": {
        "reports": {
          "totalCount": 16,
          "nodes": [
            {
              "id": "1",
              "index": 0
            },
            {
              "id": "2",
              "index": 0
            },
            {
              "id": "3",
              "index": 0
            },
            {
              "id": "4",
              "index": 0
            },
            {
              "id": "5",
              "index": 0
            },
            {
              "id": "6",
              "index": 0
            },
            {
              "id": "7",
              "index": 0
            },
            {
              "id": "8",
              "index": 0
            },
            {
              "id": "9",
              "index": 0
            },
            {
              "id": "10",
              "index": 0
            },
            {
              "id": "11",
              "index": 0
            },
            {
              "id": "12",
              "index": 0
            },
            {
              "id": "13",
              "index": 0
            },
            {
              "id": "14",
              "index": 0
            },
            {
              "id": "15",
              "index": 0
            },
            {
              "id": "16",
              "index": 0
            }
          ],
          "pageInfo": {
            "startCursor": "1",
            "endCursor": "16",
            "hasNextPage": false,
            "hasPreviousPage": false
          }
        }
      }
    }"#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_reports_cursors(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
      reports (first: 7, last: 6, after:\"2\",before:\"11\")  {
        totalCount
        nodes {
          id
          index
        }
        pageInfo {
          startCursor
          endCursor
          hasNextPage
          hasPreviousPage
        }
      }
    }"}"#;

    let expected_response = r#"
           {
          "data": {
            "reports": {
              "totalCount": 16,
              "nodes": [
                {
                  "id": "4",
                  "index": 0
                },
                {
                  "id": "5",
                  "index": 0
                },
                {
                  "id": "6",
                  "index": 0
                },
                {
                  "id": "7",
                  "index": 0
                },
                {
                  "id": "8",
                  "index": 0
                },
                {
                  "id": "9",
                  "index": 0
                }
              ],
              "pageInfo": {
                "startCursor": "4",
                "endCursor": "9",
                "hasNextPage": true,
                "hasPreviousPage": true
              }
            }
          }
        }
   "#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_reports_edges(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          reports (first: 2, last: 3)  {
            totalCount
            nodes {
              id
              index
            }
            edges {
              node {
                id
                index
                input {
                  id
                  index
                  epoch {
                    id
                    index
                    __typename
                  }
                }
                payload
              }
            }
            pageInfo {
              startCursor
              endCursor
              hasNextPage
              hasPreviousPage
            }
          }
        }"}"#;

    let expected_response = r#"
       {
          "data": {
            "reports": {
              "totalCount": 16,
              "nodes": [
                {
                  "id": "1",
                  "index": 0
                },
                {
                  "id": "2",
                  "index": 0
                }
              ],
              "edges": [
                {
                  "node": {
                    "id": "1",
                    "index": 0,
                    "input": {
                      "id": "4",
                      "index": 3,
                      "epoch": {
                        "id": "1",
                        "index": 0,
                        "__typename": "Epoch"
                      }
                    },
                    "payload": "0x4572726f7220657865637574696e672073746174656d656e74202753454c454354202a2046524f20506572736f6e73273a206e656172202246524f223a2073796e746178206572726f72"
                  }
                },
                {
                  "node": {
                    "id": "2",
                    "index": 0,
                    "input": {
                      "id": "8",
                      "index": 7,
                      "epoch": {
                        "id": "1",
                        "index": 0,
                        "__typename": "Epoch"
                      }
                    },
                    "payload": "0x4572726f7220657865637574696e672073746174656d656e74202753454c45273a206e656172202253454c45223a2073796e746178206572726f72"
                  }
                }
              ],
              "pageInfo": {
                "startCursor": "1",
                "endCursor": "2",
                "hasNextPage": true,
                "hasPreviousPage": false
              }
            }
          }
        }
  "#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_input_id(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "
        query {
          input(id:\"5\") {
            id
            index
            epoch {
              id
              index
            }
            blockNumber
            notices {
              totalCount
              nodes {
                id
                index
                keccak
              }
            }
          }
        }"}"#;

    let expected_response = r#"
     {
      "data": {
        "input": {
          "id": "5",
          "index": 4,
          "epoch": {
            "id": "1",
            "index": 0
          },
          "blockNumber": 37,
          "notices": {
            "totalCount": 1,
            "nodes": [
              {
                "id": "1",
                "index": 0,
                "keccak": "0xac00532afbe52b5428b9201fdc89cc1e555089c37e7ffee5f8d0bb12f90d0f79"
              }
            ]
          }
        }
      }
    }"#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_input_subfields(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          input(id:\"5\") {
            id
            index
            epoch {
              id
              index
            }
            notice(index: 0) {
              id
              index
              payload
            }
          }
        }"}"#;

    let expected_response = r#"
        {
          "data": {
            "input": {
              "id": "5",
              "index": 4,
              "epoch": {
                "id": "1",
                "index": 0
              },
              "notice": {
                "id": "1",
                "index": 0,
                "payload": "0x5b5b224a696d6d79222c2033315d5d"
              }
            }
          }
        }"#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_input_all_subfields(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
  inputs (first: 10, last: 5)  {
    nodes {
      id
      index
      epoch {
        id
        index
      }
      msgSender
      timestamp
      blockNumber
      vouchers {
        totalCount
        nodes {
          id
          index
        }
      }
      notices {
        totalCount
        nodes {
          id
          index
        }
      }
      reports {
        totalCount
        nodes {
          id
          index
        }
      }

    }
	}
}"}"#;

    let expected_response = r#"
        {
          "data": {
            "inputs": {
              "nodes": [
                {
                  "id": "6",
                  "index": 5,
                  "epoch": {
                    "id": "1",
                    "index": 0
                  },
                  "msgSender": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
                  "timestamp": 1655711339,
                  "blockNumber": 38,
                  "vouchers": {
                    "totalCount": 0,
                    "nodes": []
                  },
                  "notices": {
                    "totalCount": 0,
                    "nodes": []
                  },
                  "reports": {
                    "totalCount": 0,
                    "nodes": []
                  }
                },
                {
                  "id": "7",
                  "index": 6,
                  "epoch": {
                    "id": "1",
                    "index": 0
                  },
                  "msgSender": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
                  "timestamp": 1655711344,
                  "blockNumber": 39,
                  "vouchers": {
                    "totalCount": 0,
                    "nodes": []
                  },
                  "notices": {
                    "totalCount": 1,
                    "nodes": [
                      {
                        "id": "2",
                        "index": 0
                      }
                    ]
                  },
                  "reports": {
                    "totalCount": 0,
                    "nodes": []
                  }
                },
                {
                  "id": "8",
                  "index": 7,
                  "epoch": {
                    "id": "1",
                    "index": 0
                  },
                  "msgSender": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
                  "timestamp": 1655711359,
                  "blockNumber": 40,
                  "vouchers": {
                    "totalCount": 0,
                    "nodes": []
                  },
                  "notices": {
                    "totalCount": 0,
                    "nodes": []
                  },
                  "reports": {
                    "totalCount": 1,
                    "nodes": [
                      {
                        "id": "2",
                        "index": 0
                      }
                    ]
                  }
                },
                {
                  "id": "9",
                  "index": 8,
                  "epoch": {
                    "id": "1",
                    "index": 0
                  },
                  "msgSender": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
                  "timestamp": 1655711369,
                  "blockNumber": 41,
                  "vouchers": {
                    "totalCount": 0,
                    "nodes": []
                  },
                  "notices": {
                    "totalCount": 0,
                    "nodes": []
                  },
                  "reports": {
                    "totalCount": 0,
                    "nodes": []
                  }
                },
                {
                  "id": "10",
                  "index": 9,
                  "epoch": {
                    "id": "1",
                    "index": 0
                  },
                  "msgSender": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
                  "timestamp": 1655711373,
                  "blockNumber": 42,
                  "vouchers": {
                    "totalCount": 0,
                    "nodes": []
                  },
                  "notices": {
                    "totalCount": 0,
                    "nodes": []
                  },
                  "reports": {
                    "totalCount": 1,
                    "nodes": [
                      {
                        "id": "3",
                        "index": 0
                      }
                    ]
                  }
                }
              ]
            }
          }
        }
      "#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_inputs_count(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          inputs  {
            totalCount
            nodes {
              id
              index
            }
            pageInfo {
              startCursor
              endCursor
              hasNextPage
              hasPreviousPage
            }
          }
        }"}"#;

    let expected_response = r#"
        {
          "data": {
            "inputs": {
              "totalCount": 39,
              "nodes": [
                {
                  "id": "1",
                  "index": 0
                },
                {
                  "id": "2",
                  "index": 1
                },
                {
                  "id": "3",
                  "index": 2
                },
                {
                  "id": "4",
                  "index": 3
                },
                {
                  "id": "5",
                  "index": 4
                },
                {
                  "id": "6",
                  "index": 5
                },
                {
                  "id": "7",
                  "index": 6
                },
                {
                  "id": "8",
                  "index": 7
                },
                {
                  "id": "9",
                  "index": 8
                },
                {
                  "id": "10",
                  "index": 9
                },
                {
                  "id": "11",
                  "index": 10
                },
                {
                  "id": "12",
                  "index": 11
                },
                {
                  "id": "13",
                  "index": 12
                },
                {
                  "id": "14",
                  "index": 13
                },
                {
                  "id": "15",
                  "index": 14
                },
                {
                  "id": "16",
                  "index": 15
                },
                {
                  "id": "17",
                  "index": 16
                },
                {
                  "id": "18",
                  "index": 17
                },
                {
                  "id": "19",
                  "index": 18
                },
                {
                  "id": "20",
                  "index": 19
                },
                {
                  "id": "21",
                  "index": 20
                },
                {
                  "id": "22",
                  "index": 21
                },
                {
                  "id": "23",
                  "index": 22
                },
                {
                  "id": "24",
                  "index": 23
                },
                {
                  "id": "25",
                  "index": 24
                },
                {
                  "id": "26",
                  "index": 25
                },
                {
                  "id": "27",
                  "index": 26
                },
                {
                  "id": "28",
                  "index": 27
                },
                {
                  "id": "29",
                  "index": 28
                },
                {
                  "id": "30",
                  "index": 29
                },
                {
                  "id": "31",
                  "index": 30
                },
                {
                  "id": "32",
                  "index": 31
                },
                {
                  "id": "33",
                  "index": 32
                },
                {
                  "id": "34",
                  "index": 33
                },
                {
                  "id": "35",
                  "index": 34
                },
                {
                  "id": "36",
                  "index": 35
                },
                {
                  "id": "37",
                  "index": 36
                },
                {
                  "id": "38",
                  "index": 37
                },
                {
                  "id": "39",
                  "index": 38
                }
              ],
              "pageInfo": {
                "startCursor": "1",
                "endCursor": "39",
                "hasNextPage": false,
                "hasPreviousPage": false
              }
            }
          }
        }"#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_inputs_cursors(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          inputs (first: 5, last: 6, after:\"2\",before:\"11\")  {
            totalCount
            nodes {
              id
              index
            }
            pageInfo {
              startCursor
              endCursor
              hasNextPage
              hasPreviousPage
            }
          }
        }"}"#;

    let expected_response = r#"
        {
          "data": {
            "inputs": {
              "totalCount": 39,
              "nodes": [
                {
                  "id": "3",
                  "index": 2
                },
                {
                  "id": "4",
                  "index": 3
                },
                {
                  "id": "5",
                  "index": 4
                },
                {
                  "id": "6",
                  "index": 5
                },
                {
                  "id": "7",
                  "index": 6
                }
              ],
              "pageInfo": {
                "startCursor": "3",
                "endCursor": "7",
                "hasNextPage": true,
                "hasPreviousPage": true
              }
            }
          }
        }
    "#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_inputs_edges(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          inputs (first: 1, last: 3)  {
            totalCount
            edges {
              node {
                id
                index
                epoch {
                  id
                  index
                }
                timestamp
                msgSender
                blockNumber
              }
              cursor
            }
            pageInfo {
              startCursor
              endCursor
              hasNextPage
              hasPreviousPage
            }
          }
        }"}"#;

    let expected_response = r#"
        {
          "data": {
            "inputs": {
              "totalCount": 39,
              "edges": [
                {
                  "node": {
                    "id": "1",
                    "index": 0,
                    "epoch": {
                      "id": "1",
                      "index": 0
                    },
                    "timestamp": 1655711157,
                    "msgSender": "0xa37ae2b259d35af4abdde122ec90b204323ed304",
                    "blockNumber": 33
                  },
                  "cursor": "1"
                }
              ],
              "pageInfo": {
                "startCursor": "1",
                "endCursor": "1",
                "hasNextPage": true,
                "hasPreviousPage": false
              }
            }
          }
        }
       "#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_epoch_id(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          epoch(id: \"1\") {
            id
            index
            inputs (first:4) {
              nodes {
                id
                index
                epoch {
                  index
                }
                notices {
                  nodes {
                    id
                    index
                    keccak
                  }
                }
              }
            }
          }
        }"}"#;

    let expected_response = r#"
        {
          "data": {
            "epoch": {
              "id": "1",
              "index": 0,
              "inputs": {
                "nodes": [
                  {
                    "id": "1",
                    "index": 0,
                    "epoch": {
                      "index": 0
                    },
                    "notices": {
                      "nodes": []
                    }
                  },
                  {
                    "id": "2",
                    "index": 1,
                    "epoch": {
                      "index": 0
                    },
                    "notices": {
                      "nodes": []
                    }
                  },
                  {
                    "id": "3",
                    "index": 2,
                    "epoch": {
                      "index": 0
                    },
                    "notices": {
                      "nodes": []
                    }
                  },
                  {
                    "id": "4",
                    "index": 3,
                    "epoch": {
                      "index": 0
                    },
                    "notices": {
                      "nodes": []
                    }
                  }
                ]
              }
            }
          }
        }
        "#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_epoch_index(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          epochI(index:0) {
            id
            index
            inputs (first:4) {
              nodes {
                id
                index
                epoch {
                  index
                }
                notices {
                  nodes {
                    id
                    index
                    keccak
                  }
                }
              }
            }
          }
        }"}"#;

    let expected_response = r#"
        {
          "data": {
            "epochI": {
              "id": "1",
              "index": 0,
              "inputs": {
                "nodes": [
                  {
                    "id": "1",
                    "index": 0,
                    "epoch": {
                      "index": 0
                    },
                    "notices": {
                      "nodes": []
                    }
                  },
                  {
                    "id": "2",
                    "index": 1,
                    "epoch": {
                      "index": 0
                    },
                    "notices": {
                      "nodes": []
                    }
                  },
                  {
                    "id": "3",
                    "index": 2,
                    "epoch": {
                      "index": 0
                    },
                    "notices": {
                      "nodes": []
                    }
                  },
                  {
                    "id": "4",
                    "index": 3,
                    "epoch": {
                      "index": 0
                    },
                    "notices": {
                      "nodes": []
                    }
                  }
                ]
              }
            }
          }
        }
      "#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_epoch_subfields(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "
        query {
          epochs (first:1, last:2) {
            totalCount
            nodes {
              id
              index
              inputs (first:1, last:5, after:\"3\", before: \"10\") {
                totalCount
                nodes {
                  id
                  index
                }
              }
              input(index: 1) {
                id
                index
              }
            }
            }
        }"}"#;

    let expected_response = r#"
        {
          "data": {
            "epochs": {
              "totalCount": 1,
              "nodes": [
                {
                  "id": "1",
                  "index": 0,
                  "inputs": {
                    "totalCount": 39,
                    "nodes": [
                      {
                        "id": "4",
                        "index": 3
                      }
                    ]
                  },
                  "input": {
                    "id": "2",
                    "index": 1
                  }
                }
              ]
            }
          }
        }
      "#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_epochs_count(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          epochs {
            totalCount
            nodes {
              id
              index
            }
            pageInfo {
              startCursor
              endCursor
              hasNextPage
              hasPreviousPage
            }
          }
        }"}"#;

    let expected_response = r#"{
          "data": {
            "epochs": {
              "totalCount": 1,
              "nodes": [
                {
                  "id": "1",
                  "index": 0
                }
              ],
              "pageInfo": {
                "startCursor": "1",
                "endCursor": "1",
                "hasNextPage": false,
                "hasPreviousPage": false
              }
            }
          }
        }"#;

    perform_request_check_response(&context, request, expected_response).await
}

#[rstest]
#[tokio::test]
#[serial]
async fn test_graphql_epochs_edges_cursors(
    context_reader_db: impl Future<Output = Context>,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = context_reader_db.await;

    let request = r#"{"query": "query {
          epochs (first:1, last:2) {
            totalCount
            edges {
              node {
                id
                index
              }
              cursor
            }
            nodes {
              id
              index
            }
            pageInfo {
              startCursor
              endCursor
              hasNextPage
              hasPreviousPage
            }
          }
        }"}"#;

    let expected_response = r#"
        {
          "data": {
            "epochs": {
              "totalCount": 1,
              "edges": [
                {
                  "node": {
                    "id": "1",
                    "index": 0
                  },
                  "cursor": "1"
                }
              ],
              "nodes": [
                {
                  "id": "1",
                  "index": 0
                }
              ],
              "pageInfo": {
                "startCursor": "1",
                "endCursor": "1",
                "hasNextPage": false,
                "hasPreviousPage": false
              }
            }
          }
        }"#;

    perform_request_check_response(&context, request, expected_response).await
}
