// Copyright Cartesi Pte. Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use std::time::Duration;

use backoff::ExponentialBackoffBuilder;
use redacted::{RedactedUrl, Url};
use rollups_events::{Broker, BrokerConfig, BrokerStream};
use serde::{Deserialize, Serialize};

const REDIS_TLS_ENDPOINT: &str = "rediss://localhost:6379";
const STREAM_KEY: &'static str = "test-stream";

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

/// This code tests Broker's capability to connect to a Redis server
/// via TLS. It is an unconventional test as it is not supposed to run directly
/// in the host machine, but inside a Docker container created by the
/// tls.rs integration test.
#[tokio::main]
async fn main() {
    let config = create_config();
    let mut broker =
        Broker::new(config).await.expect("Failed to create Broker");

    let stream = MockStream {};
    let payload = MockPayload {
        data: "test-data".into(),
    };
    let id = broker
        .produce(&stream, payload.clone())
        .await
        .expect("Failed to produce an event");

    let event = broker
        .peek_latest(&stream)
        .await
        .expect("Failed to peek at latest event")
        .expect("No event found in the stream");

    assert_eq!(event.id, id);
    assert_eq!(event.payload, payload);
}

fn create_config() -> BrokerConfig {
    let redis_endpoint = Url::parse(REDIS_TLS_ENDPOINT)
        .map(RedactedUrl::new)
        .expect("failed to parse Redis Url");
    let backoff = ExponentialBackoffBuilder::new()
        .with_max_elapsed_time(Some(Duration::from_secs(10)))
        .build();

    BrokerConfig {
        redis_endpoint,
        consume_timeout: 1,
        backoff,
    }
}
