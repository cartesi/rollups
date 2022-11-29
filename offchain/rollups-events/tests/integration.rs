// Copyright 2022 Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use backoff::ExponentialBackoff;
use redis::aio::{ConnectionLike, ConnectionManager};
use redis::streams::StreamRangeReply;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use testcontainers::*;

use rollups_events::broker::{Broker, BrokerError, BrokerStream, INITIAL_ID};

const STREAM_KEY: &'static str = "test-stream";
const CONSUME_TIMEOUT: usize = 10;

struct TestState<'d> {
    _node: Container<'d, images::redis::Redis>,
    redis_address: String,
    conn: ConnectionManager,
    backoff: ExponentialBackoff,
}

async fn setup(docker: &clients::Cli) -> TestState {
    let node = docker.run(images::redis::Redis::default());
    let port = node.get_host_port_ipv4(6379);
    let redis_address = format!("redis://127.0.0.1:{}", port);
    let backoff = ExponentialBackoff::default();

    let client =
        Client::open(redis_address.clone()).expect("failed to create client");
    let mut conn = ConnectionManager::new(client)
        .await
        .expect("failed to create connection");
    flushall(&mut conn).await;

    TestState {
        _node: node,
        redis_address,
        conn,
        backoff,
    }
}

async fn teardown(mut state: TestState<'_>) {
    flushall(&mut state.conn).await;
}

async fn flushall(conn: &mut impl ConnectionLike) {
    redis::cmd("FLUSHALL")
        .query_async::<_, ()>(conn)
        .await
        .expect("failed to flushall");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MockPayload {
    data: String,
}

struct MockStream {}

impl BrokerStream for MockStream {
    type Payload = MockPayload;

    fn key(&self) -> &str {
        STREAM_KEY
    }
}

#[test_log::test(tokio::test)]
#[serial_test::serial]
async fn test_it_produces_events() {
    let docker = clients::Cli::default();
    let mut state = setup(&docker).await;
    let mut broker = Broker::new(
        &state.redis_address,
        state.backoff.clone(),
        CONSUME_TIMEOUT,
    )
    .await
    .expect("failed to initialize broker");
    // Produce events using the Broker struct
    const N: usize = 3;
    let mut ids = vec![];
    for i in 0..N {
        let data = MockPayload {
            data: i.to_string(),
        };
        let id = broker
            .produce(&MockStream {}, data)
            .await
            .expect("failed to produce");
        ids.push(id);
    }
    // Check the events directly in Redis
    let reply: StreamRangeReply = state
        .conn
        .xrange(STREAM_KEY, "-", "+")
        .await
        .expect("failed to read");
    assert_eq!(reply.ids.len(), 3);
    for i in 0..N {
        let expected = format!(r#"{{"data":"{}"}}"#, i);
        assert_eq!(reply.ids[i].id, ids[i]);
        assert_eq!(reply.ids[i].get::<String>("payload").unwrap(), expected);
    }
    teardown(state).await;
}

#[test_log::test(tokio::test)]
#[serial_test::serial]
async fn test_it_peeks_in_stream_with_no_events() {
    let docker = clients::Cli::default();
    let state = setup(&docker).await;
    let mut broker = Broker::new(
        &state.redis_address,
        state.backoff.clone(),
        CONSUME_TIMEOUT,
    )
    .await
    .expect("failed to initialize broker");
    let event = broker
        .peek_latest(&MockStream {})
        .await
        .expect("failed to peek");
    assert!(matches!(event, None));
    teardown(state).await;
}

#[test_log::test(tokio::test)]
#[serial_test::serial]
async fn test_it_peeks_in_stream_with_multiple_events() {
    let docker = clients::Cli::default();
    let mut state = setup(&docker).await;
    // Produce multiple events directly in Redis
    const N: usize = 3;
    for i in 0..N {
        let id = format!("1-{}", i);
        let data = format!(r#"{{"data":"{}"}}"#, i);
        let _: String = state
            .conn
            .xadd(STREAM_KEY, id, &[("payload", data)])
            .await
            .expect("failed to add events");
    }
    // Peek the event using the Broker struct
    let mut broker = Broker::new(
        &state.redis_address,
        state.backoff.clone(),
        CONSUME_TIMEOUT,
    )
    .await
    .expect("failed to initialize broker");
    let event = broker
        .peek_latest(&MockStream {})
        .await
        .expect("failed to peek");
    if let Some(event) = event {
        assert_eq!(&event.id, "1-2");
        assert_eq!(&event.payload.data, "2");
    } else {
        panic!("expected some event");
    }
    teardown(state).await;
}

#[test_log::test(tokio::test)]
#[serial_test::serial]
async fn test_it_fails_to_peek_event_in_invalid_format() {
    let docker = clients::Cli::default();
    let mut state = setup(&docker).await;
    // Produce event directly in Redis
    let _: String = state
        .conn
        .xadd(STREAM_KEY, "1-0", &[("wrong_field", "0")])
        .await
        .expect("failed to add events");
    // Peek the event using the Broker struct
    let mut broker = Broker::new(
        &state.redis_address,
        state.backoff.clone(),
        CONSUME_TIMEOUT,
    )
    .await
    .expect("failed to initialize broker");
    let err = broker
        .peek_latest(&MockStream {})
        .await
        .expect_err("failed to get error");
    assert!(matches!(err, BrokerError::InvalidEvent));
    teardown(state).await;
}

#[test_log::test(tokio::test)]
#[serial_test::serial]
async fn test_it_fails_to_peek_event_with_invalid_data_encoding() {
    let docker = clients::Cli::default();
    let mut state = setup(&docker).await;
    // Produce event directly in Redis
    let _: String = state
        .conn
        .xadd(STREAM_KEY, "1-0", &[("payload", "not a json")])
        .await
        .expect("failed to add events");
    // Peek the event using the Broker struct
    let mut broker = Broker::new(
        &state.redis_address,
        state.backoff.clone(),
        CONSUME_TIMEOUT,
    )
    .await
    .expect("failed to initialize broker");
    let err = broker
        .peek_latest(&MockStream {})
        .await
        .expect_err("failed to get error");
    assert!(matches!(err, BrokerError::InvalidPayload { .. }));
    teardown(state).await;
}

#[test_log::test(tokio::test)]
#[serial_test::serial]
async fn test_it_consumes_events() {
    let docker = clients::Cli::default();
    let mut state = setup(&docker).await;
    // Produce multiple events directly in Redis
    const N: usize = 3;
    for i in 0..N {
        let id = format!("1-{}", i);
        let data = format!(r#"{{"data":"{}"}}"#, i);
        let _: String = state
            .conn
            .xadd(STREAM_KEY, id, &[("payload", data)])
            .await
            .expect("failed to add events");
    }
    // Consume events using the Broker struct
    let mut broker = Broker::new(
        &state.redis_address,
        state.backoff.clone(),
        CONSUME_TIMEOUT,
    )
    .await
    .expect("failed to initialize broker");
    let mut last_id = INITIAL_ID.to_owned();
    for i in 0..N {
        let event = broker
            .consume_block(&MockStream {}, &last_id)
            .await
            .expect("failed to consume");
        assert_eq!(event.id, format!("1-{}", i));
        assert_eq!(event.payload.data, i.to_string());
        last_id = event.id;
    }

    teardown(state).await;
}

#[test_log::test(tokio::test)]
#[serial_test::serial]
async fn test_it_blocks_until_event_is_produced() {
    let docker = clients::Cli::default();
    let state = setup(&docker).await;
    // Spawn another thread that sends the event after a few ms
    let handler = {
        let mut conn = state.conn.clone();
        tokio::spawn(async move {
            let duration = std::time::Duration::from_millis(10);
            tokio::time::sleep(duration).await;
            let _: String = conn
                .xadd(STREAM_KEY, "1-0", &[("payload", r#"{"data":"0"}"#)])
                .await
                .expect("failed to write event");
        })
    };
    // In the main thread, wait for the expected event
    let mut broker = Broker::new(
        &state.redis_address,
        state.backoff.clone(),
        CONSUME_TIMEOUT,
    )
    .await
    .expect("failed to initialize broker");
    let event = broker
        .consume_block(&MockStream {}, "0")
        .await
        .expect("failed to consume event");
    assert_eq!(event.id, "1-0");
    assert_eq!(event.payload.data, "0");
    teardown(state).await;
    handler.await.expect("failed to wait handler");
}

#[test_log::test(tokio::test)]
#[serial_test::serial]
async fn test_it_consumes_events_without_blocking() {
    let docker = clients::Cli::default();
    let mut state = setup(&docker).await;
    // Produce multiple events directly in Redis
    const N: usize = 3;
    for i in 0..N {
        let id = format!("1-{}", i);
        let data = format!(r#"{{"data":"{}"}}"#, i);
        let _: String = state
            .conn
            .xadd(STREAM_KEY, id, &[("payload", data)])
            .await
            .expect("failed to add events");
    }
    // Consume events using the Broker struct
    let mut broker = Broker::new(
        &state.redis_address,
        state.backoff.clone(),
        CONSUME_TIMEOUT,
    )
    .await
    .expect("failed to initialize broker");
    let mut last_id = INITIAL_ID.to_owned();
    for i in 0..N {
        let event = broker
            .consume_nonblock(&MockStream {}, &last_id)
            .await
            .expect("failed to consume")
            .expect("expected event, got None");
        assert_eq!(event.id, format!("1-{}", i));
        assert_eq!(event.payload.data, i.to_string());
        last_id = event.id;
    }
    teardown(state).await;
}

#[test_log::test(tokio::test)]
#[serial_test::serial]
async fn test_it_does_not_block_when_consuming_empty_stream() {
    let docker = clients::Cli::default();
    let state = setup(&docker).await;
    let mut broker = Broker::new(
        &state.redis_address,
        state.backoff.clone(),
        CONSUME_TIMEOUT,
    )
    .await
    .expect("failed to initialize broker");
    let event = broker
        .consume_nonblock(&MockStream {}, INITIAL_ID)
        .await
        .expect("failed to peek");
    assert!(matches!(event, None));
    teardown(state).await;
}

#[test_log::test(tokio::test)]
#[serial_test::serial]
async fn test_it_times_out_when_no_event_is_produced() {
    let docker = clients::Cli::default();
    let state = setup(&docker).await;
    let mut broker = Broker::new(
        &state.redis_address,
        state.backoff.clone(),
        CONSUME_TIMEOUT,
    )
    .await
    .expect("failed to initialize broker");
    let err = broker
        .consume_block(&MockStream {}, "0")
        .await
        .expect_err("consume event worked but it should have failed");
    assert!(matches!(err, BrokerError::ConsumeTimeout));
    teardown(state).await;
}
