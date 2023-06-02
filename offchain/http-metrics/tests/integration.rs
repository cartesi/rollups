use axum::Router;
use clap::Parser;
use http_metrics::{config::MetricsConfig, Counter, Metrics, Registry};

#[derive(Default)]
struct TestMetrics {
    counter1: Counter,
    counter2: Counter,
    counter3: Counter,
}

impl Metrics for TestMetrics {
    fn registry(&self) -> http_metrics::Registry {
        let mut registry = Registry::default();
        registry.register("counter1", "Counter 1", self.counter1.clone());
        registry.register("counter2", "Counter 2", self.counter2.clone());
        registry.register("counter3", "Counter 3", self.counter3.clone());
        registry
    }
}

#[tokio::test]
async fn todo() {
    let config = MetricsConfig::parse();

    let app = Router::new()
        .route("/metrics", axum::routing::get(||{

        }))

    let (metrics, join_handle) = {
        let metrics = TestMetrics::default();
        let metrics_host = config.host.clone();
        let metrics_port = config.port;
        let join_handle = metrics.run(metrics_host, metrics_port).await;
        (metrics, join_handle)
    };
}
