// (c) Cartesi and individual authors (see AUTHORS)
// SPDX-License-Identifier: Apache-2.0 (see LICENSE)

use backoff::ExponentialBackoff;
use rollups_events::indexer::{IndexerEvent, IndexerState};
use rollups_events::{
    Address, Broker, BrokerConfig, BrokerEndpoint, BrokerError, BrokerStream,
    DAppMetadata, Event, Hash, RedactedUrl, RollupsAdvanceStateInput,
    RollupsData, RollupsInput, RollupsInputsStream, RollupsOutput,
    RollupsOutputsStream, Url,
};
use testcontainers::{
    clients::Cli, core::WaitFor, images::generic::GenericImage, Container,
};

pub const CONSUME_TIMEOUT: usize = 10;
pub const CHAIN_ID: u64 = 99;
pub const DAPP_ADDRESS: Address = Address::new([0xfa; 20]);

pub struct TestState<'d> {
    _node: Container<'d, GenericImage>,
    redis_endpoint: RedactedUrl,
}

impl TestState<'_> {
    pub async fn setup(docker: &Cli) -> TestState {
        let image = GenericImage::new("redis", "6.2").with_wait_for(
            WaitFor::message_on_stdout("Ready to accept connections"),
        );
        let node = docker.run(image);
        let port = node.get_host_port_ipv4(6379);
        let redis_endpoint = Url::parse(&format!("redis://127.0.0.1:{}", port))
            .map(RedactedUrl::new)
            .expect("failed to parse Redis Url");
        TestState {
            _node: node,
            redis_endpoint,
        }
    }

    pub async fn create_broker(&self) -> Broker {
        let backoff = ExponentialBackoff::default();
        let config = BrokerConfig {
            redis_endpoint: BrokerEndpoint::Single(self.redis_endpoint.clone()),
            consume_timeout: CONSUME_TIMEOUT,
            backoff,
        };
        Broker::new(config)
            .await
            .expect("failed to initialize broker")
    }
}

#[test_log::test(tokio::test)]
async fn it_times_out_when_no_indexer_event_is_produced() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;
    let broker = state.create_broker().await;
    let mut indexer_state = IndexerState::new(&dapp_metadata());
    let err = broker
        .indexer_consume(&mut indexer_state)
        .await
        .expect_err("consume event worked but it should have failed");
    assert!(matches!(err, BrokerError::ConsumeTimeout));
}

#[test_log::test(tokio::test)]
async fn it_consumes_input_events() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;
    let mut broker = state.create_broker().await;
    // Produce input events
    let inputs = generate_inputs();
    let metadata = dapp_metadata();
    let stream = RollupsInputsStream::new(&metadata);
    produce_all(&mut broker, &stream, &inputs).await;
    // Consume indexer events
    let consumed_events =
        consume_all(&mut broker, &metadata, inputs.len()).await;
    for (event, input) in consumed_events.iter().zip(&inputs) {
        assert!(matches!(event,
            IndexerEvent::Input(
                Event {
                    payload,
                    ..
                }
            )
            if payload == input
        ));
    }
}

#[test_log::test(tokio::test)]
async fn it_consumes_output_events() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;
    let mut broker = state.create_broker().await;
    // Produce output events
    let outputs = generate_outputs();
    let metadata = dapp_metadata();
    let stream = RollupsOutputsStream::new(&metadata);
    produce_all(&mut broker, &stream, &outputs).await;
    // Consume indexer events
    let consumed_events =
        consume_all(&mut broker, &metadata, outputs.len()).await;
    for (event, output) in consumed_events.iter().zip(&outputs) {
        assert!(matches!(event,
            IndexerEvent::Output(
                Event {
                    payload,
                    ..
                }
            )
            if payload == output
        ));
    }
}

#[test_log::test(tokio::test)]
async fn it_consumes_inputs_before_outputs() {
    let docker = Cli::default();
    let state = TestState::setup(&docker).await;
    let mut broker = state.create_broker().await;
    // First, produce output events
    let outputs = generate_outputs();
    let metadata = dapp_metadata();
    let outputs_stream = RollupsOutputsStream::new(&metadata);
    produce_all(&mut broker, &outputs_stream, &outputs).await;
    // Then, produce input events
    let inputs = generate_inputs();
    let inputs_stream = RollupsInputsStream::new(&metadata);
    produce_all(&mut broker, &inputs_stream, &inputs).await;
    // Finally, consume indexer events
    let consumed_events =
        consume_all(&mut broker, &metadata, outputs.len() + inputs.len()).await;
    for (i, input) in inputs.iter().enumerate() {
        assert!(matches!(&consumed_events[i],
            IndexerEvent::Input(
                Event {
                    payload,
                    ..
                }
            )
            if payload == input
        ));
    }
    for (i, output) in outputs.iter().enumerate() {
        assert!(matches!(&consumed_events[inputs.len() + i],
            IndexerEvent::Output(
                Event {
                    payload,
                    ..
                }
            )
            if payload == output
        ));
    }
}

fn dapp_metadata() -> DAppMetadata {
    DAppMetadata {
        chain_id: CHAIN_ID,
        dapp_address: DAPP_ADDRESS.to_owned(),
    }
}

fn generate_outputs() -> Vec<RollupsOutput> {
    vec![
        RollupsOutput::Voucher(Default::default()),
        RollupsOutput::Notice(Default::default()),
        RollupsOutput::Report(Default::default()),
    ]
}

fn generate_inputs() -> Vec<RollupsInput> {
    vec![
        RollupsInput {
            parent_id: "".to_owned(),
            epoch_index: 0,
            inputs_sent_count: 1,
            data: RollupsData::AdvanceStateInput(RollupsAdvanceStateInput {
                metadata: Default::default(),
                payload: Default::default(),
                tx_hash: Hash::default(),
            }),
        },
        RollupsInput {
            parent_id: "".to_owned(),
            epoch_index: 0,
            inputs_sent_count: 1,
            data: RollupsData::FinishEpoch {},
        },
    ]
}

async fn produce_all<S: BrokerStream>(
    broker: &mut Broker,
    stream: &S,
    payloads: &[S::Payload],
) {
    for payload in payloads {
        broker
            .produce(stream, payload.clone())
            .await
            .expect("failed to produce");
    }
}

async fn consume_all(
    broker: &mut Broker,
    dapp_metadata: &DAppMetadata,
    n: usize,
) -> Vec<IndexerEvent> {
    let mut state = IndexerState::new(dapp_metadata);
    let mut payloads = vec![];
    for _ in 0..n {
        let payload = broker
            .indexer_consume(&mut state)
            .await
            .expect("failed to consume indexer payload");
        payloads.push(payload);
    }
    payloads
}
